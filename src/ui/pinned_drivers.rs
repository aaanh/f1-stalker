use iced::widget::{column, container, row, text, Space};
use iced::{Element, Length};

use crate::domain::{pinned_driver_views, MAX_PINNED_DRIVERS};
use crate::state::{AppState, DriversLoadState, Message};
use crate::ui::components::secondary_button_icon;
use crate::ui::icons::{section_heading, subtitle_text, Icon};
use crate::ui::driver_card::pinned_driver_card;
use crate::ui::quali_grid::{quali_grid_section, sprint_grid_section};
use crate::ui::layout::LayoutConfig;
use crate::ui::theme::{border, muted, surface};

pub fn pinned_drivers_section(
    state: &AppState,
    layout: LayoutConfig,
) -> Element<'_, Message> {
    let subtitle = subtitle_text(format!(
        "{}/{} drivers pinned",
        state.pinned_drivers.len(),
        MAX_PINNED_DRIVERS
    ));
    let add_control: Element<Message> = if state.can_add_pin() {
        secondary_button_icon(Some(Icon::UserPlus), "Add driver", Message::OpenDriverPicker)
    } else {
        container(text("Pin limit reached").size(13).color(muted()))
            .padding([8, 14])
            .style(|_| container::Style {
                border: iced::Border {
                    color: border(),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
    };

    let header: Element<Message> = if layout.stack_header {
        column![
            section_heading(Icon::Users, "Pinned drivers", Some(subtitle)),
            add_control,
        ]
        .spacing(10)
        .width(Length::Fill)
        .into()
    } else {
        row![
            section_heading(Icon::Users, "Pinned drivers", Some(subtitle)),
            Space::with_width(Length::Fill),
            add_control,
        ]
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
        .into()
    };

    let mut body = column![].spacing(12).width(Length::Fill);
    if let Some(notice) = drivers_notice(state) {
        body = body.push(notice);
    }
    if state.pinned_drivers.is_empty() {
        body = body.push(empty_state());
    } else {
        body = body.push(pinned_grid(state, layout));
        if let Some(quali) = quali_grid_section(state, layout) {
            body = body.push(quali);
        }
        if let Some(sprint) = sprint_grid_section(state, layout) {
            body = body.push(sprint);
        }
    }

    container(
        column![header, Space::with_height(12), body]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Shrink),
    )
    .padding(16)
    .width(Length::Fill)
    .height(Length::Shrink)
    .style(|_| container::Style {
        background: Some(surface().into()),
        border: iced::Border {
            color: border(),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn empty_state() -> Element<'static, Message> {
    text("Pin up to 6 drivers to follow their season progress.")
        .size(13)
        .color(muted())
        .into()
}

fn drivers_notice(state: &AppState) -> Option<Element<'_, Message>> {
    match &state.drivers {
        DriversLoadState::Error { message, cached: None } => Some(
            column![
                text("Could not load driver roster").size(13),
                text(message).size(12).color(muted()),
                secondary_button_icon(Some(Icon::Refresh), "Retry", Message::Refresh),
            ]
            .spacing(6)
            .into(),
        ),
        DriversLoadState::Ready(loaded) if loaded.stale => Some(
            text("Driver roster is cached · refresh to update.")
                .size(12)
                .color(muted())
                .into(),
        ),
        _ => None,
    }
}

fn pinned_grid(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let roster = state.drivers_roster().unwrap_or(&[]);
    let views = pinned_driver_views(&state.pinned_drivers, roster);

    if views.is_empty() && !matches!(state.drivers, DriversLoadState::Loading) {
        return column![
            text("Pinned drivers are saved, but roster data is unavailable.")
                .size(12)
                .color(muted()),
            Space::with_height(8),
            secondary_button_icon(Some(Icon::Refresh), "Reload drivers", Message::Refresh),
        ]
        .into();
    }

    if views.is_empty() {
        return text("Loading driver details...")
            .size(12)
            .color(muted())
            .into();
    }

    let columns = layout.pinned_grid_columns.max(1);
    let total = views.len();
    let cards: Vec<Element<Message>> = views
        .into_iter()
        .enumerate()
        .map(|(index, view)| {
            pinned_driver_card(state, &view.driver, index, total, layout)
        })
        .collect();

    if columns == 1 {
        return column(cards).spacing(12).width(Length::Fill).into();
    }

    let mut grid = column![].spacing(12).width(Length::Fill);
    let mut iter = cards.into_iter();

    loop {
        let mut cells: Vec<Element<Message>> = Vec::new();
        for _ in 0..columns {
            match iter.next() {
                Some(card) => cells.push(
                    container(card)
                        .width(Length::FillPortion(1))
                        .height(Length::Shrink)
                        .into(),
                ),
                None => break,
            }
        }

        if cells.is_empty() {
            break;
        }

        while cells.len() < columns {
            cells.push(Space::new(Length::FillPortion(1), Length::Shrink).into());
        }

        grid = grid.push(
            row(cells)
                .spacing(12)
                .width(Length::Fill)
                .align_y(iced::Alignment::Start),
        );
    }

    grid.into()
}
