use super::*;

#[test]
fn default_viewer_settings_are_zero_override() {
    let settings = ChunkViewerSettings::default();
    assert_eq!(settings.request_radius, 0);
    assert_eq!(settings.keep_radius, 0);
    assert_eq!(settings.priority, 0);
}
