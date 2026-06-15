use iced::widget::{button, container, mouse_area, row, text, Space};
use iced::{Element, Length};

use crate::assets::title_bar_brand;
use crate::state::{AppState, Message, WindowAction};
use crate::ui::components::{icon_button_group, nav_tab_group};
use crate::ui::fonts::MONO;
use crate::ui::icons::{icon, Icon};
use crate::ui::layout::scale_text;
use crate::ui::theme::{bg, border, muted, surface};

const TITLE_HEIGHT: f32 = 44.0;
const CONTROL_SIZE: f32 = 12.0;

const CLOSE: iced::Color = iced::Color::from_rgb(1.0, 0.373, 0.341);
const CLOSE_HOVER: iced::Color = iced::Color::from_rgb(0.878, 0.267, 0.227);
const MINIMIZE: iced::Color = iced::Color::from_rgb(0.996, 0.737, 0.180);
const MINIMIZE_HOVER: iced::Color = iced::Color::from_rgb(0.929, 0.663, 0.125);
const ZOOM: iced::Color = iced::Color::from_rgb(0.157, 0.784, 0.251);
const ZOOM_HOVER: iced::Color = iced::Color::from_rgb(0.125, 0.690, 0.220);

const CLOSE_GLYPH: iced::Color = iced::Color::from_rgba(0.24, 0.0, 0.0, 0.65);
const MINIMIZE_GLYPH: iced::Color = iced::Color::from_rgba(0.45, 0.24, 0.0, 0.75);
const ZOOM_GLYPH: iced::Color = iced::Color::from_rgba(0.0, 0.35, 0.08, 0.75);

#[derive(Debug, Clone, Copy)]
enum WindowControl {
    Close,
    Minimize,
    Zoom,
}

pub fn title_bar(state: &AppState) -> Element<'_, Message> {
    let drag_region = mouse_area(
        row![
            title_bar_brand(),
            Space::with_width(16),
            nav_tab_group(state.screen, state.settings.font_scale),
            Space::with_width(Length::Fill),
            status_chip(state),
        ]
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .on_press(Message::TitleBarPressed)
    .on_release(Message::TitleBarReleased)
    .on_move(|_| Message::TitleBarMoved);

    mouse_area(
        container(
            row![
                window_controls(state.title_bar_controls_hover),
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
            background: Some(bg().into()),
            border: iced::Border {
                color: border(),
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
        }),
    )
    .on_enter(Message::TitleBarControlsHover(true))
    .on_exit(Message::TitleBarControlsHover(false))
    .into()
}

fn window_controls(show_symbols: bool) -> Element<'static, Message> {
    row![
        window_control(
            WindowControl::Close,
            show_symbols,
            CLOSE,
            CLOSE_HOVER,
            Message::WindowAction(WindowAction::Close),
        ),
        window_control(
            WindowControl::Minimize,
            show_symbols,
            MINIMIZE,
            MINIMIZE_HOVER,
            Message::WindowAction(WindowAction::Minimize),
        ),
        window_control(
            WindowControl::Zoom,
            show_symbols,
            ZOOM,
            ZOOM_HOVER,
            Message::WindowAction(WindowAction::Fullscreen),
        ),
    ]
    .spacing(8)
    .into()
}

fn window_control(
    kind: WindowControl,
    show_symbols: bool,
    base: iced::Color,
    hover: iced::Color,
    message: Message,
) -> Element<'static, Message> {
    let label = control_label(kind, show_symbols);

    button(
        container(label)
            .width(Length::Fixed(CONTROL_SIZE))
            .height(Length::Fixed(CONTROL_SIZE))
            .center_x(Length::Fixed(CONTROL_SIZE))
            .center_y(Length::Fixed(CONTROL_SIZE)),
    )
    .padding(0)
    .on_press(message)
    .style(move |_, status| {
        let fill = match status {
            button::Status::Hovered | button::Status::Pressed => hover,
            _ => base,
        };

        button::Style {
            background: Some(iced::Background::Color(fill)),
            border: iced::Border {
                radius: (CONTROL_SIZE / 2.0).into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}

fn control_label(kind: WindowControl, show_symbols: bool) -> Element<'static, Message> {
    if !show_symbols {
        return Space::new(Length::Fill, Length::Fill).into();
    }

    let (icon_kind, color) = match kind {
        WindowControl::Close => (Icon::Close, CLOSE_GLYPH),
        WindowControl::Minimize => (Icon::Minus, MINIMIZE_GLYPH),
        WindowControl::Zoom => (Icon::Maximize, ZOOM_GLYPH),
    };

    icon(icon_kind, 8.0, color)
}

fn status_chip(state: &AppState) -> Element<'_, Message> {
    let label = if state.refreshing
        || state.drivers_refreshing
        || state.championship_refreshing
        || state.weekend_refreshing
    {
        "Refreshing…".to_string()
    } else {
        match &state.load {
            crate::state::LoadState::Loading => "Loading…".into(),
            crate::state::LoadState::Ready(loaded) | crate::state::LoadState::Error { cached: Some(loaded), .. } => {
                let prefix = if state.is_any_stale() {
                    "Cached · "
                } else {
                    "Updated "
                };
                format!(
                    "{prefix}{}",
                    crate::domain::format_fetched_at(loaded.data.fetched_at)
                )
            }
            crate::state::LoadState::Error { .. } => "Update failed".into(),
        }
    };

    container(text(label).size(scale_text(11, state.settings.font_scale)).font(MONO).color(muted()))
        .padding([4, 8])
        .style(|_| container::Style {
            background: Some(surface().into()),
            border: iced::Border {
                color: border(),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}
