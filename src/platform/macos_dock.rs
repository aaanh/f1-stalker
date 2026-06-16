#[cfg(target_os = "macos")]
pub fn set_dock_visible(visible: bool) {
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
    use objc2_foundation::MainThreadMarker;

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };

    let app = NSApplication::sharedApplication(mtm);
    let policy = if visible {
        NSApplicationActivationPolicy::Regular
    } else {
        NSApplicationActivationPolicy::Accessory
    };

    let _ = app.setActivationPolicy(policy);
}

#[cfg(not(target_os = "macos"))]
pub fn set_dock_visible(_visible: bool) {}
