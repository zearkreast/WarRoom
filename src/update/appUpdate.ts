import { isTauri } from "@tauri-apps/api/core";
import type { Update } from "@tauri-apps/plugin-updater";
import { create } from "zustand";
import { APP_VERSION } from "../config";
import { isNewer } from "../domain/version";

const LATEST_RELEASE = "https://api.github.com/repos/zearkreast/WarRoom/releases/latest";
const RECHECK_MS = 6 * 3_600_000;

export type UpdatePhase =
  | { phase: "idle" }
  | { phase: "checking" }
  | { phase: "current" }
  | { phase: "available"; version: string; notes: string }
  | { phase: "downloading"; version: string; progress: number | null }
  | { phase: "installing"; version: string }
  | { phase: "handed-off"; version: string }
  | { phase: "error"; version?: string; message: string };

interface UpdateState {
  state: UpdatePhase;
  /** Hidden until the next launch or a manual check. */
  dismissed: boolean;
  dismiss(): void;
}

export const useUpdate = create<UpdateState>((set) => ({
  state: { phase: "idle" },
  dismissed: false,
  dismiss: () => set({ dismissed: true }),
}));

const setState = (state: UpdatePhase) => useUpdate.setState({ state });

/** The Android build has no updater: its update is the release's APK, installed by the system. */
export const isAndroid = () => isTauri() && /android/i.test(navigator.userAgent);

let pending: Update | null = null;
let apkUrl: string | null = null;

/** Looks for a newer release; the web build has nothing to install, so it never looks. */
export async function checkForUpdate(manual = false) {
  if (!isTauri()) return;
  if (manual) useUpdate.setState({ dismissed: false });
  setState({ phase: "checking" });
  try {
    if (isAndroid()) {
      const res = await fetch(LATEST_RELEASE, { headers: { Accept: "application/vnd.github+json" } });
      if (!res.ok) throw new Error(`GitHub answered ${res.status}`);
      const release: { tag_name: string; body?: string; assets: { name: string; browser_download_url: string }[] } =
        await res.json();
      const apk = release.assets.find((a) => a.name.endsWith("_android.apk"));
      if (!apk || !isNewer(release.tag_name, APP_VERSION)) return setState({ phase: "current" });
      apkUrl = apk.browser_download_url;
      return setState({ phase: "available", version: release.tag_name.replace(/^v/i, ""), notes: release.body ?? "" });
    }
    const { check } = await import("@tauri-apps/plugin-updater");
    pending = await check();
    if (!pending) return setState({ phase: "current" });
    setState({ phase: "available", version: pending.version, notes: pending.body ?? "" });
  } catch (e) {
    // A check that fails quietly at start-up is better than an error card on every offline launch.
    setState(manual ? { phase: "error", message: String(e) } : { phase: "idle" });
  }
}

/** Downloads and installs the update found by the last check, then restarts into it. */
export async function installUpdate() {
  const s = useUpdate.getState().state;
  if (s.phase !== "available" && s.phase !== "error") return;
  const version = s.phase === "available" ? s.version : (s.version ?? "");
  try {
    if (isAndroid()) {
      if (!apkUrl) return;
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(apkUrl);
      return setState({ phase: "handed-off", version });
    }
    if (!pending) return;
    let total = 0;
    let received = 0;
    setState({ phase: "downloading", version, progress: null });
    await pending.downloadAndInstall((e) => {
      if (e.event === "Started") total = e.data.contentLength ?? 0;
      else if (e.event === "Progress") {
        received += e.data.chunkLength;
        setState({ phase: "downloading", version, progress: total ? received / total : null });
      } else setState({ phase: "installing", version });
    });
    const { relaunch } = await import("@tauri-apps/plugin-process");
    await relaunch();
  } catch (e) {
    setState({ phase: "error", version, message: String(e) });
  }
}

/** Checks shortly after start-up, then every few hours while the app stays open. */
export function startUpdateChecks(): () => void {
  if (!isTauri()) return () => {};
  const first = window.setTimeout(() => void checkForUpdate(), 8_000);
  const again = window.setInterval(() => {
    const phase = useUpdate.getState().state.phase;
    if (phase === "idle" || phase === "current") void checkForUpdate();
  }, RECHECK_MS);
  return () => {
    window.clearTimeout(first);
    window.clearInterval(again);
  };
}
