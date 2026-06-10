use iced::widget::{button, container, mouse_area, row, stack, text, Space};
use iced::{Background, Color, Element, Length};

use crate::state::{Message, Screen};
use crate::ui::icons::{icon, Icon};
use crate::ui::theme::{ACCENT, BORDER, MUTED, SURFACE, TEXT};

const GROUP_RADIUS: f32 = 6.0;
const SEGMENT_RADIUS: f32 = 4.0;

#[derive(Copy, Clone, PartialEq, Eq)]
enum GroupPosition {
    Only,
    First,
    Middle,
    Last,
}

pub fn nav_tab_group(active: Screen) -> Element<'static, Message> {
    let tabs = [
        (Icon::Dashboard, "Dashboard", Screen::Dashboard),
        (Icon::Settings, "Settings", Screen::Settings),
    ];
    let total = tabs.len();
    let segments = tabs
        .into_iter()
        .enumerate()
        .map(|(index, (icon_kind, label, screen))| {
            nav_tab_segment(icon_kind, label, screen, active, group_position(index, total))
        })
        .collect();
    button_group(segments)
}

pub fn danger_button_group(actions: &[(Icon, &'static str, Message)]) -> Element<'static, Message> {
    let total = actions.len();
    let segments: std::vec::Vec<Element<Message>> = actions
        .iter()
        .enumerate()
        .map(|(index, (icon_kind, label, message))| {
            danger_button_segment(*icon_kind, label, message.clone(), group_position(index, total))
        })
        .collect();
    button_group(segments)
}

pub fn icon_button_group(actions: &[(Icon, Message)]) -> Element<'static, Message> {
    icon_button_group_sized(actions, 16.0, [8, 10])
}

pub fn icon_button_group_sized(
    actions: &[(Icon, Message)],
    icon_size: f32,
    padding: [u16; 2],
) -> Element<'static, Message> {
    let total = actions.len();
    let segments: std::vec::Vec<Element<Message>> = actions
        .iter()
        .enumerate()
        .map(|(index, (icon_kind, message))| {
            icon_button_segment(
                *icon_kind,
                message.clone(),
                group_position(index, total),
                icon_size,
                padding,
            )
        })
        .collect();
    button_group(segments)
}

pub fn action_button(label: &'static str, message: Message) -> Element<'static, Message> {
    action_button_icon(None, label, message)
}

pub fn action_button_icon(
    icon_kind: Option<Icon>,
    label: &'static str,
    message: Message,
) -> Element<'static, Message> {
    button(button_label(icon_kind, label, TEXT))
        .padding([10, 16])
        .on_press(message)
        .style(|_, status| standalone_button_style(status, ACCENT, TEXT, ACCENT))
        .into()
}

pub fn secondary_button(label: &'static str, message: Message) -> Element<'static, Message> {
    secondary_button_icon(None, label, message)
}

pub fn secondary_button_icon(
    icon_kind: Option<Icon>,
    label: &'static str,
    message: Message,
) -> Element<'static, Message> {
    button(button_label(icon_kind, label, TEXT))
        .padding([8, 14])
        .on_press(message)
        .style(|_, status| {
            use button::Status::{Active, Disabled, Hovered, Pressed};
            let (background, text_color, border_color) = match status {
                Active => (
                    Background::Color(Color::TRANSPARENT),
                    TEXT,
                    BORDER,
                ),
                Hovered => (
                    Background::Color(iced::Color {
                        a: 0.35,
                        ..SURFACE
                    }),
                    TEXT,
                    ACCENT,
                ),
                Pressed => (
                    Background::Color(iced::Color {
                        a: 0.55,
                        ..SURFACE
                    }),
                    TEXT,
                    ACCENT,
                ),
                Disabled => (
                    Background::Color(Color::TRANSPARENT),
                    Color { a: 0.45, ..MUTED },
                    BORDER,
                ),
            };
            button::Style {
                background: Some(background),
                text_color,
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: GROUP_RADIUS.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

pub fn section_card<'a>(
    title: &'static str,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    section_card_icon(None, title, body)
}

pub fn section_card_icon<'a>(
    icon_kind: Option<Icon>,
    title: &'static str,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    container(column_section(icon_kind, title, body))
        .padding(16)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(SURFACE.into()),
            border: iced::Border {
                color: BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn button_label(
    icon_kind: Option<Icon>,
    label: &'static str,
    color: Color,
) -> Element<'static, Message> {
    match icon_kind {
        Some(kind) => row![
            icon(kind, 14.0, color),
            text(label).size(13).color(color),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into(),
        None => text(label).size(13).color(color).into(),
    }
}

fn column_section<'a>(
    icon_kind: Option<Icon>,
    title: &'static str,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    let title_row: Element<'a, Message> = match icon_kind {
        Some(kind) => row![
            icon(kind, 16.0, TEXT),
            text(title).size(15).color(TEXT),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into(),
        None => text(title).size(15).color(TEXT).into(),
    };

    iced::widget::column![title_row, Space::with_height(12), body]
        .spacing(4)
        .into()
}

pub fn modal_card(
    title: &'static str,
    body: Element<'static, Message>,
    footer: Element<'static, Message>,
) -> Element<'static, Message> {
    container(
        iced::widget::column![
            text(title).size(18).color(TEXT),
            Space::with_height(12),
            body,
            Space::with_height(16),
            footer,
        ]
        .spacing(4)
        .width(Length::Fill),
    )
    .padding(20)
    .width(Length::Fixed(420.0))
    .style(|_| container::Style {
        background: Some(SURFACE.into()),
        border: iced::Border {
            color: BORDER,
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color {
                a: 0.45,
                ..iced::Color::BLACK
            },
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    })
    .into()
}

fn button_group(segments: std::vec::Vec<Element<'static, Message>>) -> Element<'static, Message> {
    let mut items: std::vec::Vec<Element<Message>> = std::vec::Vec::new();
    for (index, segment) in segments.into_iter().enumerate() {
        if index > 0 {
            items.push(group_divider());
        }
        items.push(segment);
    }

    container(
        row(items)
            .spacing(0)
            .align_y(iced::Alignment::Center),
    )
    .padding(1)
    .style(|_| container::Style {
        background: Some(SURFACE.into()),
        border: iced::Border {
            color: BORDER,
            width: 1.0,
            radius: GROUP_RADIUS.into(),
        },
        ..Default::default()
    })
    .into()
}

fn group_divider() -> Element<'static, Message> {
    container(Space::new(Length::Fixed(1.0), Length::Fill))
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(28.0))
        .style(|_| container::Style {
            background: Some(BORDER.into()),
            ..Default::default()
        })
        .into()
}

fn nav_tab_segment(
    icon_kind: Icon,
    label: &'static str,
    screen: Screen,
    active: Screen,
    position: GroupPosition,
) -> Element<'static, Message> {
    let selected = screen == active;
    let radius = segment_radius(position);
    let tab_color = if selected { TEXT } else { MUTED };

    button(button_label(Some(icon_kind), label, tab_color))
        .padding([8, 14])
        .on_press(Message::Navigate(screen))
        .style(move |_, status| {
            let (background, text_color) = nav_segment_colors(selected, status);
            segment_style(background, text_color, radius)
        })
        .into()
}

fn danger_button_segment(
    icon_kind: Icon,
    label: &'static str,
    message: Message,
    position: GroupPosition,
) -> Element<'static, Message> {
    let radius = segment_radius(position);

    button(button_label(Some(icon_kind), label, ACCENT))
        .padding([10, 16])
        .on_press(message)
        .style(move |_, status| {
            let (background, text_color) = danger_segment_colors(status);
            segment_style(background, text_color, radius)
        })
        .into()
}

fn icon_button_segment(
    icon_kind: Icon,
    message: Message,
    position: GroupPosition,
    icon_size: f32,
    padding: [u16; 2],
) -> Element<'static, Message> {
    let radius = segment_radius(position);

    button(icon(icon_kind, icon_size, MUTED))
        .padding(padding)
        .on_press(message)
        .style(move |_, status| {
            let (background, text_color) = icon_segment_colors(status);
            segment_style(background, text_color, radius)
        })
        .into()
}

fn segment_style(
    background: Background,
    text_color: Color,
    radius: iced::border::Radius,
) -> button::Style {
    button::Style {
        background: Some(background),
        text_color,
        border: iced::Border {
            radius,
            width: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn standalone_button_style(
    status: button::Status,
    fill: Color,
    label: Color,
    border_color: Color,
) -> button::Style {
    use button::Status::{Active, Disabled, Hovered, Pressed};

    let (background, text_color, border) = match status {
        Active => (
            Background::Color(fill),
            label,
            iced::Border {
                color: border_color,
                width: 1.0,
                radius: GROUP_RADIUS.into(),
            },
        ),
        Hovered => (
            Background::Color(lift(fill, 0.08)),
            label,
            iced::Border {
                color: lift(border_color, 0.12),
                width: 1.0,
                radius: GROUP_RADIUS.into(),
            },
        ),
        Pressed => (
            Background::Color(darken(fill, 0.08)),
            label,
            iced::Border {
                color: darken(border_color, 0.05),
                width: 1.0,
                radius: GROUP_RADIUS.into(),
            },
        ),
        Disabled => (
            Background::Color(Color { a: 0.35, ..fill }),
            Color { a: 0.45, ..label },
            iced::Border {
                color: BORDER,
                width: 1.0,
                radius: GROUP_RADIUS.into(),
            },
        ),
    };

    button::Style {
        background: Some(background),
        text_color,
        border,
        ..Default::default()
    }
}

fn nav_segment_colors(selected: bool, status: button::Status) -> (Background, Color) {
    use button::Status::{Active, Disabled, Hovered, Pressed};

    match (selected, status) {
        (true, Active) => (tint(ACCENT, 0.24), TEXT),
        (true, Hovered) => (tint(ACCENT, 0.38), TEXT),
        (true, Pressed) => (tint(ACCENT, 0.50), TEXT),
        (false, Active) => (Background::Color(Color::TRANSPARENT), MUTED),
        (false, Hovered) => (tint(SURFACE, 0.75), TEXT),
        (false, Pressed) => (tint(SURFACE, 0.95), TEXT),
        (_, Disabled) => (Background::Color(Color::TRANSPARENT), MUTED),
    }
}

fn danger_segment_colors(status: button::Status) -> (Background, Color) {
    use button::Status::{Active, Disabled, Hovered, Pressed};

    match status {
        Active => (Background::Color(Color::TRANSPARENT), ACCENT),
        Hovered => (tint(ACCENT, 0.18), TEXT),
        Pressed => (tint(ACCENT, 0.32), TEXT),
        Disabled => (Background::Color(Color::TRANSPARENT), Color { a: 0.45, ..ACCENT }),
    }
}

fn icon_segment_colors(status: button::Status) -> (Background, Color) {
    use button::Status::{Active, Disabled, Hovered, Pressed};

    match status {
        Active => (Background::Color(Color::TRANSPARENT), MUTED),
        Hovered => (tint(SURFACE, 0.85), TEXT),
        Pressed => (tint(SURFACE, 1.0), TEXT),
        Disabled => (Background::Color(Color::TRANSPARENT), Color { a: 0.45, ..MUTED }),
    }
}

fn group_position(index: usize, total: usize) -> GroupPosition {
    match total {
        0 => GroupPosition::Only,
        1 => GroupPosition::Only,
        _ if index == 0 => GroupPosition::First,
        _ if index + 1 == total => GroupPosition::Last,
        _ => GroupPosition::Middle,
    }
}

fn segment_radius(position: GroupPosition) -> iced::border::Radius {
    match position {
        GroupPosition::Only => SEGMENT_RADIUS.into(),
        GroupPosition::First => iced::border::Radius {
            top_left: SEGMENT_RADIUS,
            bottom_left: SEGMENT_RADIUS,
            top_right: 0.0,
            bottom_right: 0.0,
        },
        GroupPosition::Middle => 0.0.into(),
        GroupPosition::Last => iced::border::Radius {
            top_left: 0.0,
            bottom_left: 0.0,
            top_right: SEGMENT_RADIUS,
            bottom_right: SEGMENT_RADIUS,
        },
    }
}

fn tint(color: Color, alpha: f32) -> Background {
    Background::Color(Color { a: alpha, ..color })
}

fn lift(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r + amount).min(1.0),
        g: (color.g + amount).min(1.0),
        b: (color.b + amount).min(1.0),
        a: color.a,
    }
}

fn darken(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r - amount).max(0.0),
        g: (color.g - amount).max(0.0),
        b: (color.b - amount).max(0.0),
        a: color.a,
    }
}

pub fn modal_overlay<'a>(card: Element<'a, Message>) -> Element<'a, Message> {
    let dimmed = container(Space::new(Length::Fill, Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(
                Color {
                    a: 0.55,
                    ..Color::BLACK
                }
                .into(),
            ),
            ..Default::default()
        });

    stack![
        mouse_area(dimmed).on_press(Message::CloseOverlay),
        container(
            mouse_area(card).on_press(Message::OverlayClick),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
