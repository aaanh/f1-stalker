#[cfg(target_os = "macos")]
mod macos_platform;

pub mod fonts;

use bytes::Bytes;
use iced::widget::{column, image, row, text, Space};
use iced::{Element, Length};

use crate::state::Message;
use crate::ui::theme::ACCENT;

pub const APP_DISPLAY_NAME: &str = "F1 Stalker";

const LOGO_TITLE_BAR: &[u8] =
    include_bytes!("../../AppIcons/Assets.xcassets/AppIcon.appiconset/64.png");
const LOGO_BOOT: &[u8] =
    include_bytes!("../../AppIcons/Assets.xcassets/AppIcon.appiconset/128.png");
const LOGO_ABOUT: &[u8] =
    include_bytes!("../../AppIcons/Assets.xcassets/AppIcon.appiconset/180.png");
const LOGO_WINDOW: &[u8] =
    include_bytes!("../../AppIcons/Assets.xcassets/AppIcon.appiconset/512.png");
#[cfg(target_os = "macos")]
const LOGO_DOCK: &[u8] =
    include_bytes!("../../AppIcons/Assets.xcassets/AppIcon.appiconset/1024.png");

pub fn window_icon() -> iced::window::Icon {
    iced::window::icon::from_file_data(LOGO_WINDOW, None).expect("embedded app icon")
}

#[cfg(target_os = "macos")]
pub fn early_platform_init() {
    macos_platform::early_init();
}

#[cfg(not(target_os = "macos"))]
pub fn early_platform_init() {}

pub fn apply_platform_branding() {
    #[cfg(target_os = "macos")]
    macos_platform::try_apply(LOGO_DOCK);
}

fn logo(size: f32, data: &'static [u8]) -> Element<'static, Message> {
    image(image::Handle::from_bytes(Bytes::from_static(data)))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .content_fit(iced::ContentFit::Contain)
        .into()
}

pub fn title_bar_brand() -> Element<'static, Message> {
    row![
        logo(22.0, LOGO_TITLE_BAR),
        Space::with_width(8),
        text(APP_DISPLAY_NAME).size(14).color(ACCENT),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn boot_brand() -> Element<'static, Message> {
    column![
        logo(72.0, LOGO_BOOT),
        Space::with_height(12),
        text(APP_DISPLAY_NAME).size(28).color(ACCENT),
    ]
    .align_x(iced::Alignment::Center)
    .into()
}

pub fn about_brand() -> Element<'static, Message> {
    column![
        logo(64.0, LOGO_ABOUT),
        Space::with_height(8),
        text(APP_DISPLAY_NAME).size(22).color(ACCENT),
    ]
    .align_x(iced::Alignment::Center)
    .width(Length::Fill)
    .into()
}

pub fn header_brand() -> Element<'static, Message> {
    row![
        logo(36.0, LOGO_BOOT),
        Space::with_width(12),
        text(APP_DISPLAY_NAME).size(30).color(ACCENT),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}
