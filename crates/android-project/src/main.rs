//! Adds what the app needs to the Android project that `tauri android init` generates:
//!
//!   cargo run -q -p android-project -- firebase-values google-services.json \
//!     > src-tauri/gen/android/app/src/main/res/values/firebase.xml
//!   cargo run -q -p android-project -- manifest src-tauri/gen/android/app/src/main/AndroidManifest.xml

mod firebase;
mod manifest;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [command, path] if command == "firebase-values" => firebase::run(path).map(|xml| println!("{xml}")),
        [command, path] if command == "manifest" => manifest::run(path),
        _ => {
            Err("usage: android-project firebase-values <google-services.json> | manifest <AndroidManifest.xml>".into())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("android-project: {e}");
            ExitCode::FAILURE
        }
    }
}
