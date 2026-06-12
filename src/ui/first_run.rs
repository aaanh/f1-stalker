use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length};

use crate::state::{AppState, Message};
use crate::ui::theme::{border, muted, surface, text_color};

pub fn first_run_overlay(state: &AppState) -> Element<'_, Message> {
    container(
        column![
            text("Welcome to F1 Stalker").size(24).color(text_color()),
            Space::with_height(8),
            text("Historical OpenF1 data only (approx. 24h delay).")
                .size(13)
                .color(muted()),
            Space::with_height(16),
            text(format!("Timezone: {}", state.settings.timezone))
                .size(13)
                .color(text_color()),
            Space::with_height(8),
            text(format!(
                "Pinned drivers: {}",
                state.pinned_drivers.len()
            ))
            .size(13)
            .color(text_color()),
            Space::with_height(20),
            row![
                button(text("Get started").size(14))
                    .on_press(Message::CompleteFirstRun)
                    .padding([10, 16]),
            ],
        ]
        .spacing(4)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center),
    )
    .padding(24)
    .width(Length::Fixed(420.0))
    .style(|_| container::Style {
        background: Some(surface().into()),
        border: iced::Border {
            color: border(),
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color {
                a: 0.35,
                ..iced::Color::BLACK
            },
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    })
    .into()
}

pub fn should_show_first_run(state: &AppState) -> bool {
    state.show_first_run && !state.settings.first_run_complete
}
