use iced::widget::{column, container, row, text, Space};
use iced::{Element, Length};

use crate::data::grid::QualiGridData;
use crate::domain::{
    driver_display_name, format_gap_to_pole, format_grid_position, pinned_driver_views,
    QualiGridVisibility,
};
use crate::state::{AppState, Message};
use crate::ui::icons::{section_heading, Icon};
use crate::ui::layout::LayoutConfig;
use crate::ui::theme::{border, FLAG_GREEN, muted, surface, text_color};

pub fn sprint_grid_section(state: &AppState, layout: LayoutConfig) -> Option<Element<'_, Message>> {
    match state.sprint_visibility() {
        QualiGridVisibility::Hidden => None,
        QualiGridVisibility::Pending => Some(pending_panel_with_title(
            layout,
            "Sprint starting grid",
        )),
        QualiGridVisibility::Ready => Some(ready_panel_with_title(
            state,
            layout,
            "Sprint starting grid",
            |data| data.sprint_grid.as_ref(),
        )),
    }
}

pub fn quali_grid_section(state: &AppState, layout: LayoutConfig) -> Option<Element<'_, Message>> {
    match state.quali_visibility() {
        QualiGridVisibility::Hidden => None,
        QualiGridVisibility::Pending => {
            Some(pending_panel_with_title(layout, "Qualifying grid"))
        }
        QualiGridVisibility::Ready => Some(ready_panel_with_title(
            state,
            layout,
            "Qualifying grid",
            |data| data.quali_grid.as_ref(),
        )),
    }
}

fn pending_panel_with_title(layout: LayoutConfig, title: &'static str) -> Element<'static, Message> {
    panel_shell(
        column![
            section_heading(Icon::Trophy, title, None),
            text("Grid not available yet")
                .size(layout.card_detail_size)
                .color(muted()),
        ]
        .spacing(8)
        .width(Length::Fill),
    )
}

fn ready_panel_with_title<'a, F>(
    state: &'a AppState,
    layout: LayoutConfig,
    title: &'static str,
    grid_for: F,
) -> Element<'a, Message>
where
    F: Fn(&crate::data::WeekendDetailData) -> Option<&QualiGridData>,
{
    let roster = state.drivers_roster().unwrap_or(&[]);
    let views = pinned_driver_views(&state.pinned_drivers, roster);
    let slots = state
        .weekend_data()
        .and_then(grid_for)
        .map(|grid| grid.slots.as_slice())
        .unwrap_or(&[]);

    let mut rows = column![section_heading(Icon::Trophy, title, None)].spacing(8);
    let mut rendered = 0usize;

    for view in views {
        let Some(slot) = slots
            .iter()
            .find(|slot| slot.driver_number == view.driver.driver_number)
        else {
            continue;
        };

        let position = format_grid_position(slot.position);
        let gap = format_gap_to_pole(slot.gap_to_pole_secs).unwrap_or_else(|| "—".into());
        let name = driver_display_name(&view.driver).to_string();
        rendered += 1;
        rows = rows.push(
            row![
                text(name).size(13).color(text_color()),
                Space::with_width(Length::Fill),
                text(position).size(13).color(FLAG_GREEN),
                text(gap).size(12).color(muted()),
            ]
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
        );
    }

    if rendered == 0 {
        return pending_panel_with_title(layout, title);
    }

    panel_shell(rows.width(Length::Fill))
}

fn panel_shell<'a>(content: iced::widget::Column<'a, Message>) -> Element<'a, Message> {
    container(content)
        .padding(12)
        .width(Length::Fill)
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
