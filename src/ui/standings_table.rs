use iced::widget::{column, container, row, text, Space};
use iced::{Element, Length};

use crate::domain::{build_standings, ChampionshipTab, ChartMode, StandingRow};
use crate::state::{AppState, ChampionshipLoadState, Message};
use crate::ui::championship_charts::tab_button_inner;
use crate::ui::components::secondary_button_icon;
use crate::ui::icons::{section_heading, subtitle_text, Icon};
use crate::ui::layout::{scale_text, LayoutConfig};
use crate::ui::theme::{border, muted, surface, text_color};

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
        ChartMode::RaceStanding => "Latest race results for the full grid",
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

    standings_grid(rows, layout)
}

fn standings_grid(rows: Vec<StandingRow>, layout: LayoutConfig) -> Element<'static, Message> {
    let columns = standings_columns(layout.viewport.width);
    let cards: Vec<Element<'static, Message>> = rows
        .into_iter()
        .map(|row| standing_card(row, layout))
        .collect();

    if columns == 1 {
        return column(cards).spacing(8).width(Length::Fill).into();
    }

    let mut grid = column![].spacing(8).width(Length::Fill);
    let mut iter = cards.into_iter();

    loop {
        let mut cells: Vec<Element<'static, Message>> = Vec::new();
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
                .spacing(8)
                .width(Length::Fill)
                .align_y(iced::Alignment::Start),
        );
    }

    grid.into()
}

fn standings_columns(viewport_width: f32) -> usize {
    if viewport_width < 720.0 {
        1
    } else {
        2
    }
}

fn standing_card(row: StandingRow, layout: LayoutConfig) -> Element<'static, Message> {
    let accent = row.accent;

    container(
        row![
            container(text(row.position_label).size(layout.text(18)).font(crate::ui::fonts::MONO).color(text_color()))
                .width(Length::Fixed(40.0))
                .align_x(iced::alignment::Horizontal::Center),
            column![
                text(row.label).size(layout.text(15)).color(text_color()),
                text(row.code).size(layout.text(14)).color(accent),
            ]
            .spacing(2)
            .width(Length::Fill),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .style(move |_| container::Style {
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
