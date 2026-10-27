//! Desktop entry point; Android starts from `run` in lib.rs.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    warcommand_tracker_lib::run()
}
