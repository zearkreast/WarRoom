//! WarRoom collector: polls the Helldivers 2 API and writes to Supabase.
//!
//!   warroom-collector          run forever
//!   warroom-collector --once   run every job once and exit (non-zero on failure)
//!
//! Config comes from the environment, or from collector/.env when present.

mod hd2;
mod http;
mod language;
mod push;
mod recorder;
mod supabase;
mod tasks;

use hd2::{Clock, Hd2Client, Options, SystemClock};
use std::collections::HashMap;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tasks::{Collector, Job};

const TICK_MS: f64 = 5_000.0;
/// With no job succeeding for this long, exit so Docker or systemd starts a clean process.
const STALL_MS: f64 = 10.0 * 60_000.0;

fn stamp() -> String {
    warroom_core::time::iso_string(SystemClock.now())[11..19].to_string()
}

fn log(msg: &str) {
    println!("[{}] {msg}", stamp());
}

/// Reads KEY=VALUE lines into the environment; variables already set win, as with Node's env files.
fn load_env_file(path: &str) {
    let Ok(text) = std::fs::read_to_string(path) else { return };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let key = key.trim().trim_start_matches("export ").trim();
        let mut value = value.trim();
        if value.len() >= 2
            && (value.starts_with('"') && value.ends_with('"') || value.starts_with('\'') && value.ends_with('\''))
        {
            value = &value[1..value.len() - 1];
        }
        if std::env::var_os(key).is_none() {
            // SAFETY: called at start-up, before any other thread exists.
            unsafe { std::env::set_var(key, value) };
        }
    }
}

fn env(name: &str) -> String {
    std::env::var(name).map(|v| v.trim().to_string()).unwrap_or_default()
}

fn run() -> Result<u8, String> {
    load_env_file("collector/.env");
    load_env_file(".env");
    let missing: Vec<&str> =
        ["SUPABASE_URL", "SUPABASE_SECRET_KEY", "SUPER_CONTACT"].into_iter().filter(|n| env(n).is_empty()).collect();
    if !missing.is_empty() {
        eprintln!("Missing {}. Copy collector/.env.example to collector/.env and fill it in.", missing.join(", "));
        return Ok(2);
    }

    let once = std::env::args().any(|a| a == "--once");
    let client = if env("SUPER_CLIENT").is_empty() { "warcommand-tracker".to_string() } else { env("SUPER_CLIENT") };
    let contact = env("SUPER_CONTACT");
    let db = || supabase::Supabase::new(&env("SUPABASE_URL"), &env("SUPABASE_SECRET_KEY"), Box::new(http::Http));
    let hd2 = Hd2Client::new(
        Options::new(&client, &contact, "https://api.helldivers2.dev"),
        Box::new(http::Http),
        Box::new(SystemClock),
    );
    let game = Hd2Client::new(
        Options::new(&client, &contact, "https://api.live.prod.thehelldiversgame.com"),
        Box::new(http::Http),
        Box::new(SystemClock),
    );
    let push: Option<Box<dyn push::Push>> = match env("FCM_SERVICE_ACCOUNT").as_str() {
        "" => None,
        value => {
            let account = push::parse_service_account(value)?;
            Some(Box::new(push::Fcm::new(account, Box::new(http::Http), Box::new(SystemClock))?))
        }
    };

    let mut recorder = recorder::Recorder::default();
    recorder.load(&db(), SystemClock.now())?;
    log(&format!(
        "started ({} planets known, pushes {}){}",
        recorder.known_planets(),
        if push.is_some() { "on" } else { "off" },
        if once { ", single run" } else { "" }
    ));
    let mut collector =
        Collector::new(Box::new(hd2), db(), recorder, Box::new(|| SystemClock.now()), Some(Box::new(game)), push);
    let jobs = collector.jobs();

    if once {
        let mut failed = false;
        for job in jobs {
            let r = collector.run(job);
            log(&format!("{} {}: {}", if r.ok { "ok  " } else { "FAIL" }, r.name, r.message));
            failed |= !r.ok;
        }
        return Ok(u8::from(failed));
    }

    let stopping = Arc::new(AtomicBool::new(false));
    for sig in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        signal_hook::flag::register(sig, stopping.clone()).map_err(|e| e.to_string())?;
    }

    // Everything is due at start; each job then waits its own interval.
    let mut due: HashMap<Job, f64> = jobs.iter().map(|j| (*j, 0.0)).collect();
    let mut last_ok = SystemClock.now();
    while !stopping.load(Ordering::Relaxed) {
        let mut parts = Vec::new();
        for &job in &jobs {
            if stopping.load(Ordering::Relaxed) || SystemClock.now() < due[&job] {
                continue;
            }
            let r = collector.run(job);
            if r.ok {
                last_ok = SystemClock.now();
            }
            due.insert(job, r.retry_at.unwrap_or_else(|| SystemClock.now() + job.every_ms()));
            parts.push(format!("{}{}: {}", if r.ok { "" } else { "FAIL " }, r.name, r.message));
        }
        if !parts.is_empty() {
            log(&parts.join(" | "));
        }
        if SystemClock.now() - last_ok > STALL_MS {
            log(&format!("no job has succeeded for {} minutes, exiting for a clean restart", STALL_MS / 60_000.0));
            return Ok(1);
        }
        let until = SystemClock.now() + TICK_MS;
        while !stopping.load(Ordering::Relaxed) && SystemClock.now() < until {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
    log("stop signal received, stopped after the current job");
    Ok(0)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("[{}] fatal: {e}", stamp());
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests;
