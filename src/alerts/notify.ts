import { isTauri } from "@tauri-apps/api/core";
import { create } from "zustand";

export type ToastKind = "order" | "dispatch" | "capture" | "loss" | "defense";

export interface Toast {
  id: number;
  kind: ToastKind;
  title: string;
  body: string;
  /** Full game text (dispatch or Major Order briefing) for the briefing popup. */
  message?: string;
}

interface ToastState {
  toasts: Toast[];
  /** The toast whose briefing popup is open. */
  briefing: Toast | null;
  push(t: Omit<Toast, "id">): void;
  dismiss(id: number): void;
  /** Swaps the toast for its briefing popup. */
  open(id: number): void;
  closeBriefing(): void;
}

let nextId = 1;
const TOAST_LIMIT = 5;

export const useToasts = create<ToastState>((set) => ({
  toasts: [],
  briefing: null,
  push: (t) => set((s) => ({ toasts: [{ ...t, id: nextId++ }, ...s.toasts].slice(0, TOAST_LIMIT) })),
  dismiss: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
  open: (id) =>
    set((s) => ({ briefing: s.toasts.find((t) => t.id === id) ?? s.briefing, toasts: s.toasts.filter((t) => t.id !== id) })),
  closeBriefing: () => set({ briefing: null }),
}));

let permission: Promise<boolean> | null = null;

async function ensurePermission(): Promise<boolean> {
  if (isTauri()) {
    const n = await import("@tauri-apps/plugin-notification");
    if (await n.isPermissionGranted()) return true;
    return (await n.requestPermission()) === "granted";
  }
  if (typeof Notification === "undefined") return false;
  if (Notification.permission === "granted") return true;
  return (await Notification.requestPermission()) === "granted";
}

/** In-app toast always; an OS notification too when `system` is on and permission is granted. */
export async function notify(kind: ToastKind, title: string, body: string, system: boolean, message?: string) {
  useToasts.getState().push({ kind, title, body, message });
  if (!system) return;
  permission ??= ensurePermission().catch(() => false);
  if (!(await permission)) return;
  if (isTauri()) {
    const n = await import("@tauri-apps/plugin-notification");
    n.sendNotification({ title, body });
  } else {
    new Notification(title, { body });
  }
}
