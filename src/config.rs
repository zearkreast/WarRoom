pub const APP_NAME: &str = "WarCommand Tracker";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// The app only reads from Supabase; the collector (collector/) is what calls the Helldivers API.
// The publishable key is safe to ship: row-level security makes it read-only.
pub const SUPABASE_URL: &str = match option_env!("WARROOM_SUPABASE_URL") {
    Some(url) => url,
    None => "https://hdjqdwscppnaynjqjnal.supabase.co",
};
pub const SUPABASE_PUBLISHABLE_KEY: &str = match option_env!("WARROOM_SUPABASE_PUBLISHABLE_KEY") {
    Some(key) => key,
    None => "sb_publishable_Tnmt9AwgQDvjK5pnjv0WEQ_2BUlH1pV",
};

/// Android builds with Firebase get Strohmann News pushed by the collector, so the app only toasts it.
pub const NEWS_PUSHED: bool = match option_env!("WARROOM_NEWS_PUSH") {
    Some(v) => v.len() == 1 && v.as_bytes()[0] == b'1',
    None => false,
};

/// How often the app checks Supabase for newer snapshots and ownership changes.
pub const POLL_MS: f64 = 60_000.0;
