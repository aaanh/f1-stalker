use iced::widget::{column, container, row, text, Space};
use iced::{Element, Length};

use crate::assets::about_brand;
use crate::state::Message;
use crate::ui::components::{action_button, modal_overlay};
use crate::ui::theme::{border, muted, surface, text_color};

pub fn about_overlay() -> Element<'static, Message> {
    let card = container(
        column![
            about_brand(),
            Space::with_height(8),
            text(format!("Version {}", env!("CARGO_PKG_VERSION")))
                .size(13)
                .color(muted()),
            Space::with_height(12),
            text("Native desktop dashboard for the current Formula 1 season.")
                .size(13)
                .color(text_color()),
            Space::with_height(8),
            text("Data via OpenF1 (historical, approx. 24h delay). Forecasts via Open-Meteo.")
                .size(12)
                .color(muted()),
            Space::with_height(8),
            text("Built with Rust and Iced.")
                .size(12)
                .color(muted()),
            Space::with_height(16),
            row![
                Space::with_width(Length::Fill),
                action_button("Close", Message::CloseOverlay),
            ],
        ]
        .width(Length::Fill),
    )
    .padding(24)
    .width(Length::Fixed(400.0))
    .style(|_| container::Style {
        background: Some(surface().into()),
        border: iced::Border {
            color: border(),
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color {
                a: 0.5,
                ..iced::Color::BLACK
            },
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    });

    modal_overlay(card.into())
}
