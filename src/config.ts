export const APP_NAME = "WarCommand Tracker";
export const APP_VERSION = "0.3.0-beta";

// The app only reads from Supabase; the collector (collector/) is what calls the Helldivers API.
// The publishable key is safe to ship: row-level security makes it read-only.
export const SUPABASE_URL: string = import.meta.env?.VITE_SUPABASE_URL || "https://hdjqdwscppnaynjqjnal.supabase.co";
export const SUPABASE_PUBLISHABLE_KEY: string =
  import.meta.env?.VITE_SUPABASE_PUBLISHABLE_KEY || "sb_publishable_Tnmt9AwgQDvjK5pnjv0WEQ_2BUlH1pV";

/** How often the app checks Supabase for newer snapshots and ownership changes. */
export const POLL_MS = 60_000;
