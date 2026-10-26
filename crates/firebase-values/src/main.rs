//! Turns google-services.json into the string resources Firebase starts from, as the google-services Gradle
//! plugin would, so the project `tauri android init` regenerates needs no extra plugin.
//!
//!   cargo run -q -p firebase-values -- google-services.json \
//!     > src-tauri/gen/android/app/src/main/res/values/firebase.xml

use serde_json::Value;
use std::process::ExitCode;

const TAURI_CONF: &str = include_str!("../../../src-tauri/tauri.conf.json");

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: firebase-values <google-services.json>");
        return ExitCode::FAILURE;
    };
    match identifier().and_then(|id| resources(&read(&path)?, &id)) {
        Ok(xml) => {
            println!("{xml}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("firebase-values: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The app's Android package.
fn identifier() -> Result<String, String> {
    let conf: Value = serde_json::from_str(TAURI_CONF).map_err(|e| format!("tauri.conf.json: {e}"))?;
    conf["identifier"].as_str().map(str::to_string).ok_or_else(|| "tauri.conf.json has no identifier".to_string())
}

fn read(path: &str) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))
}

/// `values/firebase.xml` for the app with the package `identifier`.
fn resources(services: &Value, identifier: &str) -> Result<String, String> {
    let client = services["client"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|c| c["client_info"]["android_client_info"]["package_name"] == identifier)
        .ok_or_else(|| format!("google-services.json has no Android app with the package {identifier}"))?;
    let project = &services["project_info"];
    let values = [
        ("google_app_id", &client["client_info"]["mobilesdk_app_id"]),
        ("gcm_defaultSenderId", &project["project_number"]),
        ("google_api_key", &client["api_key"][0]["current_key"]),
        ("project_id", &project["project_id"]),
        ("google_storage_bucket", &project["storage_bucket"]),
    ];
    let lines: Vec<String> = values
        .into_iter()
        .filter_map(|(name, value)| {
            let text = match value {
                Value::String(s) if !s.is_empty() => s.clone(),
                Value::Number(n) if n.as_f64() != Some(0.0) => n.to_string(),
                _ => return None,
            };
            Some(format!(r#"    <string name="{name}" translatable="false">{}</string>"#, escape(&text)))
        })
        .collect();
    Ok(format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<resources>\n{}\n</resources>", lines.join("\n")))
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn services() -> Value {
        json!({
            "project_info": {
                "project_number": "123456789012",
                "project_id": "warroom-test",
                "storage_bucket": "warroom-test.firebasestorage.app"
            },
            "client": [
                {
                    "client_info": {
                        "mobilesdk_app_id": "1:123456789012:android:0ther",
                        "android_client_info": { "package_name": "com.example.other" }
                    },
                    "api_key": [{ "current_key": "other-key" }]
                },
                {
                    "client_info": {
                        "mobilesdk_app_id": "1:123456789012:android:abc123",
                        "android_client_info": { "package_name": "com.warcommand.tracker" }
                    },
                    "api_key": [{ "current_key": "AIza<key&more" }]
                }
            ]
        })
    }

    #[test]
    fn writes_the_values_of_the_apps_own_client() {
        assert_eq!(
            resources(&services(), "com.warcommand.tracker").unwrap(),
            r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="google_app_id" translatable="false">1:123456789012:android:abc123</string>
    <string name="gcm_defaultSenderId" translatable="false">123456789012</string>
    <string name="google_api_key" translatable="false">AIza&lt;key&amp;more</string>
    <string name="project_id" translatable="false">warroom-test</string>
    <string name="google_storage_bucket" translatable="false">warroom-test.firebasestorage.app</string>
</resources>"#
        );
    }

    #[test]
    fn leaves_out_values_the_file_lacks() {
        let mut s = services();
        s["project_info"]["storage_bucket"] = Value::Null;
        s["client"][1]["api_key"] = json!([]);
        s["project_info"]["project_number"] = json!(123456789012u64);
        let xml = resources(&s, "com.warcommand.tracker").unwrap();
        assert!(!xml.contains("google_storage_bucket") && !xml.contains("google_api_key"));
        assert!(xml.contains(r#"<string name="gcm_defaultSenderId" translatable="false">123456789012</string>"#));
    }

    #[test]
    fn fails_without_a_client_for_the_package() {
        let err = resources(&services(), "com.example.missing").unwrap_err();
        assert!(err.contains("com.example.missing"));
    }

    #[test]
    fn reads_the_package_from_the_tauri_config() {
        assert!(identifier().is_ok_and(|id| !id.is_empty()));
    }
}
