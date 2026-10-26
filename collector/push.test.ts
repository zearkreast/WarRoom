import { createVerify, generateKeyPairSync } from "node:crypto";
import { describe, expect, it } from "vitest";
import { createPush, parseServiceAccount } from "./push.ts";

const { privateKey, publicKey } = generateKeyPairSync("rsa", { modulusLength: 2048 });
const account = {
  project_id: "warroom",
  client_email: "push@warroom.iam.gserviceaccount.com",
  private_key: privateKey.export({ type: "pkcs8", format: "pem" }).toString(),
};

/** Google's token and FCM endpoints, recording each request. */
function fakeGoogle(sendStatus = 200) {
  const calls: { url: string; body: string; auth: string | null }[] = [];
  const fetchFn = async (url: string, init: RequestInit) => {
    calls.push({ url, body: String(init.body), auth: new Headers(init.headers).get("Authorization") });
    if (url.includes("oauth2")) return Response.json({ access_token: `token${calls.length}`, expires_in: 3600 });
    return new Response(sendStatus === 200 ? '{"name":"m1"}' : "denied", { status: sendStatus });
  };
  return { calls, fetchFn };
}

describe("FCM pushes", () => {
  it("signs a token request with the service account, then sends to the news topic", async () => {
    const { calls, fetchFn } = fakeGoogle();
    await createPush(account, fetchFn, () => 1_000_000)("New dispatch", "Helldivers, stand by");
    const [token, send] = calls;

    expect(token.url).toBe("https://oauth2.googleapis.com/token");
    const [head, claims, signature] = new URLSearchParams(token.body).get("assertion")!.split(".");
    expect(createVerify("RSA-SHA256").update(`${head}.${claims}`).verify(publicKey, signature, "base64url")).toBe(true);
    expect(JSON.parse(Buffer.from(claims, "base64url").toString())).toEqual({
      iss: account.client_email,
      scope: "https://www.googleapis.com/auth/firebase.messaging",
      aud: "https://oauth2.googleapis.com/token",
      iat: 1000,
      exp: 4600,
    });

    expect(send.url).toBe("https://fcm.googleapis.com/v1/projects/warroom/messages:send");
    expect(send.auth).toBe("Bearer token1");
    expect(JSON.parse(send.body).message).toEqual({
      topic: "news",
      notification: { title: "New dispatch", body: "Helldivers, stand by" },
      android: { priority: "high", notification: { channel_id: "news", icon: "ic_notification", color: "#ffff5a" } },
    });
  });

  it("reuses the token until a minute before it lapses", async () => {
    let t = 0;
    const { calls, fetchFn } = fakeGoogle();
    const push = createPush(account, fetchFn, () => t);
    await push("a", "b");
    t = 3_500_000;
    await push("a", "b");
    t = 3_560_000;
    await push("a", "b");
    expect(calls.filter((c) => c.url.includes("oauth2"))).toHaveLength(2);
    expect(calls.filter((c) => c.url.includes("fcm")).map((c) => c.auth)).toEqual([
      "Bearer token1",
      "Bearer token1",
      "Bearer token4",
    ]);
  });

  it("cuts a long briefing down to what FCM accepts", async () => {
    const { calls, fetchFn } = fakeGoogle();
    await createPush(account, fetchFn)("New Major Order", "x".repeat(5000));
    const body = JSON.parse(calls[1].body).message.notification.body as string;
    expect(body).toHaveLength(1000);
    expect(body.endsWith("…")).toBe(true);
  });

  it("fails when FCM refuses the message", async () => {
    const { fetchFn } = fakeGoogle(403);
    await expect(createPush(account, fetchFn)("a", "b")).rejects.toThrow("FCM send 403: denied");
  });

  it("reads the key as base64 or as JSON on one line", () => {
    const json = JSON.stringify(account);
    expect(parseServiceAccount(Buffer.from(json).toString("base64"))).toEqual(account);
    expect(parseServiceAccount(json)).toEqual(account);
    expect(() => parseServiceAccount("{}")).toThrow("FCM_SERVICE_ACCOUNT needs");
  });
});
