//! Lucide icons (https://lucide.dev, ISC licence), with the markup and class names of Lucide 1.61's components.

use crate::js::num;
use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconData {
    /// The `class` lucide gives the icon, "lucide lucide-x".
    pub class: &'static str,
    /// The icon's shapes, inside a 24-unit view box.
    pub body: &'static str,
}

pub const ACTIVITY: IconData = IconData {
    class: "lucide lucide-activity",
    body: r#"<path d="M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2"></path>"#,
};
pub const ARCHIVE: IconData = IconData {
    class: "lucide lucide-archive",
    body: r#"<rect width="20" height="5" x="2" y="3" rx="1"></rect><path d="M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8"></path><path d="M10 12h4"></path>"#,
};
pub const ARROW_DOWN_WIDE_NARROW: IconData = IconData {
    class: "lucide lucide-arrow-down-wide-narrow lucide-sort-desc",
    body: r#"<path d="m3 16 4 4 4-4"></path><path d="M7 20V4"></path><path d="M11 4h10"></path><path d="M11 8h7"></path><path d="M11 12h4"></path>"#,
};
pub const ARROW_LEFT: IconData = IconData {
    class: "lucide lucide-arrow-left",
    body: r#"<path d="m12 19-7-7 7-7"></path><path d="M19 12H5"></path>"#,
};
pub const ARROW_RIGHT: IconData = IconData {
    class: "lucide lucide-arrow-right",
    body: r#"<path d="M5 12h14"></path><path d="m12 5 7 7-7 7"></path>"#,
};
pub const ARROW_UP_RIGHT: IconData = IconData {
    class: "lucide lucide-arrow-up-right",
    body: r#"<path d="M7 7h10v10"></path><path d="M7 17 17 7"></path>"#,
};
pub const AWARD: IconData = IconData {
    class: "lucide lucide-award",
    body: r#"<path d="m15.477 12.89 1.515 8.526a.5.5 0 0 1-.81.47l-3.58-2.687a1 1 0 0 0-1.197 0l-3.586 2.686a.5.5 0 0 1-.81-.469l1.514-8.526"></path><circle cx="12" cy="8" r="6"></circle>"#,
};
pub const BELL_RING: IconData = IconData {
    class: "lucide lucide-bell-ring",
    body: r#"<path d="M10.268 21a2 2 0 0 0 3.464 0"></path><path d="M22 8c0-2.3-.8-4.3-2-6"></path><path d="M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326"></path><path d="M4 2C2.8 3.7 2 5.7 2 8"></path>"#,
};
pub const BOOK_OPEN: IconData = IconData {
    class: "lucide lucide-book-open",
    body: r#"<path d="M12 5v16"></path><path d="M20.001 19A2 2 0 0022 17V5a2 2 0 00-1.999-2L16 3.002A5 5 0 0012 5a5 5 0 00-4-2H4a2 2 0 00-2 2v12a2 2 0 001.999 2H8a5 5 0 014 2 5 5 0 014-2z"></path>"#,
};
pub const BUILDING_2: IconData = IconData {
    class: "lucide lucide-building-complex lucide-building-2",
    body: r#"<path d="M10 12h4"></path><path d="M10 8h4"></path><path d="M14 21v-3a2 2 0 0 0-4 0v3"></path><path d="M6 10H4a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-2"></path><path d="M6 21V5a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v16"></path>"#,
};
pub const CALENDAR_CLOCK: IconData = IconData {
    class: "lucide lucide-calendar-clock",
    body: r#"<path d="M16 14v2.2l1.6 1"></path><path d="M16 2v3"></path><path d="M21 7.338V5a2 2 0 00-2-2H5a2 2 0 00-2 2v14a2 2 0 002 2h2.338"></path><path d="M3 9h5.859"></path><path d="M8 2v3"></path><circle cx="16" cy="16" r="6"></circle>"#,
};
pub const CALENDAR_DAYS: IconData = IconData {
    class: "lucide lucide-calendar-days",
    body: r#"<path d="M8 2v3"></path><path d="M16 2v3"></path><rect x="3" y="3" width="18" height="18" rx="2"></rect><path d="M3 9h18"></path><path d="M8 13h.01"></path><path d="M12 13h.01"></path><path d="M16 13h.01"></path><path d="M8 17h.01"></path><path d="M12 17h.01"></path><path d="M16 17h.01"></path>"#,
};
pub const CHECK: IconData = IconData { class: "lucide lucide-check", body: r#"<path d="M20 6 9 17l-5-5"></path>"# };
pub const CHECK_SQUARE: IconData = IconData {
    class: "lucide lucide-square-check-big lucide-check-square",
    body: r#"<path d="M21 10.656V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h12.344"></path><path d="m9 11 3 3L22 4"></path>"#,
};
pub const CHEVRON_DOWN: IconData =
    IconData { class: "lucide lucide-chevron-down", body: r#"<path d="m6 9 6 6 6-6"></path>"# };
pub const CHEVRON_LEFT: IconData =
    IconData { class: "lucide lucide-chevron-left", body: r#"<path d="m15 18-6-6 6-6"></path>"# };
pub const CHEVRON_RIGHT: IconData =
    IconData { class: "lucide lucide-chevron-right", body: r#"<path d="m9 18 6-6-6-6"></path>"# };
pub const CIRCLE_HELP: IconData = IconData {
    class: "lucide lucide-circle-question-mark lucide-help-circle lucide-circle-help",
    body: r#"<circle cx="12" cy="12" r="10"></circle><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path><path d="M12 17h.01"></path>"#,
};
pub const CLIPBOARD_COPY: IconData = IconData {
    class: "lucide lucide-clipboard-copy",
    body: r#"<rect width="8" height="4" x="8" y="2" rx="1" ry="1"></rect><path d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"></path><path d="M16 4h2a2 2 0 0 1 2 2v4"></path><path d="M21 14H11"></path><path d="m15 10-4 4 4 4"></path>"#,
};
pub const CLOUD_FOG: IconData = IconData {
    class: "lucide lucide-cloud-fog",
    body: r#"<path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"></path><path d="M16 17H7"></path><path d="M17 21H9"></path>"#,
};
pub const CLOUD_LIGHTNING: IconData = IconData {
    class: "lucide lucide-cloud-lightning",
    body: r#"<path d="M6 16.326A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 .5 8.973"></path><path d="m13 12-3 5h4l-3 5"></path>"#,
};
pub const CLOUD_RAIN: IconData = IconData {
    class: "lucide lucide-cloud-rain",
    body: r#"<path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"></path><path d="M16 14v6"></path><path d="M8 14v6"></path><path d="M12 16v6"></path>"#,
};
pub const CLOUD_SUN: IconData = IconData {
    class: "lucide lucide-cloud-sun",
    body: r#"<path d="M12 2v2"></path><path d="m4.93 4.93 1.41 1.41"></path><path d="M20 12h2"></path><path d="m19.07 4.93-1.41 1.41"></path><path d="M15.947 12.65a4 4 0 0 0-5.925-4.128"></path><path d="M13 22H7a5 5 0 1 1 4.9-6H13a3 3 0 0 1 0 6Z"></path>"#,
};
pub const COMPASS: IconData = IconData {
    class: "lucide lucide-compass",
    body: r#"<circle cx="12" cy="12" r="10"></circle><path d="m16.24 7.76-1.804 5.411a2 2 0 0 1-1.265 1.265L7.76 16.24l1.804-5.411a2 2 0 0 1 1.265-1.265z"></path>"#,
};
pub const CROSSHAIR: IconData = IconData {
    class: "lucide lucide-crosshair",
    body: r#"<circle cx="12" cy="12" r="10"></circle><line x1="22" x2="18" y1="12" y2="12"></line><line x1="6" x2="2" y1="12" y2="12"></line><line x1="12" x2="12" y1="6" y2="2"></line><line x1="12" x2="12" y1="22" y2="18"></line>"#,
};
pub const DATABASE: IconData = IconData {
    class: "lucide lucide-database",
    body: r#"<ellipse cx="12" cy="5" rx="9" ry="3"></ellipse><path d="M3 5V19A9 3 0 0 0 21 19V5"></path><path d="M3 12A9 3 0 0 0 21 12"></path>"#,
};
pub const DOWNLOAD: IconData = IconData {
    class: "lucide lucide-download",
    body: r#"<path d="M12 15V3"></path><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><path d="m7 10 5 5 5-5"></path>"#,
};
pub const DROPLETS: IconData = IconData {
    class: "lucide lucide-droplets",
    body: r#"<path d="M7 16.3c2.2 0 4-1.83 4-4.05 0-1.16-.57-2.26-1.71-3.19S7.29 6.75 7 5.3c-.29 1.45-1.14 2.84-2.29 3.76S3 11.1 3 12.25c0 2.22 1.8 4.05 4 4.05z"></path><path d="M12.56 6.6A10.97 10.97 0 0 0 14 3.02c.5 2.5 2 4.9 4 6.5s3 3.5 3 5.5a6.98 6.98 0 0 1-11.91 4.97"></path>"#,
};
pub const EARTH: IconData = IconData {
    class: "lucide lucide-earth lucide-globe-2",
    body: r#"<path d="M21.54 15H17a2 2 0 0 0-2 2v4.54"></path><path d="M7 3.34V5a3 3 0 0 0 3 3a2 2 0 0 1 2 2c0 1.1.9 2 2 2a2 2 0 0 0 2-2c0-1.1.9-2 2-2h3.17"></path><path d="M11 21.95V18a2 2 0 0 0-2-2a2 2 0 0 1-2-2v-1a2 2 0 0 0-2-2H2.05"></path><circle cx="12" cy="12" r="10"></circle>"#,
};
pub const ELLIPSIS: IconData = IconData {
    class: "lucide lucide-ellipsis",
    body: r#"<circle cx="12" cy="12" r="1"></circle><circle cx="19" cy="12" r="1"></circle><circle cx="5" cy="12" r="1"></circle>"#,
};
pub const FILE_TEXT: IconData = IconData {
    class: "lucide lucide-file-text",
    body: r#"<path d="M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z"></path><path d="M14 2v5a1 1 0 0 0 1 1h5"></path><path d="M10 9H8"></path><path d="M16 13H8"></path><path d="M16 17H8"></path>"#,
};
pub const FLAG: IconData = IconData {
    class: "lucide lucide-flag",
    body: r#"<path d="M4 22V4a1 1 0 0 1 .4-.8A6 6 0 0 1 8 2c3 0 5 2 7.333 2q2 0 3.067-.8A1 1 0 0 1 20 4v10a1 1 0 0 1-.4.8A6 6 0 0 1 16 16c-3 0-5-2-8-2a6 6 0 0 0-4 1.528"></path>"#,
};
pub const FLAG_OFF: IconData = IconData {
    class: "lucide lucide-flag-off",
    body: r#"<path d="M16 16c-3 0-5-2-8-2a6 6 0 0 0-4 1.528"></path><path d="m2 2 20 20"></path><path d="M4 22V4"></path><path d="M7.656 2H8c3 0 5 2 7.333 2q2 0 3.067-.8A1 1 0 0 1 20 4v10.347"></path>"#,
};
pub const FLAME: IconData = IconData {
    class: "lucide lucide-flame",
    body: r#"<path d="M12 3q1 4 4 6.5t3 5.5a1 1 0 0 1-14 0 5 5 0 0 1 1-3 1 1 0 0 0 5 0c0-2-1.5-3-1.5-5q0-2 2.5-4"></path>"#,
};
pub const GAUGE: IconData = IconData {
    class: "lucide lucide-gauge",
    body: r#"<path d="m12 14 4-4"></path><path d="M3.34 19a10 10 0 1 1 17.32 0"></path>"#,
};
pub const GIFT: IconData = IconData {
    class: "lucide lucide-gift",
    body: r#"<path d="M12 7v14"></path><path d="M20 11v8a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-8"></path><path d="M7.5 7a1 1 0 0 1 0-5A4.8 8 0 0 1 12 7a4.8 8 0 0 1 4.5-5 1 1 0 0 1 0 5"></path><rect x="3" y="7" width="18" height="4" rx="1"></rect>"#,
};
pub const GLOBE: IconData = IconData {
    class: "lucide lucide-globe",
    body: r#"<circle cx="12" cy="12" r="10"></circle><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"></path><path d="M2 12h20"></path>"#,
};
pub const GLOBE_2: IconData = IconData {
    class: "lucide lucide-earth lucide-globe-2",
    body: r#"<path d="M21.54 15H17a2 2 0 0 0-2 2v4.54"></path><path d="M7 3.34V5a3 3 0 0 0 3 3a2 2 0 0 1 2 2c0 1.1.9 2 2 2a2 2 0 0 0 2-2c0-1.1.9-2 2-2h3.17"></path><path d="M11 21.95V18a2 2 0 0 0-2-2a2 2 0 0 1-2-2v-1a2 2 0 0 0-2-2H2.05"></path><circle cx="12" cy="12" r="10"></circle>"#,
};
pub const GRADUATION_CAP: IconData = IconData {
    class: "lucide lucide-graduation-cap",
    body: r#"<path d="M21.42 10.922a1 1 0 0 0-.019-1.838L12.83 5.18a2 2 0 0 0-1.66 0L2.6 9.08a1 1 0 0 0 0 1.832l8.57 3.908a2 2 0 0 0 1.66 0z"></path><path d="M22 10v6"></path><path d="M6 12.5V16a6 3 0 0 0 12 0v-3.5"></path>"#,
};
pub const HEART: IconData = IconData {
    class: "lucide lucide-heart",
    body: r#"<path d="M2 9.5a5.5 5.5 0 0 1 9.591-3.676.56.56 0 0 0 .818 0A5.49 5.49 0 0 1 22 9.5c0 2.29-1.5 4-3 5.5l-5.492 5.313a2 2 0 0 1-3 .019L5 15c-1.5-1.5-3-3.2-3-5.5"></path>"#,
};
pub const HISTORY: IconData = IconData {
    class: "lucide lucide-rotate-ccw-clock lucide-history",
    body: r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path><path d="M3 3v5h5"></path><path d="M12 7v5l4 2"></path>"#,
};
pub const HOUSE: IconData = IconData {
    class: "lucide lucide-house lucide-home",
    body: r#"<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"></path><path d="M3 10a2 2 0 0 1 .709-1.528l7-6a2 2 0 0 1 2.582 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>"#,
};
pub const INFO: IconData = IconData {
    class: "lucide lucide-info",
    body: r#"<circle cx="12" cy="12" r="10"></circle><path d="M12 16v-4"></path><path d="M12 8h.01"></path>"#,
};
pub const LANDMARK: IconData = IconData {
    class: "lucide lucide-landmark",
    body: r#"<path d="M10 18v-7"></path><path d="M11.119 2.205a2 2 0 0 1 1.762 0l7.84 3.846A.5.5 0 0 1 20.5 7h-17a.5.5 0 0 1-.22-.949z"></path><path d="M14 18v-7"></path><path d="M18 18v-7"></path><path d="M3 22h18"></path><path d="M6 18v-7"></path>"#,
};
pub const LAYOUT_GRID: IconData = IconData {
    class: "lucide lucide-layout-grid",
    body: r#"<rect width="7" height="7" x="3" y="3" rx="1"></rect><rect width="7" height="7" x="14" y="3" rx="1"></rect><rect width="7" height="7" x="14" y="14" rx="1"></rect><rect width="7" height="7" x="3" y="14" rx="1"></rect>"#,
};
pub const LIBRARY: IconData = IconData {
    class: "lucide lucide-library",
    body: r#"<path d="m16 6 4 14"></path><path d="M12 6v14"></path><path d="M8 8v12"></path><path d="M4 4v16"></path>"#,
};
pub const LIGHTBULB: IconData = IconData {
    class: "lucide lucide-lightbulb",
    body: r#"<path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"></path><path d="M9 18h6"></path><path d="M10 22h4"></path>"#,
};
pub const LINE_CHART: IconData = IconData {
    class: "lucide lucide-chart-line lucide-line-chart",
    body: r#"<path d="M3 3v16a2 2 0 0 0 2 2h16"></path><path d="m19 9-5 5-4-4-3 3"></path>"#,
};
pub const LOCK: IconData = IconData {
    class: "lucide lucide-lock",
    body: r#"<rect width="18" height="11" x="3" y="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path>"#,
};
pub const MAIL_OPEN: IconData = IconData {
    class: "lucide lucide-mail-open",
    body: r#"<path d="M21.2 8.4c.5.38.8.97.8 1.6v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V10a2 2 0 0 1 .8-1.6l8-6a2 2 0 0 1 2.4 0l8 6Z"></path><path d="m22 10-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 10"></path>"#,
};
pub const MAP: IconData = IconData {
    class: "lucide lucide-map",
    body: r#"<path d="M14.106 5.553a2 2 0 0 0 1.788 0l3.659-1.83A1 1 0 0 1 21 4.619v12.764a1 1 0 0 1-.553.894l-4.553 2.277a2 2 0 0 1-1.788 0l-4.212-2.106a2 2 0 0 0-1.788 0l-3.659 1.83A1 1 0 0 1 3 19.381V6.618a1 1 0 0 1 .553-.894l4.553-2.277a2 2 0 0 1 1.788 0z"></path><path d="M15 5.764v15"></path><path d="M9 3.236v15"></path>"#,
};
pub const MAP_PIN: IconData = IconData {
    class: "lucide lucide-map-pin",
    body: r#"<path d="M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0"></path><circle cx="12" cy="10" r="3"></circle>"#,
};
pub const MEDAL: IconData = IconData {
    class: "lucide lucide-medal",
    body: r#"<path d="M7.21 15 2.66 7.14a2 2 0 0 1 .13-2.2L4.4 2.8A2 2 0 0 1 6 2h12a2 2 0 0 1 1.6.8l1.6 2.14a2 2 0 0 1 .14 2.2L16.79 15"></path><path d="M11 12 5.12 2.2"></path><path d="m13 12 5.88-9.8"></path><path d="M8 7h8"></path><circle cx="12" cy="17" r="5"></circle><path d="M12 18v-2h-.5"></path>"#,
};
pub const MOUNTAIN: IconData =
    IconData { class: "lucide lucide-mountain", body: r#"<path d="m8 3 4 8 5-5 5 15H2L8 3z"></path>"# };
pub const MOUSE: IconData = IconData {
    class: "lucide lucide-mouse",
    body: r#"<rect x="5" y="2" width="14" height="20" rx="7"></rect><path d="M12 6v4"></path>"#,
};
pub const MOUSE_POINTER_2: IconData = IconData {
    class: "lucide lucide-mouse-pointer-2",
    body: r#"<path d="M4.037 4.688a.495.495 0 0 1 .651-.651l16 6.5a.5.5 0 0 1-.063.947l-6.124 1.58a2 2 0 0 0-1.438 1.435l-1.579 6.126a.5.5 0 0 1-.947.063z"></path>"#,
};
pub const MOUSE_POINTER_CLICK: IconData = IconData {
    class: "lucide lucide-mouse-pointer-click",
    body: r#"<path d="M14 4.1 12 6"></path><path d="m5.1 8-2.9-.8"></path><path d="m6 12-1.9 2"></path><path d="M7.2 2.2 8 5.1"></path><path d="M9.037 9.69a.498.498 0 0 1 .653-.653l11 4.5a.5.5 0 0 1-.074.949l-4.349 1.041a1 1 0 0 0-.74.739l-1.04 4.35a.5.5 0 0 1-.95.074z"></path>"#,
};
pub const MOVE: IconData = IconData {
    class: "lucide lucide-move",
    body: r#"<path d="M12 2v20"></path><path d="m15 19-3 3-3-3"></path><path d="m19 9 3 3-3 3"></path><path d="M2 12h20"></path><path d="m5 9-3 3 3 3"></path><path d="m9 5 3-3 3 3"></path>"#,
};
pub const NETWORK: IconData = IconData {
    class: "lucide lucide-network",
    body: r#"<rect x="16" y="16" width="6" height="6" rx="1"></rect><rect x="2" y="16" width="6" height="6" rx="1"></rect><rect x="9" y="2" width="6" height="6" rx="1"></rect><path d="M5 16v-3a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v3"></path><path d="M12 12V8"></path>"#,
};
pub const NEWSPAPER: IconData = IconData {
    class: "lucide lucide-newspaper",
    body: r#"<path d="M15 18h-5"></path><path d="M18 14h-8"></path><path d="M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-4 0v-9a2 2 0 0 1 2-2h2"></path><rect width="8" height="4" x="10" y="6" rx="1"></rect>"#,
};
pub const ORBIT: IconData = IconData {
    class: "lucide lucide-orbit",
    body: r#"<path d="M20.341 6.484A10 10 0 0 1 10.266 21.85"></path><path d="M3.659 17.516A10 10 0 0 1 13.74 2.152"></path><circle cx="12" cy="12" r="3"></circle><circle cx="19" cy="5" r="2"></circle><circle cx="5" cy="19" r="2"></circle>"#,
};
pub const PACKAGE_CHECK: IconData = IconData {
    class: "lucide lucide-package-check",
    body: r#"<path d="M12 22V12"></path><path d="m16 17 2 2 4-4"></path><path d="M21 11.127V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.729l7 4a2 2 0 0 0 2 .001l1.32-.753"></path><path d="M3.29 7 12 12l8.71-5"></path><path d="m7.5 4.27 8.997 5.148"></path>"#,
};
pub const PAUSE: IconData = IconData {
    class: "lucide lucide-pause",
    body: r#"<rect x="14" y="3" width="5" height="18" rx="1"></rect><rect x="5" y="3" width="5" height="18" rx="1"></rect>"#,
};
pub const PIE_CHART: IconData = IconData {
    class: "lucide lucide-chart-pie lucide-pie-chart",
    body: r#"<path d="M21 12c.552 0 1.005-.449.95-.998a10 10 0 0 0-8.953-8.951c-.55-.055-.998.398-.998.95v8a1 1 0 0 0 1 1z"></path><path d="M21.21 15.89A10 10 0 1 1 8 2.83"></path>"#,
};
pub const PIN: IconData = IconData {
    class: "lucide lucide-pin",
    body: r#"<path d="M12 17v5"></path><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z"></path>"#,
};
pub const PIN_OFF: IconData = IconData {
    class: "lucide lucide-pin-off",
    body: r#"<path d="M12 17v5"></path><path d="M15 9.34V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H7.89"></path><path d="m2 2 20 20"></path><path d="M9 9v1.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h11"></path>"#,
};
pub const PLAY: IconData = IconData {
    class: "lucide lucide-play",
    body: r#"<path d="M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z"></path>"#,
};
pub const QUOTE: IconData = IconData {
    class: "lucide lucide-quote",
    body: r#"<path d="M16 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z"></path><path d="M5 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z"></path>"#,
};
pub const RADIO: IconData = IconData {
    class: "lucide lucide-radio",
    body: r#"<path d="M16.247 7.761a6 6 0 0 1 0 8.478"></path><path d="M19.075 4.933a10 10 0 0 1 0 14.134"></path><path d="M4.925 19.067a10 10 0 0 1 0-14.134"></path><path d="M7.753 16.239a6 6 0 0 1 0-8.478"></path><circle cx="12" cy="12" r="2"></circle>"#,
};
pub const REFRESH_CW: IconData = IconData {
    class: "lucide lucide-refresh-cw",
    body: r#"<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"></path><path d="M21 3v5h-5"></path><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"></path><path d="M8 16H3v5"></path>"#,
};
pub const ROCKET: IconData = IconData {
    class: "lucide lucide-rocket",
    body: r#"<path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"></path><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09"></path><path d="M9 12a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.4 22.4 0 0 1-4 2z"></path><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 .05 5 .05"></path>"#,
};
pub const ROTATE_3D: IconData = IconData {
    class: "lucide lucide-rotate-3d lucide-rotate-3-d",
    body: r#"<path d="m15.194 13.707 3.814 1.86-1.86 3.814"></path><path d="M16.47214 7.52786 A 5 10 0 1 0 13 21.79796"></path><path d="M21.79796 11 A 10 5 0 1 0 19 15.57071"></path>"#,
};
pub const ROTATE_CCW: IconData = IconData {
    class: "lucide lucide-rotate-ccw",
    body: r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path><path d="M3 3v5h5"></path>"#,
};
pub const ROTATE_CW: IconData = IconData {
    class: "lucide lucide-rotate-cw",
    body: r#"<path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"></path><path d="M21 3v5h-5"></path>"#,
};
pub const ROUTE: IconData = IconData {
    class: "lucide lucide-route",
    body: r#"<circle cx="6" cy="19" r="3"></circle><path d="M9 19h8.5a3.5 3.5 0 0 0 0-7h-11a3.5 3.5 0 0 1 0-7H15"></path><circle cx="18" cy="5" r="3"></circle>"#,
};
pub const SCROLL_TEXT: IconData = IconData {
    class: "lucide lucide-scroll-text",
    body: r#"<path d="M15 12h-5"></path><path d="M15 8h-5"></path><path d="M19 17V5a2 2 0 0 0-2-2H4"></path><path d="M8 21h12a2 2 0 0 0 2-2v-1a1 1 0 0 0-1-1H11a1 1 0 0 0-1 1v1a2 2 0 1 1-4 0V5a2 2 0 1 0-4 0v2a1 1 0 0 0 1 1h3"></path>"#,
};
pub const SEARCH: IconData = IconData {
    class: "lucide lucide-search",
    body: r#"<path d="m21 21-4.34-4.34"></path><circle cx="11" cy="11" r="8"></circle>"#,
};
pub const SETTINGS: IconData = IconData {
    class: "lucide lucide-settings",
    body: r#"<path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915"></path><circle cx="12" cy="12" r="3"></circle>"#,
};
pub const SHIELD: IconData = IconData {
    class: "lucide lucide-shield",
    body: r#"<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"></path>"#,
};
pub const SHIELD_ALERT: IconData = IconData {
    class: "lucide lucide-shield-alert",
    body: r#"<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"></path><path d="M12 8v4"></path><path d="M12 16h.01"></path>"#,
};
pub const SHIELD_CHECK: IconData = IconData {
    class: "lucide lucide-shield-check",
    body: r#"<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"></path><path d="m9 12 2 2 4-4"></path>"#,
};
pub const SHIELD_HALF: IconData = IconData {
    class: "lucide lucide-shield-half",
    body: r#"<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"></path><path d="M12 22V2"></path>"#,
};
pub const SKULL: IconData = IconData {
    class: "lucide lucide-skull",
    body: r#"<path d="m12.5 17-.5-1-.5 1h1z"></path><path d="M15 22a1 1 0 0 0 1-1v-1a2 2 0 0 0 1.56-3.25 8 8 0 1 0-11.12 0A2 2 0 0 0 8 20v1a1 1 0 0 0 1 1z"></path><circle cx="15" cy="12" r="1"></circle><circle cx="9" cy="12" r="1"></circle>"#,
};
pub const SNOWFLAKE: IconData = IconData {
    class: "lucide lucide-snowflake",
    body: r#"<path d="m10 20-1.25-2.5L6 18"></path><path d="M10 4 8.75 6.5 6 6"></path><path d="m14 20 1.25-2.5L18 18"></path><path d="m14 4 1.25 2.5L18 6"></path><path d="m17 21-3-6h-4"></path><path d="m17 3-3 6 1.5 3"></path><path d="M2 12h6.5L10 9"></path><path d="m20 10-1.5 2 1.5 2"></path><path d="M22 12h-6.5L14 15"></path><path d="m4 10 1.5 2L4 14"></path><path d="m7 21 3-6-1.5-3"></path><path d="m7 3 3 6h4"></path>"#,
};
pub const SPARKLE: IconData = IconData {
    class: "lucide lucide-sparkle",
    body: r#"<path d="M11.017 2.814a1 1 0 0 1 1.966 0l1.051 5.558a2 2 0 0 0 1.594 1.594l5.558 1.051a1 1 0 0 1 0 1.966l-5.558 1.051a2 2 0 0 0-1.594 1.594l-1.051 5.558a1 1 0 0 1-1.966 0l-1.051-5.558a2 2 0 0 0-1.594-1.594l-5.558-1.051a1 1 0 0 1 0-1.966l5.558-1.051a2 2 0 0 0 1.594-1.594z"></path>"#,
};
pub const SPARKLES: IconData = IconData {
    class: "lucide lucide-sparkles lucide-stars",
    body: r#"<path d="M11.017 2.814a1 1 0 0 1 1.966 0l1.051 5.558a2 2 0 0 0 1.594 1.594l5.558 1.051a1 1 0 0 1 0 1.966l-5.558 1.051a2 2 0 0 0-1.594 1.594l-1.051 5.558a1 1 0 0 1-1.966 0l-1.051-5.558a2 2 0 0 0-1.594-1.594l-5.558-1.051a1 1 0 0 1 0-1.966l5.558-1.051a2 2 0 0 0 1.594-1.594z"></path><path d="M20 2v4"></path><path d="M22 4h-4"></path><circle cx="4" cy="20" r="2"></circle>"#,
};
pub const SQUARE: IconData =
    IconData { class: "lucide lucide-square", body: r#"<rect width="18" height="18" x="3" y="3" rx="2"></rect>"# };
pub const SUN: IconData = IconData {
    class: "lucide lucide-sun",
    body: r#"<circle cx="12" cy="12" r="4"></circle><path d="M12 2v2"></path><path d="M12 20v2"></path><path d="m4.93 4.93 1.41 1.41"></path><path d="m17.66 17.66 1.41 1.41"></path><path d="M2 12h2"></path><path d="M20 12h2"></path><path d="m6.34 17.66-1.41 1.41"></path><path d="m19.07 4.93-1.41 1.41"></path>"#,
};
pub const SWORDS: IconData = IconData {
    class: "lucide lucide-swords",
    body: r#"<path d="m13 19 6-6"></path><path d="M14.5 17.5 3.586 6.586A2 2 0 013 5.172V3h2.172a2 2 0 011.414.586L17.5 14.5"></path><path d="m14.828 6.172 2.586-2.586A2 2 0 0118.828 3H21v2.172a2 2 0 01-.586 1.414l-2.586 2.586"></path><path d="m16 16 4 4"></path><path d="m19 21 2-2"></path><path d="m5 14 4 4"></path><path d="m5 21-2-2"></path><path d="M7.5 16.5 4 20"></path>"#,
};
pub const TARGET: IconData = IconData {
    class: "lucide lucide-target",
    body: r#"<circle cx="12" cy="12" r="10"></circle><circle cx="12" cy="12" r="6"></circle><circle cx="12" cy="12" r="2"></circle>"#,
};
pub const TIMER: IconData = IconData {
    class: "lucide lucide-timer",
    body: r#"<line x1="10" x2="14" y1="2" y2="2"></line><line x1="12" x2="15" y1="14" y2="11"></line><circle cx="12" cy="14" r="8"></circle>"#,
};
pub const TRENDING_DOWN: IconData = IconData {
    class: "lucide lucide-trending-down",
    body: r#"<path d="M16 17h6v-6"></path><path d="m22 17-8.5-8.5-5 5L2 7"></path>"#,
};
pub const TRENDING_UP: IconData = IconData {
    class: "lucide lucide-trending-up",
    body: r#"<path d="M16 7h6v6"></path><path d="m22 7-8.5 8.5-5-5L2 17"></path>"#,
};
pub const TRIANGLE_ALERT: IconData = IconData {
    class: "lucide lucide-triangle-alert lucide-alert-triangle",
    body: r#"<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"></path><path d="M12 9v4"></path><path d="M12 17h.01"></path>"#,
};
pub const TROPHY: IconData = IconData {
    class: "lucide lucide-trophy",
    body: r#"<path d="M10 14.66V17a1 1 0 0 1-1 1 2 2 0 0 0-2 2v2"></path><path d="M14 14.66V17a1 1 0 0 0 1 1 2 2 0 0 1 2 2v2"></path><path d="M17.916 10H19.5A2.5 2.5 0 0 0 22 7.5V5a1 1 0 0 0-1-1h-3"></path><path d="M4 22h16"></path><path d="M6 9a6 6 0 0 0 12 0V3a1 1 0 0 0-1-1H7a1 1 0 0 0-1 1z"></path><path d="M6.084 10H4.5A2.5 2.5 0 0 1 2 7.5V5a1 1 0 0 1 1-1h3"></path>"#,
};
pub const USERS: IconData = IconData {
    class: "lucide lucide-users",
    body: r#"<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"></path><path d="M16 3.128a4 4 0 0 1 0 7.744"></path><path d="M22 21v-2a4 4 0 0 0-3-3.87"></path><circle cx="9" cy="7" r="4"></circle>"#,
};
pub const WIND: IconData = IconData {
    class: "lucide lucide-wind",
    body: r#"<path d="M12.8 19.6A2 2 0 1 0 14 16H2"></path><path d="M17.5 8a2.5 2.5 0 1 1 2 4H2"></path><path d="M9.8 4.4A2 2 0 1 1 11 8H2"></path>"#,
};
pub const X: IconData =
    IconData { class: "lucide lucide-x", body: r#"<path d="M18 6 6 18"></path><path d="m6 6 12 12"></path>"# };
pub const ZAP: IconData = IconData {
    class: "lucide lucide-zap",
    body: r#"<path d="M15.914 4a1.5 1.5 0 00-2.474-1.561l-9 9A1.5 1.5 0 005.5 14h4.002a.5.5 0 01.471.666L8.086 20a1.5 1.5 0 002.475 1.56l9-9A1.5 1.5 0 0018.5 10h-3.997a.5.5 0 01-.472-.667z"></path>"#,
};
pub const ZOOM_IN: IconData = IconData {
    class: "lucide lucide-zoom-in",
    body: r#"<circle cx="11" cy="11" r="8"></circle><line x1="21" x2="16.65" y1="21" y2="16.65"></line><line x1="11" x2="11" y1="8" y2="14"></line><line x1="8" x2="14" y1="11" y2="11"></line>"#,
};
pub const ZOOM_OUT: IconData = IconData {
    class: "lucide lucide-zoom-out",
    body: r#"<circle cx="11" cy="11" r="8"></circle><line x1="21" x2="16.65" y1="21" y2="16.65"></line><line x1="8" x2="14" y1="11" y2="11"></line>"#,
};

/// An icon at `size` pixels with `stroke_width` lines.
#[component]
pub fn Icon(
    icon: IconData,
    #[prop(default = 24.0)] size: f64,
    #[prop(default = 2.0)] stroke_width: f64,
    #[prop(optional, into)] class: MaybeProp<String>,
    #[prop(optional, into)] label: MaybeProp<String>,
    /// Line colour, like lucide's `color`.
    #[prop(optional, into)]
    color: MaybeProp<String>,
    #[prop(optional, into)] style: MaybeProp<String>,
) -> impl IntoView {
    let label = label.get_untracked();
    let stroke = color.get_untracked().unwrap_or_else(|| "currentColor".into());
    let class = match class.get_untracked() {
        Some(c) if !c.is_empty() => format!("{} {c}", icon.class),
        _ => icon.class.to_string(),
    };
    let hidden = label.is_none().then_some("true");
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=num(size)
            height=num(size)
            viewBox="0 0 24 24"
            fill="none"
            stroke=stroke
            stroke-width=num(stroke_width)
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
            aria-hidden=hidden
            aria-label=label
            style=style.get_untracked()
            inner_html=icon.body
        ></svg>
    }
}
