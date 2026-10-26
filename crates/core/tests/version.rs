use warroom_core::version::{is_newer, release_number, version_parts};

#[test]
fn reads_release_tags() {
    assert_eq!(version_parts("v0.2.8-alpha"), [0, 2, 8]);
    assert_eq!(version_parts("1.10"), [1, 10]);
}

#[test]
fn drops_the_pre_release_label() {
    assert_eq!(release_number("0.3.0-beta"), "0.3.0");
    assert_eq!(release_number("v0.3.0-beta.2+build.5"), "0.3.0");
    assert_eq!(release_number("0.2.7"), "0.2.7");
}

#[test]
fn compares_by_number_not_text() {
    assert!(is_newer("v0.2.10", "0.2.9"));
    assert!(is_newer("v0.3.0", "0.2.9"));
    assert!(!is_newer("v0.2.6", "0.2.7"));
    assert!(is_newer("v0.3.0-beta", "0.2.7"));
}

#[test]
fn puts_a_full_release_after_its_pre_releases() {
    assert!(is_newer("v0.3.0", "0.3.0-beta"));
    assert!(!is_newer("v0.3.0-beta", "0.3.0"));
    assert!(!is_newer("v0.2.7-alpha", "0.2.7"));
    assert!(!is_newer("v0.3.0-beta", "0.3.0-beta"));
}

#[test]
fn orders_pre_releases_by_label() {
    assert!(is_newer("v0.3.0-rc", "0.3.0-beta"));
    assert!(is_newer("v0.3.0-beta.10", "0.3.0-beta.2"));
    assert!(!is_newer("v0.3.0-alpha", "0.3.0-beta"));
}
