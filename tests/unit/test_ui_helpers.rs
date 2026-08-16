use oled_wallpaper::configurator::ConfiguratorApp;

#[test]
fn clamp_zoom_bounds() {
    assert_eq!(ConfiguratorApp::clamp_zoom(0.0), 0.1);
    assert_eq!(ConfiguratorApp::clamp_zoom(10.0), 5.0);
    assert_eq!(ConfiguratorApp::clamp_zoom(2.5), 2.5);
}
