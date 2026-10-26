//! What the in-app APK update needs in AndroidManifest.xml: permission to open Android's installer, and the
//! provider that hands it the downloaded file. Keep the names in sync with ApkUpdate and ApkFileProvider in
//! src-tauri/android/MainActivity.kt.template and with res/xml/apk_update_paths.xml.

const PERMISSION: &str = r#"<uses-permission android:name="android.permission.REQUEST_INSTALL_PACKAGES" />"#;
const PROVIDER: &str = r#"<provider
            android:name=".ApkFileProvider"
            android:authorities="${applicationId}.apkupdate"
            android:exported="false"
            android:grantUriPermissions="true">
            <meta-data
                android:name="android.support.FILE_PROVIDER_PATHS"
                android:resource="@xml/apk_update_paths" />
        </provider>"#;

/// Adds the entries to the manifest at `path`; running it twice changes nothing.
pub fn run(path: &str) -> Result<(), String> {
    let xml = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let xml = add_entries(&xml).map_err(|missing| format!("{path} has no {missing}"))?;
    std::fs::write(path, xml).map_err(|e| format!("{path}: {e}"))
}

fn add_entries(xml: &str) -> Result<String, &'static str> {
    let mut xml = xml.to_string();
    if !xml.contains("REQUEST_INSTALL_PACKAGES") {
        insert(&mut xml, "<application", PERMISSION)?;
    }
    if !xml.contains(".ApkFileProvider") {
        insert(&mut xml, "</application>", &format!("    {PROVIDER}"))?;
    }
    Ok(xml)
}

fn insert(xml: &mut String, before: &'static str, text: &str) -> Result<(), &'static str> {
    let at = xml.find(before).ok_or(before)?;
    xml.insert_str(at, &format!("{text}\n    "));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENERATED: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <uses-permission android:name="android.permission.INTERNET" />

    <application
        android:icon="@mipmap/ic_launcher"
        android:label="@string/app_name">
        <activity android:name=".MainActivity" android:exported="true">
        </activity>
    </application>
</manifest>
"#;

    #[test]
    fn adds_the_install_permission_and_the_apk_provider() {
        assert_eq!(
            add_entries(GENERATED).unwrap(),
            r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <uses-permission android:name="android.permission.INTERNET" />

    <uses-permission android:name="android.permission.REQUEST_INSTALL_PACKAGES" />
    <application
        android:icon="@mipmap/ic_launcher"
        android:label="@string/app_name">
        <activity android:name=".MainActivity" android:exported="true">
        </activity>
        <provider
            android:name=".ApkFileProvider"
            android:authorities="${applicationId}.apkupdate"
            android:exported="false"
            android:grantUriPermissions="true">
            <meta-data
                android:name="android.support.FILE_PROVIDER_PATHS"
                android:resource="@xml/apk_update_paths" />
        </provider>
    </application>
</manifest>
"#
        );
    }

    #[test]
    fn changes_nothing_the_second_time() {
        let once = add_entries(GENERATED).unwrap();
        assert_eq!(add_entries(&once).unwrap(), once);
    }

    #[test]
    fn fails_without_an_application_element() {
        assert_eq!(add_entries("<manifest></manifest>"), Err("<application"));
    }
}
