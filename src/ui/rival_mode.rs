use iced::widget::{button, column, container, row, text, Space};
use iced::{alignment, Color, Element, Length};

use openf1::Driver;

use crate::domain::{build_rival_narrative, driver_display_name, team_colour};
use crate::state::{AppState, Message};
use crate::ui::driver_card::{rival_fighter_portrait, rival_fighter_team_logo_fill};
use crate::ui::icons::{section_heading, Icon};
use crate::ui::layout::{scale_text, LayoutConfig};
use crate::ui::theme::{accent, border, surface, text_color};

const VS_WIDTH: f32 = 88.0;
const ON_ACCENT: Color = Color::from_rgb(0.98, 0.98, 0.99);

pub fn rival_section(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let scale = layout.font_scale;
    let (first, second) = state.rival_drivers();
    let ready = state.rival_ready();
    let comparing = state.rival_compare_active();

    let header = row![section_heading(Icon::Users, "Driver rivalry", None),]
        .width(Length::Fill);

    let hint = if !ready {
        Some(
            text("Pick two drivers below. Pinned drivers are separate — this is your head-to-head matchup.")
                .size(scale_text(12, scale))
                .color(text_color()),
        )
    } else if comparing {
        Some(
            text("Charts show only these two drivers while Compare rivals is active.")
                .size(scale_text(12, scale))
                .color(text_color()),
        )
    } else {
        None
    };

    let mut body = column![header].spacing(8).width(Length::Fill);
    if let Some(hint) = hint {
        body = body.push(hint);
    }
    body = body
        .push(Space::with_height(8))
        .push(rival_matchup(state, first, second, scale));

    container(body.padding(20))
        .width(Length::Fill)
        .height(Length::Shrink)
        .style(|_| container::Style {
            background: Some(surface().into()),
            border: iced::Border {
                color: border(),
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn rival_matchup(state: &AppState, first: i64, second: i64, scale: f32) -> Element<'_, Message> {
    let roster = state.drivers_roster().unwrap_or(&[]);

    let left = fighter_panel(state, roster, 0, first, scale);
    let center = container(text("VS").size(scale_text(36, scale)).color(accent()))
        .width(Length::Fixed(VS_WIDTH))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);
    let right = fighter_panel(state, roster, 1, second, scale);

    column![
        row![left, center, right]
            .spacing(0)
            .align_y(alignment::Vertical::Center)
            .width(Length::Fill),
        rival_gap_banner(state, roster, first, second, scale),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn rival_gap_banner(
    state: &AppState,
    roster: &[Driver],
    first: i64,
    second: i64,
    scale: f32,
) -> Element<'static, Message> {
    let (headline, detail) = rival_gap_copy(state, roster, first, second);

    container(
        column![
            text(headline)
                .size(scale_text(24, scale))
                .color(text_color()),
            text(detail).size(scale_text(18, scale)).color(text_color()),
        ]
        .spacing(4)
        .align_x(alignment::Horizontal::Center)
        .width(Length::Fill),
    )
    .padding([10, 16])
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(iced::Color {
            a: 0.35,
            ..accent()
        })),
        border: iced::Border {
            color: iced::Color { a: 0.5, ..accent() },
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn rival_gap_copy(
    state: &AppState,
    roster: &[Driver],
    first: i64,
    second: i64,
) -> (String, String) {
    if !state.rival_ready() {
        return (
            "Championship gap".into(),
            "Pick two drivers to compare their points gap.".into(),
        );
    }

    let Some(data) = state.championship_data() else {
        return ("Championship gap".into(), "Standings are loading…".into());
    };

    if let Some(story) = build_rival_narrative(&data.rounds, roster, first, second) {
        return (story.headline, story.detail);
    }

    (
        "Championship gap".into(),
        "Standings unavailable for one or both rivals.".into(),
    )
}

fn fighter_panel(
    state: &AppState,
    roster: &[Driver],
    slot: u8,
    driver_number: i64,
    scale: f32,
) -> Element<'static, Message> {
    let driver = roster
        .iter()
        .find(|entry| entry.driver_number == driver_number);

    if let Some(driver) = driver {
        let colour = team_colour(&driver.team_colour);
        let code = driver_code(driver);
        let stats = rival_driver_stats(state, driver.driver_number);

        let body = column![
            row![
                identity_column(state, driver, code, scale),
                constructor_column(state, driver, scale),
                stats_column(stats, scale),
            ]
            .spacing(20)
            .align_y(alignment::Vertical::Center)
            .width(Length::Fill),
            Space::with_height(16),
            container(pick_driver_button(slot, "Change driver", scale))
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
        ]
        .spacing(0)
        .width(Length::Fill);

        container(body)
            .padding([16, 20])
            .width(Length::FillPortion(1))
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(iced::Color { a: 0.12, ..colour })),
                border: iced::Border {
                    color: iced::Color { a: 0.55, ..colour },
                    width: 2.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            })
            .into()
    } else {
        empty_fighter_panel(slot, driver_number, scale)
    }
}

fn identity_column(
    state: &AppState,
    driver: &Driver,
    code: String,
    scale: f32,
) -> Element<'static, Message> {
    column![
        rival_fighter_portrait(state, driver),
        Space::with_height(12),
        text(driver.full_name.clone())
            .size(scale_text(17, scale))
            .color(text_color()),
        text(code)
            .size(scale_text(28, scale))
            .color(team_colour(&driver.team_colour)),
    ]
    .spacing(6)
    .align_x(alignment::Horizontal::Left)
    .width(Length::FillPortion(1))
    .into()
}

fn constructor_column(state: &AppState, driver: &Driver, scale: f32) -> Element<'static, Message> {
    column![
        rival_fighter_team_logo_fill(state, driver),
        Space::with_height(12),
        text(driver.team_name.clone())
            .size(scale_text(17, scale))
            .color(text_color()),
    ]
    .spacing(6)
    .align_x(alignment::Horizontal::Center)
    .width(Length::FillPortion(1))
    .into()
}

fn stats_column(stats: RivalDriverStats, scale: f32) -> Element<'static, Message> {
    container(
        column![
            text(stats.points_line)
                .size(scale_text(36, scale))
                .color(text_color()),
            Space::with_height(10),
            text(stats.championship_line)
                .size(scale_text(20, scale))
                .color(text_color()),
            Space::with_height(6),
            text(stats.latest_race_line)
                .size(scale_text(20, scale))
                .color(text_color()),
        ]
        .spacing(0)
        .align_x(alignment::Horizontal::Right)
        .width(Length::Fill),
    )
    .width(Length::FillPortion(1))
    .align_y(alignment::Vertical::Center)
    .height(Length::Shrink)
    .into()
}

fn empty_fighter_panel(slot: u8, driver_number: i64, scale: f32) -> Element<'static, Message> {
    let label = if driver_number > 0 {
        format!("#{driver_number}")
    } else {
        format!("Driver {}", slot + 1)
    };

    container(
        column![
            container(text("?").size(scale_text(56, scale)).color(text_color()))
                .width(Length::Fixed(112.0))
                .height(Length::Fixed(112.0))
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .style(|_| container::Style {
                    background: Some(surface().into()),
                    border: iced::Border {
                        color: border(),
                        width: 2.0,
                        radius: 10.0.into(),
                    },
                    ..Default::default()
                }),
            Space::with_height(12),
            text(label).size(scale_text(26, scale)).color(text_color()),
            Space::with_height(12),
            pick_driver_button(slot, "Pick driver", scale),
        ]
        .spacing(0)
        .align_x(alignment::Horizontal::Center)
        .width(Length::Fill),
    )
    .padding([16, 20])
    .width(Length::FillPortion(1))
    .into()
}

struct RivalDriverStats {
    points_line: String,
    championship_line: String,
    latest_race_line: String,
}

fn rival_driver_stats(state: &AppState, driver_number: i64) -> RivalDriverStats {
    let Some(data) = state.championship_data() else {
        return RivalDriverStats {
            points_line: "— pts".into(),
            championship_line: "Championship loading…".into(),
            latest_race_line: "Latest race loading…".into(),
        };
    };

    let Some(latest) = data.rounds.last() else {
        return RivalDriverStats {
            points_line: "0 pts".into(),
            championship_line: "No championship data".into(),
            latest_race_line: "No race data yet".into(),
        };
    };

    let standing = latest
        .drivers
        .iter()
        .find(|entry| entry.driver_number == driver_number);

    let points_line = standing
        .map(|entry| format!("{} pts", entry.points))
        .unwrap_or_else(|| "— pts".into());

    let championship_line = standing
        .map(|entry| format!("P{} championship", entry.position))
        .unwrap_or_else(|| "Not in standings".into());

    let latest_race_line = latest
        .race_results
        .iter()
        .find(|entry| entry.driver_number == driver_number)
        .map(format_latest_race)
        .unwrap_or_else(|| "No latest race result".into());

    RivalDriverStats {
        points_line,
        championship_line,
        latest_race_line,
    }
}

fn format_latest_race(result: &crate::domain::championship::RaceResultSnapshot) -> String {
    let label = if result.dsq {
        "DSQ".into()
    } else if result.dns {
        "DNS".into()
    } else if result.dnf {
        "DNF".into()
    } else {
        format!("P{}", result.classified_position)
    };

    format!("{label} latest race")
}

fn driver_code(driver: &Driver) -> String {
    if driver.name_acronym.is_empty() {
        driver_display_name(driver).to_string()
    } else {
        driver.name_acronym.to_ascii_uppercase()
    }
}

fn pick_driver_button(slot: u8, label: &'static str, scale: f32) -> Element<'static, Message> {
    button(text(label).size(scale_text(14, scale)).color(ON_ACCENT))
        .padding([12, 20])
        .on_press(Message::OpenRivalPicker(slot))
        .style(|_, status| {
            use iced::widget::button::{self, Status};
            let base = accent();
            let background = match status {
                Status::Hovered => iced::Color {
                    r: (base.r * 0.88).max(0.0),
                    g: (base.g * 0.88).max(0.0),
                    b: (base.b * 0.88).max(0.0),
                    a: 1.0,
                },
                Status::Pressed => iced::Color {
                    r: (base.r * 0.76).max(0.0),
                    g: (base.g * 0.76).max(0.0),
                    b: (base.b * 0.76).max(0.0),
                    a: 1.0,
                },
                _ => base,
            };
            button::Style {
                background: Some(iced::Background::Color(background)),
                text_color: ON_ACCENT,
                border: iced::Border {
                    color: background,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}
