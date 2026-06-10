use std::sync::atomic::{AtomicU8, Ordering};

use objc2::ClassType;
use objc2_app_kit::{
    NSApplication, NSBitmapImageRep, NSDeviceRGBColorSpace, NSImage, NSRunningApplication,
};
use objc2_foundation::{ns_string, MainThreadMarker, NSProcessInfo, NSSize, NSString};

use super::APP_DISPLAY_NAME;
use crate::debug;

const STATUS_TRY: u8 = 0;
const STATUS_DONE: u8 = 1;
const STATUS_GAVE_UP: u8 = 2;

static PLATFORM_STATUS: AtomicU8 = AtomicU8::new(STATUS_TRY);

enum PlatformAttempt {
    Done,
    TryAgain,
    GaveUp,
}

pub fn early_init() {
    let name = NSString::from_str(APP_DISPLAY_NAME);
    unsafe {
        NSProcessInfo::processInfo().setProcessName(&name);
    }
}

pub fn try_apply(png_bytes: &'static [u8]) {
    match PLATFORM_STATUS.load(Ordering::Relaxed) {
        STATUS_DONE | STATUS_GAVE_UP => {}
        _ => match apply_once(png_bytes) {
            PlatformAttempt::Done => {
                PLATFORM_STATUS.store(STATUS_DONE, Ordering::Relaxed);
            }
            PlatformAttempt::GaveUp => {
                PLATFORM_STATUS.store(STATUS_GAVE_UP, Ordering::Relaxed);
                debug::warn("Could not apply macOS platform branding");
            }
            PlatformAttempt::TryAgain => {}
        },
    }
}

fn apply_once(png_bytes: &'static [u8]) -> PlatformAttempt {
    let Some(mtm) = MainThreadMarker::new() else {
        return PlatformAttempt::TryAgain;
    };

    let app = NSApplication::sharedApplication(mtm);

    let launched = unsafe { NSRunningApplication::currentApplication().isFinishedLaunching() };
    if !launched {
        return PlatformAttempt::TryAgain;
    }

    apply_app_name(&app);
    set_dock_icon(png_bytes, &app);

    PlatformAttempt::Done
}

fn apply_app_name(app: &NSApplication) {
    let name = NSString::from_str(APP_DISPLAY_NAME);

    let Some(main_menu) = (unsafe { app.mainMenu() }) else {
        return;
    };

    let Some(app_item) = (unsafe { main_menu.itemAtIndex(0) }) else {
        return;
    };

    unsafe {
        app_item.setTitle(&name);
    }

    let Some(app_menu) = (unsafe { app_item.submenu() }) else {
        return;
    };

    unsafe {
        app_menu.setTitle(&name);

        if let Some(about_item) = app_menu.itemAtIndex(0) {
            let about_title = ns_string!("About ").stringByAppendingString(&name);
            about_item.setTitle(&about_title);
        }

        if let Some(hide_item) = app_menu.itemAtIndex(3) {
            let hide_title = ns_string!("Hide ").stringByAppendingString(&name);
            hide_item.setTitle(&hide_title);
        }

        if let Some(quit_item) = app_menu.itemAtIndex(7) {
            let quit_title = ns_string!("Quit ").stringByAppendingString(&name);
            quit_item.setTitle(&quit_title);
        }
    }
}

fn set_dock_icon(png_bytes: &'static [u8], app: &NSApplication) -> PlatformAttempt {
    let image = match image::load_from_memory(png_bytes) {
        Ok(image) => image.to_rgba8(),
        Err(error) => {
            debug::warn(format!("Dock icon PNG decode failed: {error}"));
            return PlatformAttempt::GaveUp;
        }
    };

    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return PlatformAttempt::GaveUp;
    }

    let Some(image_rep) = (unsafe {
        NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            [image.as_ptr().cast_mut()].as_mut_ptr(),
            width as isize,
            height as isize,
            8,
            4,
            true,
            false,
            NSDeviceRGBColorSpace,
            (width * 4) as isize,
            32,
        )
    }) else {
        return PlatformAttempt::TryAgain;
    };

    let app_icon = unsafe {
        NSImage::initWithSize(
            NSImage::alloc(),
            NSSize::new(width as f64, height as f64),
        )
    };

    unsafe {
        app_icon.addRepresentation(&image_rep);
        app.setApplicationIconImage(Some(&app_icon));
        app.dockTile().display();
    }

    PlatformAttempt::Done
}
