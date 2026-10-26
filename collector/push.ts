// Firebase Cloud Messaging: sends Strohmann News to every Android app subscribed to the news topic.

import { createSign } from "node:crypto";

/** Keep in sync with NEWS_TOPIC and NEWS_CHANNEL in src-tauri/android/MainActivity.kt.template. */
const TOPIC = "news";
const CHANNEL = "news";
/** FCM refuses messages over 4 KB; a Major Order briefing can come close. */
const MAX_BODY = 1000;
const TOKEN_URL = "https://oauth2.googleapis.com/token";

export interface ServiceAccount {
  project_id: string;
  client_email: string;
  private_key: string;
}

export type Push = (title: string, body: string) => Promise<void>;

type FetchFn = (url: string, init: RequestInit) => Promise<Response>;

/** The service account key as base64 (`base64 -w0 key.json`), or as JSON on one line. */
export function parseServiceAccount(value: string): ServiceAccount {
  const json = value.startsWith("{") ? value : Buffer.from(value, "base64").toString("utf8");
  const account = JSON.parse(json) as ServiceAccount;
  if (!account.project_id || !account.client_email || !account.private_key)
    throw new Error("FCM_SERVICE_ACCOUNT needs project_id, client_email and private_key");
  return account;
}

const b64url = (s: string) => Buffer.from(s).toString("base64url");

export function createPush(account: ServiceAccount, fetchFn: FetchFn = fetch, now = Date.now): Push {
  let token = "";
  let expires = 0;

  // A short-lived OAuth token from a JWT the service account signs; reused until a minute before it lapses.
  async function accessToken(): Promise<string> {
    if (now() < expires - 60_000) return token;
    const iat = Math.floor(now() / 1000);
    const unsigned = `${b64url(JSON.stringify({ alg: "RS256", typ: "JWT" }))}.${b64url(
      JSON.stringify({
        iss: account.client_email,
        scope: "https://www.googleapis.com/auth/firebase.messaging",
        aud: TOKEN_URL,
        iat,
        exp: iat + 3600,
      }),
    )}`;
    const signature = createSign("RSA-SHA256").update(unsigned).sign(account.private_key, "base64url");
    const res = await fetchFn(TOKEN_URL, {
      method: "POST",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({
        grant_type: "urn:ietf:params:oauth:grant-type:jwt-bearer",
        assertion: `${unsigned}.${signature}`,
      }).toString(),
    });
    if (!res.ok) throw new Error(`FCM token ${res.status}: ${await res.text()}`);
    const got = (await res.json()) as { access_token: string; expires_in: number };
    token = got.access_token;
    expires = now() + got.expires_in * 1000;
    return token;
  }

  return async (title, body) => {
    const res = await fetchFn(`https://fcm.googleapis.com/v1/projects/${account.project_id}/messages:send`, {
      method: "POST",
      headers: { Authorization: `Bearer ${await accessToken()}`, "Content-Type": "application/json" },
      body: JSON.stringify({
        message: {
          topic: TOPIC,
          notification: { title, body: body.length > MAX_BODY ? `${body.slice(0, MAX_BODY - 1)}…` : body },
          android: {
            priority: "high",
            notification: { channel_id: CHANNEL, icon: "ic_notification", color: "#ffff5a" },
          },
        },
      }),
    });
    if (!res.ok) throw new Error(`FCM send ${res.status}: ${await res.text()}`);
  };
}
