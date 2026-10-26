// Adds what the in-app APK update needs to the generated AndroidManifest.xml: permission to open Android's
// installer, and the provider that hands it the downloaded file. Running it twice changes nothing.
// usage: node manifest.mjs <AndroidManifest.xml>

import { readFileSync, writeFileSync } from "node:fs";

const path = process.argv[2];
let xml = readFileSync(path, "utf8");

const PERMISSION = '<uses-permission android:name="android.permission.REQUEST_INSTALL_PACKAGES" />';
const PROVIDER = `<provider
            android:name=".ApkFileProvider"
            android:authorities="\${applicationId}.apkupdate"
            android:exported="false"
            android:grantUriPermissions="true">
            <meta-data
                android:name="android.support.FILE_PROVIDER_PATHS"
                android:resource="@xml/apk_update_paths" />
        </provider>`;

function insert(before, text) {
  const at = xml.indexOf(before);
  if (at < 0) throw new Error(`${path} has no ${before}`);
  xml = `${xml.slice(0, at)}${text}\n    ${xml.slice(at)}`;
}

if (!xml.includes("REQUEST_INSTALL_PACKAGES")) insert("<application", PERMISSION);
if (!xml.includes(".ApkFileProvider")) insert("</application>", `    ${PROVIDER}`);
writeFileSync(path, xml);
