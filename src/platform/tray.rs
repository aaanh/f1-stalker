use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

const LOGO_TRAY: &[u8] =
    include_bytes!("../../AppIcons/Assets.xcassets/AppIcon.appiconset/64.png");

static SHOW_ID: std::sync::OnceLock<tray_icon::menu::MenuId> = std::sync::OnceLock::new();
static QUIT_ID: std::sync::OnceLock<tray_icon::menu::MenuId> = std::sync::OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Show,
    Quit,
}

pub fn install() -> Result<TrayIcon, String> {
    let icon = load_icon()?;
    let show = MenuItem::new("Show F1 Stalker", true, None);
    let quit = MenuItem::new("Quit", true, None);
    let show_id = show.id().clone();
    let quit_id = quit.id().clone();

    let menu = Menu::new();
    menu.append(&show).map_err(|error| error.to_string())?;
    menu.append(&PredefinedMenuItem::separator())
        .map_err(|error| error.to_string())?;
    menu.append(&quit).map_err(|error| error.to_string())?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("F1 Stalker")
        .with_icon(icon)
        .build()
        .map_err(|error| error.to_string())?;

    let _ = SHOW_ID.set(show_id);
    let _ = QUIT_ID.set(quit_id);
    Ok(tray)
}

pub fn poll_actions() -> Vec<TrayAction> {
    let Some(show_id) = SHOW_ID.get() else {
        return Vec::new();
    };
    let Some(quit_id) = QUIT_ID.get() else {
        return Vec::new();
    };

    MenuEvent::receiver()
        .try_iter()
        .filter_map(|event| {
            if event.id == *show_id {
                Some(TrayAction::Show)
            } else if event.id == *quit_id {
                Some(TrayAction::Quit)
            } else {
                None
            }
        })
        .collect()
}

fn load_icon() -> Result<Icon, String> {
    let image = image::load_from_memory(LOGO_TRAY).map_err(|error| error.to_string())?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    Icon::from_rgba(rgba.into_raw(), width, height).map_err(|error| error.to_string())
}
