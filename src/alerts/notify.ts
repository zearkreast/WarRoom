import { isTauri } from "@tauri-apps/api/core";
import { create } from "zustand";

export type ToastKind = "order" | "dispatch" | "capture" | "loss" | "defense";

export interface Toast {
  id: number;
  kind: ToastKind;
  title: string;
  body: string;
}

interface ToastState {
  toasts: Toast[];
  push(t: Omit<Toast, "id">): void;
  dismiss(id: number): void;
}

let nextId = 1;
const TOAST_LIMIT = 5;

export const useToasts = create<ToastState>((set) => ({
  toasts: [],
  push: (t) => set((s) => ({ toasts: [{ ...t, id: nextId++ }, ...s.toasts].slice(0, TOAST_LIMIT) })),
  dismiss: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
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
export async function notify(kind: ToastKind, title: string, body: string, system: boolean) {
  useToasts.getState().push({ kind, title, body });
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
