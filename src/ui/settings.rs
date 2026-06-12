use iced::widget::{button, column, container, row, text, Space};
use iced::widget::scrollable;
use iced::{Element, Length};

use crate::db::default_db_path;
use crate::debug;
use crate::state::{AppState, Message, SettingsToggle};
use crate::ui::components::{danger_button_group, secondary_button_icon, section_card_icon};
use crate::ui::icons::Icon;
use crate::ui::fonts::MONO;
use crate::ui::layout::LayoutConfig;
use crate::ui::scroll::{scrollable_page, vertical_scroll};
use crate::ui::theme::{ThemePresetId, accent, border, muted, surface, text_color};

pub fn settings_page(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let db_path = default_db_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "Unavailable".into());

    let notice: Option<Element<Message>> = state.settings_notice.as_ref().map(|message| {
        container(text(message).size(12).color(accent()))
            .padding([8, 12])
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(surface().into()),
                border: iced::Border {
                    color: accent(),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
    });

    let appearance = section_card_icon(
        Some(Icon::Circle),
        "Appearance",
        column![
            text("Theme preset").size(12).color(muted()),
            Space::with_height(8),
            theme_buttons(state),
        ]
        .spacing(4)
        .into(),
    );

    let preferences = section_card_icon(
        Some(Icon::Settings),
        "Preferences",
        column![
            toggle_row(
                "Include pre-season testing",
                state.settings.include_testing,
                SettingsToggle::IncludeTesting,
            ),
            toggle_row(
                "Run in background when window closes",
                state.settings.background_on_close,
                SettingsToggle::BackgroundOnClose,
            ),
        ]
        .spacing(8)
        .into(),
    );

    let notifications = section_card_icon(
        Some(Icon::Alert),
        "Notifications",
        column![
            toggle_row(
                "Enable notifications",
                state.settings.notifications_enabled,
                SettingsToggle::NotificationsEnabled,
            ),
            toggle_row(
                "Standings changes for pinned drivers",
                state.settings.notify_standings,
                SettingsToggle::NotifyStandings,
            ),
            toggle_row(
                "Upcoming session reminders",
                state.settings.notify_sessions,
                SettingsToggle::NotifySessions,
            ),
            text(format!(
                "Session reminders fire {} minutes before start.",
                state.settings.session_reminder_minutes
            ))
            .size(11)
            .color(muted()),
        ]
        .spacing(8)
        .into(),
    );

    let storage = section_card_icon(
        Some(Icon::Database),
        "Storage",
        column![
            text(format!("Database: {db_path}")).size(12).color(muted()),
            Space::with_height(12),
            danger_button_group(&[
                (Icon::Trash, "Clear cache", Message::ClearCache),
                (Icon::Database, "Rebuild database", Message::RebuildDatabase),
            ]),
            Space::with_height(8),
            text("Clear cache removes API response blobs. Rebuild recreates the SQLite file while keeping settings and pinned drivers.")
                .size(11)
                .color(muted()),
        ]
        .spacing(4)
        .into(),
    );

    let logs = debug::entries();
    let log_body: Element<Message> = if logs.is_empty() {
        text("No log entries yet.").size(12).color(muted()).into()
    } else {
        let lines: Vec<Element<Message>> = logs
            .iter()
            .map(|line| text(line.clone()).size(11).font(MONO).color(text_color()).into())
            .collect();
        vertical_scroll(
            column(lines).spacing(4).width(Length::Fill).into(),
            state.scrollbar_visible.visible,
        )
            .height(Length::Fixed(280.0))
            .width(Length::Fill)
            .into()
    };

    let debug_section = section_card_icon(
        Some(Icon::FileText),
        "Debug log",
        column![
            row![
                text("Recent application events (in-memory, last 500 lines).")
                    .size(11)
                    .color(muted()),
                Space::with_width(Length::Fill),
                secondary_button_icon(Some(Icon::Copy), "Copy log", Message::CopyDebugLog),
            ]
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
            Space::with_height(8),
            log_body,
        ]
        .spacing(4)
        .into(),
    );

    let mut page = column![
        text("Settings").size(24).color(text_color()),
        Space::with_height(4),
        text("Maintenance and diagnostics").size(13).color(muted()),
        Space::with_height(16),
    ]
    .spacing(0)
    .width(Length::Fill);

    if let Some(notice) = notice {
        page = page.push(notice).push(Space::with_height(12));
    }

    page = page
        .push(appearance)
        .push(Space::with_height(12))
        .push(preferences)
        .push(Space::with_height(12))
        .push(notifications)
        .push(Space::with_height(12))
        .push(storage)
        .push(Space::with_height(12))
        .push(debug_section)
        .height(Length::Shrink);

    let scroll_content = container(page)
        .padding(layout.padding)
        .width(Length::Fill)
        .height(Length::Shrink);

    scrollable_page(
        scrollable::Id::new("settings-scroll"),
        scroll_content.into(),
        state.scrollbar_visible.visible,
    )
}

fn theme_buttons(state: &AppState) -> Element<'_, Message> {
    let mut rows = row![].spacing(8);
    for preset in ThemePresetId::all_selectable() {
        let active = state.settings.theme_id == *preset;
        let label = preset.label();
        let button = button(
            text(label)
                .size(12)
                .color(if active { text_color() } else { muted() }),
        )
        .on_press(Message::ThemeSelected(*preset))
        .padding([6, 10])
        .style(move |_, status| {
            use iced::widget::button::{self, Status};
            let (background, border_color) = if active {
                (
                    iced::Background::Color(iced::Color { a: 0.22, ..accent() }),
                    accent(),
                )
            } else {
                match status {
                    Status::Active => (
                        iced::Background::Color(iced::Color::TRANSPARENT),
                        border(),
                    ),
                    Status::Hovered => (
                        iced::Background::Color(iced::Color { a: 0.35, ..surface() }),
                        accent(),
                    ),
                    Status::Pressed => (
                        iced::Background::Color(iced::Color { a: 0.55, ..surface() }),
                        accent(),
                    ),
                    Status::Disabled => (
                        iced::Background::Color(iced::Color::TRANSPARENT),
                        border(),
                    ),
                }
            };
            button::Style {
                background: Some(background),
                text_color: text_color(),
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        });
        rows = rows.push(button);
    }
    container(rows).width(Length::Fill).into()
}

fn toggle_row(label: &str, enabled: bool, toggle: SettingsToggle) -> Element<'_, Message> {
    row![
        text(label).size(13).color(text_color()),
        Space::with_width(Length::Fill),
        secondary_button_icon(
            if enabled { Some(Icon::Check) } else { None },
            if enabled { "On" } else { "Off" },
            Message::SettingsToggled(toggle),
        ),
    ]
    .align_y(iced::Alignment::Center)
    .width(Length::Fill)
    .into()
}
