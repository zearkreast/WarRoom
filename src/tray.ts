import { invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect } from "react";
import { useSettings } from "./state";
import { isAndroid } from "./update/appUpdate";

export const hasTray = () => isTauri() && !isAndroid();

/** Keeps the desktop app's close-to-tray behaviour in step with Settings. */
export function useCloseToTray() {
  const closeToTray = useSettings((s) => s.closeToTray);
  useEffect(() => {
    if (hasTray()) void invoke("set_close_to_tray", { enabled: closeToTray });
  }, [closeToTray]);
}
