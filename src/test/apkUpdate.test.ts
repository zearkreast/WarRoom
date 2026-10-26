import { afterEach, describe, expect, it, vi } from "vitest";

const APK = "https://example.test/WarCommand.Tracker_99.0.0_android.apk";

describe("Android APK update", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("downloads through the app's bridge, shows progress, then offers the installer", async () => {
    const calls: string[] = [];
    const win: Record<string, unknown> = {
      WarRoomApk: { download: (url: string) => calls.push(`download ${url}`), install: () => calls.push("install") },
    };
    vi.stubGlobal("isTauri", true);
    vi.stubGlobal("navigator", { userAgent: "Mozilla/5.0 (Linux; Android 14; Pixel 8)" });
    vi.stubGlobal("window", win);
    vi.stubGlobal("fetch", async () => ({
      ok: true,
      json: async () => ({
        tag_name: "v99.0.0",
        body: "",
        assets: [{ name: "WarCommand.Tracker_99.0.0_android.apk", browser_download_url: APK }],
      }),
    }));
    const { checkForUpdate, installUpdate, openInstaller, useUpdate } = await import("../update/appUpdate");

    await checkForUpdate(true);
    expect(useUpdate.getState().state).toMatchObject({ phase: "available", version: "99.0.0" });

    await installUpdate();
    expect(calls).toEqual([`download ${APK}`]);
    expect(useUpdate.getState().state).toMatchObject({ phase: "downloading", progress: null });

    const send = win.__warroomApk as (e: unknown) => void;
    send({ type: "progress", received: 25, total: 100 });
    expect(useUpdate.getState().state).toMatchObject({ phase: "downloading", progress: 0.25 });
    send({ type: "progress", received: 25, total: -1 });
    expect(useUpdate.getState().state).toMatchObject({ phase: "downloading", progress: null });

    send({ type: "done" });
    expect(useUpdate.getState().state).toEqual({ phase: "downloaded", version: "99.0.0" });
    openInstaller();
    expect(calls).toEqual([`download ${APK}`, "install"]);

    send({ type: "error", message: "The download is gone; download it again." });
    expect(useUpdate.getState().state).toMatchObject({ phase: "error", version: "99.0.0" });
  });
});
