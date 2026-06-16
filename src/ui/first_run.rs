use iced::widget::{button, column, container, pick_list, row, text, Space};
use iced::{Element, Length};

use crate::db::schema::DEFAULT_TIMEZONE;
use crate::state::{AppState, FirstRunStep, Message};
use crate::ui::components::secondary_button;
use crate::ui::theme::{border, muted, surface, text_color};

const TIMEZONE_OPTIONS: &[(&str, &str)] = &[
    ("system", "System default"),
    ("UTC", "UTC"),
    ("Europe/London", "London"),
    ("Europe/Paris", "Paris"),
    ("Europe/Berlin", "Berlin"),
    ("America/New_York", "New York"),
    ("America/Chicago", "Chicago"),
    ("America/Denver", "Denver"),
    ("America/Los_Angeles", "Los Angeles"),
    ("Asia/Tokyo", "Tokyo"),
    ("Asia/Singapore", "Singapore"),
    ("Australia/Sydney", "Sydney"),
];

pub fn first_run_overlay(state: &AppState) -> Element<'_, Message> {
    let step = state.first_run_step;
    let body = match step {
        FirstRunStep::Welcome => welcome_step(),
        FirstRunStep::Timezone => timezone_step(state),
        FirstRunStep::Pins => pins_step(state),
        FirstRunStep::Done => finish_step(state),
    };

    container(
        column![
            text("Welcome to F1 Stalker").size(24).color(text_color()),
            Space::with_height(8),
            text(step_label(step))
                .size(12)
                .color(muted()),
            Space::with_height(16),
            body,
            Space::with_height(20),
            wizard_nav(step, state),
        ]
        .spacing(4)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center),
    )
    .padding(24)
    .width(Length::Fixed(460.0))
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

fn step_label(step: FirstRunStep) -> &'static str {
    match step {
        FirstRunStep::Welcome => "Step 1 of 4 · Introduction",
        FirstRunStep::Timezone => "Step 2 of 4 · Timezone",
        FirstRunStep::Pins => "Step 3 of 4 · Driver pins",
        FirstRunStep::Done => "Step 4 of 4 · Ready",
    }
}

fn welcome_step() -> Element<'static, Message> {
    column![
        text("Historical OpenF1 data only (approx. 24h delay).")
            .size(13)
            .color(muted()),
        Space::with_height(12),
        text("This quick setup confirms your timezone, lets you pin drivers, and explains the data delay before the dashboard loads.")
            .size(13)
            .color(text_color()),
    ]
    .spacing(4)
    .into()
}

fn timezone_step(state: &AppState) -> Element<'static, Message> {
    let selected = timezone_label(&state.settings.timezone);
    let options: Vec<String> = TIMEZONE_OPTIONS
        .iter()
        .map(|(_, label)| (*label).to_string())
        .collect();

    column![
        text("Choose the timezone used for session times.")
            .size(13)
            .color(text_color()),
        Space::with_height(12),
        pick_list(options, Some(selected), |label| {
            let key = TIMEZONE_OPTIONS
                .iter()
                .find_map(|(key, value)| (*value == label).then_some(*key))
                .unwrap_or(DEFAULT_TIMEZONE);
            Message::FirstRunTimezoneSelected(key.to_string())
        }),
    ]
    .spacing(4)
    .align_x(iced::Alignment::Center)
    .into()
}

fn pins_step(state: &AppState) -> Element<'static, Message> {
    let count = state.pinned_drivers.len();
    column![
        text("Pin the drivers you want to follow on the dashboard.")
            .size(13)
            .color(text_color()),
        Space::with_height(8),
        text(format!("{count} drivers pinned"))
            .size(13)
            .color(muted()),
        Space::with_height(12),
        button(text("Choose drivers").size(14))
            .on_press(Message::FirstRunOpenPinPicker)
            .padding([10, 16]),
    ]
    .spacing(4)
    .align_x(iced::Alignment::Center)
    .into()
}

fn finish_step(state: &AppState) -> Element<'static, Message> {
    column![
        text(format!("Timezone: {}", timezone_label(&state.settings.timezone)))
            .size(13)
            .color(text_color()),
        Space::with_height(8),
        text(format!("Pinned drivers: {}", state.pinned_drivers.len()))
            .size(13)
            .color(text_color()),
        Space::with_height(8),
        text("You can change these later in Settings.")
            .size(13)
            .color(muted()),
    ]
    .spacing(4)
    .into()
}

fn wizard_nav(step: FirstRunStep, state: &AppState) -> Element<'static, Message> {
    let mut actions = row![].spacing(8);

    if step != FirstRunStep::Welcome {
        actions = actions.push(secondary_button("Back", Message::FirstRunBack));
    }

    match step {
        FirstRunStep::Done => {
            actions = actions.push(
                button(text("Get started").size(14))
                    .on_press(Message::CompleteFirstRun)
                    .padding([10, 16]),
            );
        }
        _ => {
            actions = actions.push(
                button(text("Continue").size(14))
                    .on_press(Message::FirstRunNext)
                    .padding([10, 16]),
            );
        }
    }

    if step == FirstRunStep::Pins && state.pinned_drivers.is_empty() {
        actions = actions.push(secondary_button("Skip for now", Message::FirstRunNext));
    }

    actions.align_y(iced::Alignment::Center).into()
}

fn timezone_label(value: &str) -> String {
    TIMEZONE_OPTIONS
        .iter()
        .find_map(|(key, label)| (*key == value).then_some((*label).to_string()))
        .unwrap_or_else(|| value.to_string())
}
