use iced::widget::{column, row, text, Space};
use iced::widget::{self, svg::Handle};
use iced::{Color, Element, Length};

use crate::state::Message;
use crate::ui::theme::{muted, text_color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Refresh,
    Help,
    Circle,
    CircleFilled,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    ChevronDown,
    Check,
    Minus,
    Alert,
    Loader,
    Dashboard,
    Settings,
    UserPlus,
    Pin,
    PinOff,
    Close,
    Trophy,
    Users,
    Search,
    Copy,
    Trash,
    Database,
    FileText,
    Calendar,
    Maximize,
}

impl Icon {
    fn bytes(self) -> &'static [u8] {
        match self {
            Icon::Refresh => include_bytes!("../assets/icons/refresh.svg"),
            Icon::Help => include_bytes!("../assets/icons/help.svg"),
            Icon::Circle => include_bytes!("../assets/icons/circle.svg"),
            Icon::CircleFilled => include_bytes!("../assets/icons/circle-filled.svg"),
            Icon::ChevronLeft => include_bytes!("../assets/icons/chevron-left.svg"),
            Icon::ChevronRight => include_bytes!("../assets/icons/chevron-right.svg"),
            Icon::ChevronUp => include_bytes!("../assets/icons/chevron-up.svg"),
            Icon::ChevronDown => include_bytes!("../assets/icons/chevron-down.svg"),
            Icon::Check => include_bytes!("../assets/icons/check.svg"),
            Icon::Minus => include_bytes!("../assets/icons/minus.svg"),
            Icon::Alert => include_bytes!("../assets/icons/alert.svg"),
            Icon::Loader => include_bytes!("../assets/icons/loader.svg"),
            Icon::Dashboard => include_bytes!("../assets/icons/dashboard.svg"),
            Icon::Settings => include_bytes!("../assets/icons/settings.svg"),
            Icon::UserPlus => include_bytes!("../assets/icons/user-plus.svg"),
            Icon::Pin => include_bytes!("../assets/icons/pin.svg"),
            Icon::PinOff => include_bytes!("../assets/icons/pin-off.svg"),
            Icon::Close => include_bytes!("../assets/icons/close.svg"),
            Icon::Trophy => include_bytes!("../assets/icons/trophy.svg"),
            Icon::Users => include_bytes!("../assets/icons/users.svg"),
            Icon::Search => include_bytes!("../assets/icons/search.svg"),
            Icon::Copy => include_bytes!("../assets/icons/copy.svg"),
            Icon::Trash => include_bytes!("../assets/icons/trash.svg"),
            Icon::Database => include_bytes!("../assets/icons/database.svg"),
            Icon::FileText => include_bytes!("../assets/icons/file-text.svg"),
            Icon::Calendar => include_bytes!("../assets/icons/calendar.svg"),
            Icon::Maximize => include_bytes!("../assets/icons/maximize-2.svg"),
        }
    }

    pub fn handle(self) -> Handle {
        Handle::from_memory(self.bytes())
    }
}

pub fn icon(icon: Icon, size: f32, color: Color) -> Element<'static, Message> {
    widget::svg(icon.handle())
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .content_fit(iced::ContentFit::Contain)
        .style(move |_, _| widget::svg::Style {
            color: Some(color),
        })
        .into()
}

pub fn icon_label(
    icon_kind: Icon,
    icon_size: f32,
    icon_color: Color,
    label: impl Into<String>,
    label_size: u16,
    label_color: Color,
) -> Element<'static, Message> {
    row![
        icon(icon_kind, icon_size, icon_color),
        text(label.into()).size(label_size).color(label_color),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn section_heading(
    icon_kind: Icon,
    title: &'static str,
    subtitle: Option<Element<'static, Message>>,
) -> Element<'static, Message> {
    let mut heading = column![
        icon_label(icon_kind, 16.0, text_color(), title, 15, text_color()),
    ]
    .spacing(2);

    if let Some(subtitle) = subtitle {
        heading = heading.push(
            row![
                Space::with_width(22.0),
                subtitle,
            ]
            .align_y(iced::Alignment::Center),
        );
    }

    heading.into()
}

pub fn subtitle_text(content: impl Into<String>) -> Element<'static, Message> {
    text(content.into()).size(11).color(muted()).into()
}
