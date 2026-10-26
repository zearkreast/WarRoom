// Turns google-services.json into the string resources Firebase starts from, as the google-services Gradle
// plugin would, so the project `tauri android init` regenerates needs no extra plugin.
//
//   node src-tauri/android/firebase-values.mjs google-services.json \
//     > src-tauri/gen/android/app/src/main/res/values/firebase.xml

import { readFileSync } from "node:fs";

const { identifier } = JSON.parse(readFileSync(new URL("../tauri.conf.json", import.meta.url), "utf8"));
const json = JSON.parse(readFileSync(process.argv[2], "utf8"));
const client = json.client.find((c) => c.client_info.android_client_info.package_name === identifier);
if (!client) throw new Error(`google-services.json has no Android app with the package ${identifier}`);

const values = {
  google_app_id: client.client_info.mobilesdk_app_id,
  gcm_defaultSenderId: json.project_info.project_number,
  google_api_key: client.api_key?.[0]?.current_key,
  project_id: json.project_info.project_id,
  google_storage_bucket: json.project_info.storage_bucket,
};
const escape = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;");
const lines = Object.entries(values)
  .filter(([, v]) => v)
  .map(([k, v]) => `    <string name="${k}" translatable="false">${escape(v)}</string>`);
console.log(`<?xml version="1.0" encoding="utf-8"?>\n<resources>\n${lines.join("\n")}\n</resources>`);
