//! Polling jobs: each fetches from the API, writes to Supabase, and returns a short status for the log.

use crate::hd2::{Api, ApiError};
use crate::push::{Push, Topic};
use crate::recorder::Recorder;
use crate::supabase::{Supabase, unless_missing};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::fmt;
use warroom_core::api::{
    Assignment, Campaign, Dispatch, EpisodeRaw, Planet, War, WarStatusRaw, from_value, to_js_json,
};
use warroom_core::format::dispatch_title;
use warroom_core::snapshot::{
    DISPATCH_LIMIT, EpisodeLive, EpisodeStatus, OrderSample, SnapshotKind, StationLive, add_order_samples,
    merge_episodes, to_campaign_live, to_effects_live, to_episodes, to_live, to_static, to_war_sample,
};
use warroom_core::time::iso_string;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Job {
    Planets,
    Campaigns,
    War,
    Assignments,
    Dispatches,
    Stations,
    Compact,
    Episodes,
}

impl Job {
    pub fn name(self) -> &'static str {
        match self {
            Job::Planets => "planets",
            Job::Campaigns => "campaigns",
            Job::War => "war",
            Job::Assignments => "assignments",
            Job::Dispatches => "dispatches",
            Job::Stations => "stations",
            Job::Compact => "compact",
            Job::Episodes => "episodes",
        }
    }

    pub fn every_ms(self) -> f64 {
        match self {
            Job::Campaigns => 60_000.0,
            Job::War => 120_000.0,
            Job::Compact => 3_600_000.0,
            Job::Episodes => 900_000.0,
            _ => 300_000.0,
        }
    }
}

#[derive(Debug)]
pub enum JobError {
    Api(ApiError),
    Other(String),
}

impl JobError {
    pub fn retry_after_ms(&self) -> Option<f64> {
        match self {
            JobError::Api(e) => e.retry_after_ms(),
            JobError::Other(_) => None,
        }
    }
}

impl fmt::Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobError::Api(e) => e.fmt(f),
            JobError::Other(s) => f.write_str(s),
        }
    }
}

impl From<ApiError> for JobError {
    fn from(e: ApiError) -> Self {
        JobError::Api(e)
    }
}

impl From<String> for JobError {
    fn from(e: String) -> Self {
        JobError::Other(e)
    }
}

fn typed<T: serde::de::DeserializeOwned>(v: Value) -> Result<T, JobError> {
    from_value(v).map_err(|e| JobError::Other(e.to_string()))
}

pub struct RunResult {
    pub name: &'static str,
    pub ok: bool,
    pub message: String,
    /// When set, the job should not run again before this time (after a 429 or while the API is down).
    pub retry_at: Option<f64>,
}

/// The jobs and what they remember between runs.
/// `game` is the official game API, for data the community API doesn't carry (story campaigns).
/// `push` sends new Strohmann News (Major Orders, dispatches, campaign launches and outcomes) to the app.
pub struct Collector {
    pub api: Box<dyn Api>,
    pub game: Option<Box<dyn Api>>,
    pub db: Supabase,
    pub recorder: Recorder,
    pub push: Option<Box<dyn Push>>,
    pub now: Box<dyn Fn() -> f64>,
    last_static: String,
    last_dispatches: String,
    /// Major Orders and the newest dispatch already announced, or found stored after a restart. With nothing
    /// stored yet, whatever is there on the first run counts as announced, so a fresh setup pushes nothing.
    announced_orders: Option<HashSet<i64>>,
    announced_dispatch: Option<i64>,
    /// Latest campaigns, so each war sample can split players by front.
    last_campaigns: Vec<Campaign>,
    /// Each station's planet as last seen; read back from the stored snapshot after a restart.
    station_at: Option<HashMap<i64, i64>>,
    /// The last day of Major Order readings; read back from the stored snapshot after a restart.
    order_samples: Option<Vec<OrderSample>>,
    /// Campaigns as last written; read back once after a restart so the archive and issue dates survive it.
    episodes: Option<Vec<EpisodeLive>>,
    last_effects: String,
}

fn change_note(n: usize) -> String {
    match n {
        0 => String::new(),
        1 => ", 1 owner change".into(),
        n => format!(", {n} owner changes"),
    }
}

impl Collector {
    pub fn new(
        api: Box<dyn Api>,
        db: Supabase,
        recorder: Recorder,
        now: Box<dyn Fn() -> f64>,
        game: Option<Box<dyn Api>>,
        push: Option<Box<dyn Push>>,
    ) -> Collector {
        Collector {
            api,
            game,
            db,
            recorder,
            push,
            now,
            last_static: String::new(),
            last_dispatches: String::new(),
            announced_orders: None,
            announced_dispatch: None,
            last_campaigns: Vec::new(),
            station_at: None,
            order_samples: None,
            episodes: None,
            last_effects: String::new(),
        }
    }

    pub fn jobs(&self) -> Vec<Job> {
        let mut jobs = vec![
            // Full list first, so a fresh database learns every planet's owner on the first cycle.
            Job::Planets,
            Job::Campaigns,
            Job::War,
            Job::Assignments,
            Job::Dispatches,
            Job::Stations,
            Job::Compact,
        ];
        if self.game.is_some() {
            jobs.push(Job::Episodes);
        }
        jobs
    }

    pub fn run(&mut self, job: Job) -> RunResult {
        let result = match job {
            Job::Planets => self.planets(),
            Job::Campaigns => self.campaigns(),
            Job::War => self.war(),
            Job::Assignments => self.assignments(),
            Job::Dispatches => self.dispatches(),
            Job::Stations => self.stations(),
            Job::Compact => self.compact(),
            Job::Episodes => self.episodes(),
        };
        match result {
            Ok(message) => RunResult { name: job.name(), ok: true, message, retry_at: None },
            Err(e) => RunResult {
                name: job.name(),
                ok: false,
                message: e.to_string(),
                retry_at: e.retry_after_ms().map(|ms| (self.now)() + ms),
            },
        }
    }

    fn snapshot(&self, kind: SnapshotKind, data: Value) -> Result<(), String> {
        let row = json!({ "kind": kind.as_str(), "data": data, "fetched_at": iso_string((self.now)()) });
        self.db.upsert("snapshots", &[row], None, false)
    }

    /// The `data` of a stored snapshot, if there is one.
    fn stored(&self, kind: SnapshotKind) -> Result<Option<Value>, String> {
        let rows = self.db.select("snapshots", &format!("select=data&kind=eq.{}", kind.as_str()))?;
        Ok(rows.into_iter().next().and_then(|mut r| r.get_mut("data").map(Value::take)))
    }

    // Keep the wording in step with the app's own alerts (src/alerts/watch.rs). A failed push is only logged: the
    // snapshot is already written, so a retry wouldn't see the news as new.
    fn announce(&mut self, topic: Topic, items: &[(String, String)]) -> String {
        let Some(push) = self.push.as_mut() else { return String::new() };
        let mut sent = 0;
        for (title, body) in items {
            match push.push(topic, title, body) {
                Ok(()) => sent += 1,
                Err(e) => eprintln!("push failed: {e}"),
            }
        }
        if items.is_empty() { String::new() } else { format!(", {sent}/{} pushed", items.len()) }
    }

    fn planets(&mut self) -> Result<String, JobError> {
        let planets: Vec<Planet> = typed(self.api.get("/api/v1/planets")?)?;
        let at = (self.now)();
        let r = self.recorder.ingest(&self.db, &planets, at)?;
        self.snapshot(
            SnapshotKind::Planets,
            serde_json::to_value(planets.iter().map(to_live).collect::<Vec<_>>()).unwrap(),
        )?;
        let statics: Vec<_> = planets.iter().map(to_static).collect();
        let json = to_js_json(&statics);
        // Static data barely changes; only rewrite it when it does.
        if json != self.last_static {
            self.snapshot(SnapshotKind::PlanetsStatic, serde_json::to_value(&statics).unwrap())?;
            self.last_static = json;
        }
        Ok(format!("{} planets, {} samples{}", planets.len(), r.samples, change_note(r.changes.len())))
    }

    fn campaigns(&mut self) -> Result<String, JobError> {
        let campaigns: Vec<Campaign> = typed(self.api.get("/api/v1/campaigns")?)?;
        self.last_campaigns = campaigns.clone();
        let planets: Vec<Planet> = campaigns.iter().map(|c| c.planet.clone()).collect();
        let at = (self.now)();
        let r = self.recorder.ingest(&self.db, &planets, at)?;
        let live: Vec<_> = campaigns
            .iter()
            .map(|c| {
                to_campaign_live(c, self.recorder.rate(c.planet.index), |region| {
                    self.recorder.region_rate(c.planet.index, region)
                })
            })
            .collect();
        self.snapshot(SnapshotKind::Campaigns, serde_json::to_value(live).unwrap())?;
        Ok(format!("{} fronts, {} samples{}", campaigns.len(), r.samples, change_note(r.changes.len())))
    }

    fn war(&mut self) -> Result<String, JobError> {
        let raw = self.api.get("/api/v1/war")?;
        let war: War = typed(raw.clone())?;
        self.snapshot(SnapshotKind::War, raw)?;
        let sample = to_war_sample(&war, self.last_campaigns.iter().map(|c| &c.planet), (self.now)());
        self.db.upsert("war_samples", &[serde_json::to_value(sample).unwrap()], None, true)?;
        let online = war.statistics.player_count;
        // A reading of nobody online is an API blip, not the day's low.
        let noted = online > 0.0
            && unless_missing(
                self.db.rpc("note_war_day", json!({ "reading_at": iso_string((self.now)()), "online": online })),
            )?;
        let missing = if online > 0.0 && !noted { ", no war_days table yet" } else { "" };
        Ok(format!("{} divers{missing}", warroom_core::format::js_number(online)))
    }

    fn assignments(&mut self) -> Result<String, JobError> {
        let raw = self.api.get("/api/v1/assignments")?;
        let list: Vec<Assignment> = typed(raw.clone())?;
        if self.push.is_some() && self.announced_orders.is_none() {
            let stored: Vec<Assignment> = match self.stored(SnapshotKind::Assignments)? {
                Some(v) => typed(v)?,
                None => list.clone(),
            };
            self.announced_orders = Some(stored.iter().map(|x| x.id).collect());
        }
        self.snapshot(SnapshotKind::Assignments, raw)?;
        if self.order_samples.is_none() {
            let kept: Vec<OrderSample> =
                self.stored(SnapshotKind::OrderSamples)?.map(typed).transpose()?.unwrap_or_default();
            self.order_samples = Some(kept);
        }
        let samples = add_order_samples(self.order_samples.as_deref().unwrap_or_default(), &list, (self.now)());
        self.snapshot(SnapshotKind::OrderSamples, serde_json::to_value(&samples).unwrap())?;
        self.order_samples = Some(samples);
        let mut pushed = String::new();
        if let Some(announced) = self.announced_orders.clone() {
            let fresh: Vec<&Assignment> = list.iter().filter(|x| !announced.contains(&x.id)).collect();
            let items: Vec<(String, String)> = fresh
                .iter()
                .map(|x| {
                    let body = x.briefing.clone().or_else(|| x.title.clone());
                    (
                        "New Major Order".to_string(),
                        body.unwrap_or_else(|| "High Command has issued new orders.".into()),
                    )
                })
                .collect();
            pushed = self.announce(Topic::Orders, &items);
            let ids: Vec<i64> = fresh.iter().map(|x| x.id).collect();
            self.announced_orders.as_mut().unwrap().extend(ids);
        }
        Ok(format!("{} orders{pushed}", list.len()))
    }

    fn dispatches(&mut self) -> Result<String, JobError> {
        let raw = self.api.get("/api/v1/dispatches")?;
        let mut items: Vec<(Dispatch, Value)> = match raw {
            Value::Array(list) => {
                list.into_iter().map(|v| Ok((typed::<Dispatch>(v.clone())?, v))).collect::<Result<_, JobError>>()?
            }
            _ => Vec::new(),
        };
        items.sort_by_key(|(d, _)| std::cmp::Reverse(d.id));
        items.truncate(DISPATCH_LIMIT);
        if self.push.is_some() && self.announced_dispatch.is_none() {
            let stored: Vec<Dispatch> = match self.stored(SnapshotKind::Dispatches)? {
                Some(v) => typed(v)?,
                None => items.iter().map(|(d, _)| d.clone()).collect(),
            };
            self.announced_dispatch = Some(stored.iter().map(|x| x.id).fold(0, i64::max));
        }
        // Compared by text, not id: a dispatch stored in the wrong language is replaced once it comes in English.
        let data = Value::Array(items.iter().map(|(_, v)| v.clone()).collect());
        let json = to_js_json(&data);
        if json == self.last_dispatches {
            return Ok("unchanged".into());
        }
        self.snapshot(SnapshotKind::Dispatches, data)?;
        self.last_dispatches = json;
        let mut pushed = String::new();
        if let Some(since) = self.announced_dispatch {
            // The newest three, like the app, sent oldest first so the newest ends up on top.
            let mut fresh: Vec<(String, String)> = items
                .iter()
                .filter(|(d, _)| d.id > since)
                .take(3)
                .map(|(d, _)| ("New dispatch".to_string(), dispatch_title(&d.message)))
                .collect();
            fresh.reverse();
            pushed = self.announce(Topic::Dispatches, &fresh);
            self.announced_dispatch = Some(items.iter().map(|(d, _)| d.id).fold(since, i64::max));
        }
        Ok(format!("newest #{}{pushed}", items.first().map_or(0, |(d, _)| d.id)))
    }

    fn stations(&mut self) -> Result<String, JobError> {
        let raw = self.api.get("/api/v2/space-stations")?;
        let list: Vec<Value> = match raw {
            Value::Array(list) => list,
            _ => Vec::new(),
        };
        let int = |v: &Value, key: &str| v.get(key).and_then(Value::as_i64).unwrap_or(0);
        if self.station_at.is_none() {
            let stored: Vec<StationLive> =
                self.stored(SnapshotKind::SpaceStations)?.map(typed).transpose()?.unwrap_or_default();
            self.station_at = Some(stored.iter().map(|s| (s.id32, s.planet_index)).collect());
        }
        let at = iso_string((self.now)());
        let station_at = self.station_at.as_mut().unwrap();
        let jumps: Vec<Value> = list
            .iter()
            .filter_map(|s| {
                let (id, planet) = (int(s, "id32"), s.get("planet").map_or(0, |p| int(p, "index")));
                let from = *station_at.get(&id)?;
                (from != planet).then(|| json!({ "station": id, "at": at, "from_planet": from, "to_planet": planet }))
            })
            .collect();
        if !jumps.is_empty() {
            self.db.upsert("station_jumps", &jumps, None, true)?;
        }
        let live: Vec<Value> = list
            .iter()
            .map(|s| {
                json!({
                    "id32": s.get("id32"),
                    "planetIndex": s.get("planet").and_then(|p| p.get("index")),
                    "electionEnd": s.get("electionEnd"),
                    "flags": s.get("flags"),
                    "tacticalActions": s.get("tacticalActions"),
                })
            })
            .collect();
        self.snapshot(SnapshotKind::SpaceStations, Value::Array(live))?;
        let station_at = self.station_at.as_mut().unwrap();
        for s in &list {
            station_at.insert(int(s, "id32"), s.get("planet").map_or(0, |p| int(p, "index")));
        }
        let count = format!("{} station{}", list.len(), if list.len() == 1 { "" } else { "s" });
        Ok(match jumps.len() {
            0 => count,
            1 => format!("{count}, 1 FTL jump"),
            n => format!("{count}, {n} FTL jumps"),
        })
    }

    fn compact(&mut self) -> Result<String, JobError> {
        let removed = self.db.rpc("compact_samples", json!({}))?;
        let removed_war = self.db.rpc("compact_war_samples", json!({}))?;
        let mut removed_activity = json!(0);
        unless_missing(self.db.rpc("compact_activity", json!({})).map(|n| removed_activity = n))?;
        Ok(format!("{removed} old samples, {removed_war} war samples and {removed_activity} activity readings thinned"))
    }

    fn episodes(&mut self) -> Result<String, JobError> {
        let game = self.game.as_ref().expect("episodes run only with the game API");
        let id = game.get("/api/WarSeason/current/WarID")?.get("id").cloned().unwrap_or(Value::Null);
        let id = match &id {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        let raw: Vec<EpisodeRaw> = typed(
            game.get(&format!("/api/Episode/{id}"))?
                .get_mut("episodes")
                .map(Value::take)
                .unwrap_or(Value::Array(Vec::new())),
        )?;
        let status: WarStatusRaw = typed(game.get(&format!("/api/WarSeason/{id}/Status"))?)?;
        // Effects come from the status this job already reads, at no extra request.
        let effects = to_effects_live(&status, (self.now)());
        let effects_json = to_js_json(&effects);
        if effects_json != self.last_effects {
            self.snapshot(SnapshotKind::PlanetEffects, serde_json::to_value(&effects).unwrap())?;
            self.last_effects = effects_json;
        }
        if self.episodes.is_none() {
            let stored: Vec<EpisodeLive> =
                self.stored(SnapshotKind::Episodes)?.map(typed).transpose()?.unwrap_or_default();
            self.episodes = Some(stored);
        }
        let now = (self.now)();
        let before = self.episodes.clone().unwrap_or_default();
        let list = merge_episodes(&before, &to_episodes(&raw, status.time, now), now);
        let active = list.iter().filter(|e| e.status == EpisodeStatus::Active).count();
        if list == before {
            return Ok(format!("unchanged, {active} active"));
        }
        self.snapshot(SnapshotKind::Episodes, serde_json::to_value(&list).unwrap())?;
        let total = list.len();
        self.episodes = Some(list.clone());
        // An empty archive is a fresh setup, not a dozen campaigns launching at once.
        let pushed = if self.push.is_some() && !before.is_empty() {
            self.announce(Topic::Orders, &campaign_news(&before, &list))
        } else {
            String::new()
        };
        Ok(format!("{total} campaigns, {active} active{pushed}"))
    }
}

/// Campaigns launched or ended between two archives, worded as the app's campaign_headline() words them.
pub fn campaign_news(before: &[EpisodeLive], after: &[EpisodeLive]) -> Vec<(String, String)> {
    let was: HashMap<i64, &EpisodeLive> = before.iter().map(|e| (e.id, e)).collect();
    after
        .iter()
        .filter_map(|e| match was.get(&e.id) {
            None => Some(("New Galactic Campaign".into(), format!("New campaign: {}", e.title))),
            Some(old) if old.status == EpisodeStatus::Active && e.status != EpisodeStatus::Active => Some((
                "Galactic Campaign over".into(),
                format!("{} {}", e.title, if e.status == EpisodeStatus::Won { "won" } else { "failed" }),
            )),
            _ => None,
        })
        .collect()
}
