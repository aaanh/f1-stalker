use iced::widget::{column, container, row, text, Space};
use iced::widget::scrollable;
use iced::{Element, Length};

use crate::db::default_db_path;
use crate::debug;
use crate::state::{AppState, Message};
use crate::ui::components::{danger_button_group, secondary_button_icon, section_card_icon};
use crate::ui::icons::Icon;
use crate::ui::fonts::MONO;
use crate::ui::layout::LayoutConfig;
use crate::ui::scroll::{scrollable_page, vertical_scroll};
use crate::ui::theme::{ACCENT, MUTED, SURFACE, TEXT};

pub fn settings_page(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let db_path = default_db_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "Unavailable".into());

    let notice: Option<Element<Message>> = state.settings_notice.as_ref().map(|message| {
        container(text(message).size(12).color(ACCENT))
            .padding([8, 12])
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(SURFACE.into()),
                border: iced::Border {
                    color: ACCENT,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
    });

    let storage = section_card_icon(
        Some(Icon::Database),
        "Storage",
        column![
            text(format!("Database: {db_path}")).size(12).color(MUTED),
            Space::with_height(12),
            danger_button_group(&[
                (Icon::Trash, "Clear cache", Message::ClearCache),
                (Icon::Database, "Rebuild database", Message::RebuildDatabase),
            ]),
            Space::with_height(8),
            text("Clear cache removes API response blobs. Rebuild recreates the SQLite file while keeping settings and pinned drivers.")
                .size(11)
                .color(MUTED),
        ]
        .spacing(4)
        .into(),
    );

    let logs = debug::entries();
    let log_body: Element<Message> = if logs.is_empty() {
        text("No log entries yet.").size(12).color(MUTED).into()
    } else {
        let lines: Vec<Element<Message>> = logs
            .iter()
            .map(|line| text(line.clone()).size(11).font(MONO).color(TEXT).into())
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
                    .color(MUTED),
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
        text("Settings").size(24).color(TEXT),
        Space::with_height(4),
        text("Maintenance and diagnostics").size(13).color(MUTED),
        Space::with_height(16),
    ]
    .spacing(0)
    .width(Length::Fill);

    if let Some(notice) = notice {
        page = page.push(notice).push(Space::with_height(12));
    }

    page = page
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
