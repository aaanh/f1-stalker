use iced::widget::{column, container, row, text, Space};
use iced::{Element, Length};

use crate::domain::{
    driver_display_name, format_gap_to_pole, format_grid_position, pinned_driver_views,
    QualiGridVisibility,
};
use crate::state::{AppState, Message};
use crate::ui::icons::{section_heading, Icon};
use crate::ui::layout::LayoutConfig;
use crate::ui::theme::{BORDER, FLAG_GREEN, MUTED, SURFACE, TEXT};

pub fn quali_grid_section(state: &AppState, layout: LayoutConfig) -> Option<Element<'_, Message>> {
    match state.quali_visibility() {
        QualiGridVisibility::Hidden => None,
        QualiGridVisibility::Pending => Some(pending_panel(layout)),
        QualiGridVisibility::Ready => Some(ready_panel(state, layout)),
    }
}

fn pending_panel(layout: LayoutConfig) -> Element<'static, Message> {
    panel_shell(
        column![
            section_heading(Icon::Trophy, "Qualifying grid", None),
            text("Grid not available yet")
                .size(layout.card_detail_size)
                .color(MUTED),
        ]
        .spacing(8)
        .width(Length::Fill),
    )
}

fn ready_panel(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let roster = state.drivers_roster().unwrap_or(&[]);
    let views = pinned_driver_views(&state.pinned_drivers, roster);
    let slots = state
        .weekend_data()
        .and_then(|data| data.quali_grid.as_ref())
        .map(|grid| grid.slots.as_slice())
        .unwrap_or(&[]);

    let mut rows = column![section_heading(Icon::Trophy, "Qualifying grid", None)].spacing(8);
    let mut rendered = 0usize;

    for view in views {
        let Some(slot) = slots
            .iter()
            .find(|slot| slot.driver_number == view.driver.driver_number)
        else {
            continue;
        };

        let position = format_grid_position(slot.position);
        let gap = format_gap_to_pole(slot.gap_to_pole_secs)
            .unwrap_or_else(|| "—".into());

        let name = driver_display_name(&view.driver).to_string();
        rendered += 1;
        rows = rows.push(
            row![
                text(name)
                    .size(13)
                    .color(TEXT),
                Space::with_width(Length::Fill),
                text(position).size(13).color(FLAG_GREEN),
                text(gap).size(12).color(MUTED),
            ]
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
        );
    }

    if rendered == 0 {
        return pending_panel(layout);
    }

    panel_shell(rows.width(Length::Fill))
}

fn panel_shell<'a>(content: iced::widget::Column<'a, Message>) -> Element<'a, Message> {
    container(content)
        .padding(12)
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
