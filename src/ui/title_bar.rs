use iced::widget::{button, container, mouse_area, row, text, Space};
use iced::{Element, Length};

use crate::assets::title_bar_brand;
use crate::state::{AppState, Message, WindowAction};
use crate::ui::components::{icon_button_group, nav_tab_group};
use crate::ui::fonts::MONO;
use crate::ui::icons::Icon;
use crate::ui::theme::{ACCENT, BG, BORDER, MUTED, SURFACE};

const TITLE_HEIGHT: f32 = 44.0;
pub fn title_bar(state: &AppState) -> Element<'_, Message> {
    let drag_region = mouse_area(
        row![
            title_bar_brand(),
            Space::with_width(16),
            nav_tab_group(state.screen),
            Space::with_width(Length::Fill),
            status_chip(state),
        ]
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .on_press(Message::WindowAction(WindowAction::Drag));

    container(
        row![
            window_controls(),
            Space::with_width(12),
            drag_region,
            Space::with_width(8),
            icon_button_group(&[
                (Icon::Refresh, Message::Refresh),
                (Icon::Help, Message::OpenAbout),
            ]),
        ]
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .height(Length::Fixed(TITLE_HEIGHT))
    .width(Length::Fill)
    .padding([0, 12])
    .style(|_| container::Style {
        background: Some(BG.into()),
        border: iced::Border {
            color: BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color {
                a: 0.25,
                ..iced::Color::BLACK
            },
            offset: iced::Vector::new(0.0, 1.0),
            blur_radius: 0.0,
        },
        ..Default::default()
    })
    .into()
}

fn window_controls() -> Element<'static, Message> {
    row![
        window_dot(ACCENT, Message::WindowAction(WindowAction::Close)),
        window_dot(
            iced::Color::from_rgb(0.95, 0.75, 0.15),
            Message::WindowAction(WindowAction::Minimize),
        ),
        window_dot(
            iced::Color::from_rgb(0.15, 0.78, 0.35),
            Message::WindowAction(WindowAction::Maximize),
        ),
    ]
    .spacing(8)
    .into()
}

fn window_dot(color: iced::Color, message: Message) -> Element<'static, Message> {
    button(
        container(Space::new(Length::Fixed(10.0), Length::Fixed(10.0)))
            .width(Length::Fixed(12.0))
            .height(Length::Fixed(12.0))
            .style(move |_| container::Style {
                background: Some(color.into()),
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    )
    .padding(0)
    .on_press(message)
    .style(|_, _| button::Style {
        background: None,
        ..Default::default()
    })
    .into()
}

fn status_chip(state: &AppState) -> Element<'_, Message> {
    let label = if state.refreshing {
        "Refreshing…".to_string()
    } else {
        match &state.load {
            crate::state::LoadState::Loading => "Loading…".into(),
            crate::state::LoadState::Ready(loaded) => {
                let prefix = if loaded.stale { "Cached · " } else { "Updated " };
                format!(
                    "{prefix}{}",
                    crate::domain::format_fetched_at(loaded.data.fetched_at)
                )
            }
            crate::state::LoadState::Error { .. } => "Update failed".into(),
        }
    };

    container(text(label).size(11).font(MONO).color(MUTED))
        .padding([4, 8])
        .style(|_| container::Style {
            background: Some(SURFACE.into()),
            border: iced::Border {
                color: BORDER,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}
