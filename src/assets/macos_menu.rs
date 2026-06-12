use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex};

use objc2::mutability::MainThreadOnly;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{declare_class, msg_send_id, sel, ClassType, DeclaredClass};
use objc2_app_kit::{
    NSApplication, NSEventModifierFlags, NSMenu, NSMenuItem,
};
use objc2_foundation::{ns_string, MainThreadMarker, NSInteger, NSString};

use crate::state::{Message, Screen, WindowAction};

const TAG_ABOUT: NSInteger = 1;
const TAG_SETTINGS: NSInteger = 2;
const TAG_DASHBOARD: NSInteger = 3;
const TAG_REFRESH: NSInteger = 4;
const TAG_FULLSCREEN: NSInteger = 5;
const TAG_MINIMIZE: NSInteger = 6;
const TAG_PIN_DRIVER: NSInteger = 7;

static MENUS_INSTALLED: AtomicBool = AtomicBool::new(false);
static PENDING: Mutex<Vec<Message>> = Mutex::new(Vec::new());

thread_local! {
    static MENU_HANDLER: RefCell<Option<Retained<MenuHandler>>> = const { RefCell::new(None) };
}

declare_class!(
    struct MenuHandler;

    unsafe impl ClassType for MenuHandler {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "F1StalkerMenuHandler";
    }

    impl DeclaredClass for MenuHandler {}

    unsafe impl MenuHandler {
        #[method(menuAction:)]
        fn menu_action(&self, sender: Option<&NSMenuItem>) {
            let Some(sender) = sender else {
                return;
            };

            let tag = unsafe { sender.tag() };
            if let Some(message) = message_for_tag(tag) {
                enqueue(message);
            }
        }
    }
);

pub fn drain_messages() -> Vec<Message> {
    std::mem::take(&mut *PENDING.lock().expect("menu queue lock"))
}

pub fn setup_menus(app: &NSApplication, mtm: MainThreadMarker) {
    if MENUS_INSTALLED.swap(true, Ordering::Relaxed) {
        return;
    }

    let handler = menu_handler(mtm);
    patch_app_menu(app, mtm, &handler);
    add_menus(app, mtm, &handler);
}

fn menu_handler(mtm: MainThreadMarker) -> Retained<MenuHandler> {
    MENU_HANDLER.with(|handler| {
        if let Some(handler) = handler.borrow().as_ref() {
            return handler.clone();
        }

        let this = mtm.alloc::<MenuHandler>();
        let created: Retained<MenuHandler> = unsafe { msg_send_id![this, init] };
        *handler.borrow_mut() = Some(created.clone());
        created
    })
}

fn patch_app_menu(app: &NSApplication, mtm: MainThreadMarker, handler: &MenuHandler) {
    let Some(main_menu) = (unsafe { app.mainMenu() }) else {
        return;
    };

    let Some(app_item) = (unsafe { main_menu.itemAtIndex(0) }) else {
        return;
    };

    let Some(app_menu) = (unsafe { app_item.submenu() }) else {
        return;
    };

    if let Some(about_item) = unsafe { app_menu.itemAtIndex(0) } {
        wire_item(&about_item, handler, TAG_ABOUT);
    }

    let preferences = make_item(
        mtm,
        "Settings…",
        ",",
        Some(NSEventModifierFlags::NSEventModifierFlagCommand),
        TAG_SETTINGS,
        handler,
    );
    unsafe {
        app_menu.insertItem_atIndex(&preferences, 2);
    }
}

fn add_menus(app: &NSApplication, mtm: MainThreadMarker, handler: &MenuHandler) {
    let Some(main_menu) = (unsafe { app.mainMenu() }) else {
        return;
    };

    let file_menu = NSMenu::new(mtm);
    file_menu.addItem(&make_item(
        mtm,
        "Refresh",
        "r",
        Some(NSEventModifierFlags::NSEventModifierFlagCommand),
        TAG_REFRESH,
        handler,
    ));
    unsafe {
        main_menu.insertItem_atIndex(&submenu_header(mtm, "File", &file_menu), 1);
    }

    let view_menu = NSMenu::new(mtm);
    view_menu.addItem(&make_item(
        mtm,
        "Dashboard",
        "1",
        Some(NSEventModifierFlags::NSEventModifierFlagCommand),
        TAG_DASHBOARD,
        handler,
    ));
    view_menu.addItem(&make_item(
        mtm,
        "Pin Driver…",
        "d",
        Some(NSEventModifierFlags::NSEventModifierFlagCommand),
        TAG_PIN_DRIVER,
        handler,
    ));
    view_menu.addItem(&NSMenuItem::separatorItem(mtm));
    view_menu.addItem(&make_item(
        mtm,
        "Enter Full Screen",
        "f",
        Some(
            NSEventModifierFlags::NSEventModifierFlagCommand
                | NSEventModifierFlags::NSEventModifierFlagControl,
        ),
        TAG_FULLSCREEN,
        handler,
    ));
    unsafe {
        main_menu.insertItem_atIndex(&submenu_header(mtm, "View", &view_menu), 2);
    }

    let window_menu = NSMenu::new(mtm);
    window_menu.addItem(&make_item(
        mtm,
        "Minimize",
        "m",
        Some(NSEventModifierFlags::NSEventModifierFlagCommand),
        TAG_MINIMIZE,
        handler,
    ));
    window_menu.addItem(&make_item(
        mtm,
        "Enter Full Screen",
        "f",
        Some(
            NSEventModifierFlags::NSEventModifierFlagCommand
                | NSEventModifierFlags::NSEventModifierFlagControl,
        ),
        TAG_FULLSCREEN,
        handler,
    ));
    unsafe {
        main_menu.insertItem_atIndex(&submenu_header(mtm, "Window", &window_menu), 3);
    }

    let help_menu = NSMenu::new(mtm);
    help_menu.addItem(&make_item(
        mtm,
        "F1 Stalker Help",
        "",
        None,
        TAG_ABOUT,
        handler,
    ));
    unsafe {
        main_menu.insertItem_atIndex(&submenu_header(mtm, "Help", &help_menu), 4);
    }
}

fn submenu_header(mtm: MainThreadMarker, title: &str, menu: &NSMenu) -> Retained<NSMenuItem> {
    let item = NSMenuItem::new(mtm);
    let title = NSString::from_str(title);
    unsafe {
        item.setTitle(&title);
    }
    item.setSubmenu(Some(menu));
    item
}

fn make_item(
    mtm: MainThreadMarker,
    title: &str,
    key: &str,
    modifiers: Option<NSEventModifierFlags>,
    tag: NSInteger,
    handler: &MenuHandler,
) -> Retained<NSMenuItem> {
    let title = NSString::from_str(title);
    let key_equivalent = if key.is_empty() {
        ns_string!("").to_owned()
    } else {
        NSString::from_str(key)
    };
    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &title,
            Some(sel!(menuAction:)),
            &key_equivalent,
        )
    };

    if let Some(modifiers) = modifiers {
        item.setKeyEquivalentModifierMask(modifiers);
    }

    wire_item(&item, handler, tag);
    item
}

fn wire_item(item: &NSMenuItem, handler: &MenuHandler, tag: NSInteger) {
    unsafe {
        item.setTarget(Some(handler));
        item.setAction(Some(sel!(menuAction:)));
        item.setTag(tag);
    }
}

fn message_for_tag(tag: NSInteger) -> Option<Message> {
    match tag {
        TAG_ABOUT => Some(Message::OpenAbout),
        TAG_SETTINGS => Some(Message::Navigate(Screen::Settings)),
        TAG_DASHBOARD => Some(Message::Navigate(Screen::Dashboard)),
        TAG_REFRESH => Some(Message::Refresh),
        TAG_FULLSCREEN => Some(Message::WindowAction(WindowAction::Fullscreen)),
        TAG_MINIMIZE => Some(Message::WindowAction(WindowAction::Minimize)),
        TAG_PIN_DRIVER => Some(Message::OpenDriverPicker),
        _ => None,
    }
}

fn enqueue(message: Message) {
    PENDING.lock().expect("menu queue lock").push(message);
}
