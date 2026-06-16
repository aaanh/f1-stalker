use iced::widget::{column, container, row, text, Space};
use iced::{alignment, Element, Length};

use crate::domain::{
    build_standings, ChampionshipTab, ChartMode, PositionChange, StandingRow,
};
use crate::state::{AppState, ChampionshipLoadState, Message};
use crate::ui::championship_charts::tab_button_inner;
use crate::ui::components::secondary_button_icon;
use crate::ui::fonts::MONO;
use crate::ui::icons::{icon, section_heading, subtitle_text, Icon};
use crate::ui::layout::{scale_text, LayoutConfig};
use crate::ui::theme::{accent, border, muted, surface, text_color};

pub fn standings_section(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let scale = layout.font_scale;
    let tab = state.settings.standings_tab;
    let mode = state.settings.standings_mode;

    let mode_tabs = row![
        standings_mode_button("Championship", ChartMode::Championship, mode, scale),
        standings_mode_button("Latest race", ChartMode::RaceStanding, mode, scale),
    ]
    .spacing(8);

    let subject_tabs = row![
        standings_tab_button("Drivers", ChampionshipTab::Drivers, tab, scale),
        standings_tab_button("Constructors", ChampionshipTab::Constructors, tab, scale),
    ]
    .spacing(8);

    let subtitle = match mode {
        ChartMode::Championship => "Full championship standings after the latest Grand Prix",
        ChartMode::RaceStanding => "Latest race results with starting grid positions",
    };

    let body = match &state.championship {
        ChampionshipLoadState::Loading => text("Loading standings…")
            .size(scale_text(13, scale))
            .color(muted())
            .into(),
        ChampionshipLoadState::Error { message, cached: None } => column![
            text("Could not load standings").size(scale_text(14, scale)),
            text(message).size(scale_text(12, scale)).color(muted()),
            Space::with_height(8),
            secondary_button_icon(Some(Icon::Refresh), "Retry", Message::Refresh),
        ]
        .spacing(6)
        .into(),
        ChampionshipLoadState::Ready(_) | ChampionshipLoadState::Error { cached: Some(_), .. } => {
            standings_body(state, layout, tab, mode)
        }
    };

    container(
        column![
            row![
                section_heading(
                    Icon::Trophy,
                    "Standings",
                    Some(subtitle_text(subtitle)),
                ),
                Space::with_width(Length::Fill),
                column![mode_tabs, subject_tabs]
                    .spacing(8)
                    .align_x(iced::Alignment::End),
            ]
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
            Space::with_height(12),
            body,
        ]
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

fn standings_body(
    state: &AppState,
    layout: LayoutConfig,
    tab: ChampionshipTab,
    mode: ChartMode,
) -> Element<'_, Message> {
    let Some(data) = state.championship_data() else {
        return text("Standings unavailable")
            .size(layout.text(13))
            .color(muted())
            .into();
    };

    let roster = state.drivers_roster().unwrap_or(&[]);
    let rows = build_standings(&data.rounds, roster, tab, mode);

    if rows.is_empty() {
        return text("No standings data yet")
            .size(layout.text(13))
            .color(muted())
            .into();
    }

    standings_grid(rows, layout, mode)
}

fn standings_grid(
    rows: Vec<StandingRow>,
    layout: LayoutConfig,
    mode: ChartMode,
) -> Element<'static, Message> {
    let columns = layout.pinned_grid_columns.max(1);

    if columns == 1 {
        return column(
            rows.into_iter()
                .map(|row| standing_card(row, layout, mode))
                .collect::<Vec<_>>(),
        )
        .spacing(8)
        .width(Length::Fill)
        .into();
    }

    let row_count = rows.len().div_ceil(columns);
    let mut grid = column![].spacing(8).width(Length::Fill);

    for row_idx in 0..row_count {
        let mut cells: Vec<Element<'static, Message>> = Vec::new();

        for col_idx in 0..columns {
            let index = col_idx * row_count + row_idx;
            if index < rows.len() {
                cells.push(
                    container(standing_card(rows[index].clone(), layout, mode))
                        .width(Length::FillPortion(1))
                        .height(Length::Shrink)
                        .into(),
                );
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
                .spacing(8)
                .width(Length::Fill)
                .align_y(iced::Alignment::Start),
        );
    }

    grid.into()
}

fn standing_card(
    row: StandingRow,
    layout: LayoutConfig,
    mode: ChartMode,
) -> Element<'static, Message> {
    let StandingRow {
        position_label,
        label,
        code,
        accent: accent_colour,
        grid_position,
        position_change,
        ..
    } = row;

    let mut details = column![
        text(label).size(layout.text(15)).color(text_color()),
        text(code).size(layout.text(14)).color(accent_colour),
    ]
    .spacing(2)
    .width(Length::Fill);

    if mode == ChartMode::RaceStanding {
        if let Some(grid_position) = grid_position {
            details = details.push(
                text(format!("Grid P{grid_position}"))
                    .size(layout.text(12))
                    .color(muted()),
            );
        }
    }

    container(
        row![
            rank_block(position_label, position_change, layout),
            details,
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill),
    )
    .padding([10, 12])
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

fn rank_block(
    position_label: String,
    position_change: Option<PositionChange>,
    layout: LayoutConfig,
) -> Element<'static, Message> {
    let (up_color, down_color) = match position_change {
        Some(PositionChange::Improved) => (accent(), muted()),
        Some(PositionChange::Worsened) => (muted(), accent()),
        Some(PositionChange::Unchanged) | None => (muted(), muted()),
    };

    let arrow_size = layout.standings_arrow_size;

    container(
        row![
            container(
                row![
                    icon(Icon::ChevronUp, arrow_size, up_color),
                    icon(Icon::ChevronDown, arrow_size, down_color),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
            .width(Length::Fixed(layout.standings_arrow_column_width))
            .align_x(alignment::Horizontal::Center),
            container(
                text(position_label)
                    .size(layout.standings_position_size)
                    .font(MONO)
                    .color(text_color()),
            )
            .width(Length::Fixed(layout.standings_position_column_width))
            .align_x(alignment::Horizontal::Right),
        ]
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fixed(layout.standings_rank_block_width()))
    .align_x(alignment::Horizontal::Left)
    .into()
}

fn standings_mode_button(
    label: &'static str,
    mode: ChartMode,
    active: ChartMode,
    scale: f32,
) -> Element<'static, Message> {
    tab_button_inner(
        label,
        mode == active,
        Message::StandingsModeSelected(mode),
        scale,
    )
}

fn standings_tab_button(
    label: &'static str,
    tab: ChampionshipTab,
    active: ChampionshipTab,
    scale: f32,
) -> Element<'static, Message> {
    tab_button_inner(
        label,
        tab == active,
        Message::StandingsTabSelected(tab),
        scale,
    )
}
