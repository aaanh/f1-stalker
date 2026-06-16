use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::widget::scrollable;
use iced::{Element, Length};

use crate::db::default_db_path;
use crate::debug;
use crate::state::{AppState, CustomThemeField, Message, SettingsToggle};
use crate::ui::components::{danger_button_group, secondary_button, secondary_button_icon, section_card_icon};
use crate::ui::icons::Icon;
use crate::ui::fonts::MONO;
use crate::ui::layout::LayoutConfig;
use crate::ui::scroll::{scrollable_page, vertical_scroll};
use crate::ui::theme::{ThemePresetId, accent, border, muted, surface, text_color};

pub fn settings_page(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let db_path = default_db_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "Unavailable".into());
    let font_scale = layout.font_scale;

    let notice: Option<Element<Message>> = state.settings_notice.as_ref().map(|message| {
        container(text(message).size(layout.text(12)).color(accent()))
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
            text("Theme preset").size(layout.text(12)).color(muted()),
            Space::with_height(8),
            theme_buttons(state, layout),
            custom_theme_editor(state, layout),
            Space::with_height(12),
            font_scale_row(state, layout),
        ]
        .spacing(4)
        .into(),
        font_scale,
    );

    let preferences = section_card_icon(
        Some(Icon::Settings),
        "Preferences",
        column![
            toggle_row(
                "Include pre-season testing",
                state.settings.include_testing,
                SettingsToggle::IncludeTesting,
                layout,
            ),
            toggle_row(
                "Run in background when window closes",
                state.settings.background_on_close,
                SettingsToggle::BackgroundOnClose,
                layout,
            ),
        ]
        .spacing(8)
        .into(),
        font_scale,
    );

    let notifications = section_card_icon(
        Some(Icon::Alert),
        "Notifications",
        column![
            toggle_row(
                "Enable notifications",
                state.settings.notifications_enabled,
                SettingsToggle::NotificationsEnabled,
                layout,
            ),
            toggle_row(
                "Standings changes for pinned drivers",
                state.settings.notify_standings,
                SettingsToggle::NotifyStandings,
                layout,
            ),
            toggle_row(
                "Upcoming session reminders",
                state.settings.notify_sessions,
                SettingsToggle::NotifySessions,
                layout,
            ),
            text(format!(
                "Session reminders fire {} minutes before start.",
                state.settings.session_reminder_minutes
            ))
            .size(layout.text(11))
            .color(muted()),
        ]
        .spacing(8)
        .into(),
        font_scale,
    );

    let storage = section_card_icon(
        Some(Icon::Database),
        "Storage",
        column![
            text(format!("Database: {db_path}"))
                .size(layout.text(12))
                .color(muted()),
            Space::with_height(12),
            danger_button_group(&[
                (Icon::Trash, "Clear cache", Message::ClearCache),
                (Icon::Database, "Rebuild database", Message::RebuildDatabase),
            ]),
            Space::with_height(8),
            text("Clear cache removes API response blobs. Rebuild recreates the SQLite file while keeping settings and pinned drivers.")
                .size(layout.text(11))
                .color(muted()),
        ]
        .spacing(4)
        .into(),
        font_scale,
    );

    let logs = debug::entries();
    let log_body: Element<Message> = if logs.is_empty() {
        text("No log entries yet.")
            .size(layout.text(12))
            .color(muted())
            .into()
    } else {
        let lines: Vec<Element<Message>> = logs
            .iter()
            .map(|line| {
                text(line.clone())
                    .size(layout.text(11))
                    .font(MONO)
                    .color(text_color())
                    .into()
            })
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
                    .size(layout.text(11))
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
        font_scale,
    );

    let mut page = column![
        text("Settings").size(layout.text(24)).color(text_color()),
        Space::with_height(4),
        text("Maintenance and diagnostics")
            .size(layout.text(13))
            .color(muted()),
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

fn font_scale_row(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let pct = (state.settings.font_scale * 100.0).round() as u16;

    column![
        text("Text size").size(layout.text(12)).color(muted()),
        Space::with_height(8),
        row![
            secondary_button_icon(Some(Icon::Minus), "Smaller", Message::FontScaleDelta(-1)),
            container(text(format!("{pct}%")).size(layout.text(13)).font(MONO).color(text_color()))
                .padding([6, 12])
                .center_x(Length::Fill)
                .width(Length::Fixed(72.0)),
            secondary_button("Larger", Message::FontScaleDelta(1)),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        Space::with_height(4),
        text("Use Cmd/Ctrl + or - to adjust from anywhere.")
            .size(layout.text(11))
            .color(muted()),
    ]
    .spacing(0)
    .into()
}

fn theme_buttons(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let mut rows = row![].spacing(8);
    for preset in ThemePresetId::all_selectable() {
        let active = state.settings.theme_id == *preset;
        let label = preset.label();
        let button = button(
            text(label)
                .size(layout.text(12))
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

fn custom_theme_editor(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    if state.settings.theme_id != ThemePresetId::Custom {
        return Space::with_height(0).into();
    }

    let theme = &state.settings.custom_theme;
    column![
        Space::with_height(12),
        text("Custom theme").size(layout.text(12)).color(muted()),
        Space::with_height(8),
        custom_theme_field(
            "Background",
            &theme.background,
            CustomThemeField::Background,
            layout,
        ),
        Space::with_height(8),
        custom_theme_field("Surface", &theme.surface, CustomThemeField::Surface, layout),
        Space::with_height(8),
        custom_theme_field("Accent", &theme.accent, CustomThemeField::Accent, layout),
        Space::with_height(4),
        text("Use #RRGGBB hex colours. Changes apply immediately.")
            .size(layout.text(11))
            .color(muted()),
    ]
    .into()
}

fn custom_theme_field<'a>(
    label: &'a str,
    value: &'a str,
    field: CustomThemeField,
    layout: LayoutConfig,
) -> Element<'a, Message> {
    row![
        text(label)
            .size(layout.text(12))
            .color(text_color())
            .width(Length::Fixed(96.0)),
        text_input("RRGGBB", value)
            .on_input(move |input| Message::CustomThemeFieldChanged {
                field,
                value: input,
            })
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn toggle_row(
    label: &str,
    enabled: bool,
    toggle: SettingsToggle,
    layout: LayoutConfig,
) -> Element<'_, Message> {
    row![
        text(label).size(layout.text(13)).color(text_color()),
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
