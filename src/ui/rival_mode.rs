use iced::widget::{button, column, container, row, text, Space};
use iced::{alignment, Color, Element, Length};

use openf1::Driver;

use crate::domain::{build_rival_narrative, driver_display_name, team_colour};
use crate::state::{AppState, Message};
use crate::ui::components::secondary_button_icon;
use crate::ui::driver_card::{rival_driver_portrait, team_logo};
use crate::ui::icons::{section_heading, Icon};
use crate::ui::theme::{accent, border, muted, surface, text_color};

const VS_WIDTH: f32 = 72.0;
const ON_ACCENT: Color = Color::from_rgb(0.98, 0.98, 0.99);

pub fn rival_section(state: &AppState) -> Element<'_, Message> {
    let (first, second) = state.rival_drivers();
    let ready = state.rival_ready();
    let comparing = state.rival_compare_active();

    let header_action: Option<Element<Message>> = if comparing {
        Some(secondary_button_icon(
            Some(Icon::Close),
            "Stop comparing on charts",
            Message::ExitRivalCompare,
        ))
    } else if ready {
        Some(secondary_button_icon(
            Some(Icon::Trophy),
            "Compare on charts",
            Message::ActivateRivalCompare,
        ))
    } else {
        None
    };

    let mut header = row![section_heading(Icon::Users, "Driver rivalry", None),];
    if let Some(action) = header_action {
        header = header
            .push(Space::with_width(Length::Fill))
            .push(action);
    }

    let hint = if !ready {
        Some(
            text("Pick two drivers below. Pinned drivers are separate — this is your head-to-head matchup.")
                .size(12)
                .color(muted()),
        )
    } else if comparing {
        Some(
            text("Charts below show only these two drivers.")
                .size(12)
                .color(muted()),
        )
    } else {
        Some(
            text("Your rivals are set. Turn on chart comparison when you are ready.")
                .size(12)
                .color(muted()),
        )
    };

    let mut body = column![header].spacing(8).width(Length::Fill);
    if let Some(hint) = hint {
        body = body.push(hint);
    }
    body = body
        .push(Space::with_height(8))
        .push(rival_matchup(state, first, second));

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

fn rival_matchup(state: &AppState, first: i64, second: i64) -> Element<'_, Message> {
    let roster = state.drivers_roster().unwrap_or(&[]);
    let narrative = if state.rival_compare_active() {
        state
            .championship_data()
            .and_then(|data| build_rival_narrative(&data.rounds, roster, first, second))
    } else {
        None
    };

    let left = fighter_panel(state, roster, 0, first, alignment::Horizontal::Left);
    let center = container(text("VS").size(32).color(accent()))
        .width(Length::Fixed(VS_WIDTH))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);
    let right = fighter_panel(state, roster, 1, second, alignment::Horizontal::Right);

    let mut content = column![
        row![left, center, right]
            .spacing(0)
            .align_y(alignment::Vertical::Center)
            .width(Length::Fill),
    ]
    .spacing(12)
    .width(Length::Fill);

    if let Some(story) = narrative {
        content = content.push(
            container(
                column![
                    text(story.headline).size(14).color(text_color()),
                    text(story.detail).size(12).color(muted()),
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
            }),
        );
    }

    content.into()
}

fn fighter_panel(
    state: &AppState,
    roster: &[Driver],
    slot: u8,
    driver_number: i64,
    align: alignment::Horizontal,
) -> Element<'static, Message> {
    let driver = roster
        .iter()
        .find(|entry| entry.driver_number == driver_number);

    if let Some(driver) = driver {
        let colour = team_colour(&driver.team_colour);
        let code = if driver.name_acronym.is_empty() {
            driver_display_name(driver).to_string()
        } else {
            driver.name_acronym.to_ascii_uppercase()
        };
        let standing = championship_line(state, driver.driver_number);

        let identity = column![
            rival_driver_portrait(state, driver),
            Space::with_height(12),
            text(code).size(28).color(colour),
            Space::with_height(4),
            row![
                team_logo(state, driver),
                Space::with_width(8),
                text(driver.team_name.clone()).size(13).color(muted()),
            ]
            .spacing(0)
            .align_y(alignment::Vertical::Center),
            Space::with_height(6),
            text(standing).size(12).color(text_color()),
            Space::with_height(12),
            pick_driver_button(slot, "Change driver"),
        ]
        .spacing(0)
        .align_x(align)
        .width(Length::Fill);

        container(identity)
            .padding([12, 16])
            .width(Length::FillPortion(1))
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.12,
                    ..colour
                })),
                border: iced::Border {
                    color: iced::Color { a: 0.55, ..colour },
                    width: 2.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            })
            .into()
    } else {
        let label = if driver_number > 0 {
            format!("#{driver_number}")
        } else {
            format!("Driver {}", slot + 1)
        };

        container(
            column![
                container(text("?").size(48).color(muted()))
                    .width(Length::Fixed(128.0))
                    .height(Length::Fixed(128.0))
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
                text(label).size(22).color(muted()),
                Space::with_height(12),
                pick_driver_button(slot, "Pick driver"),
            ]
            .spacing(0)
            .align_x(align)
            .width(Length::Fill),
        )
        .padding([12, 16])
        .width(Length::FillPortion(1))
        .into()
    }
}

fn championship_line(state: &AppState, driver_number: i64) -> String {
    let Some(data) = state.championship_data() else {
        return "Championship data loading…".into();
    };
    let Some(latest) = data.rounds.last() else {
        return "No race data yet".into();
    };
    let Some(entry) = latest
        .drivers
        .iter()
        .find(|row| row.driver_number == driver_number)
    else {
        return "Not in standings".into();
    };

    format!("P{} · {} pts", entry.position, entry.points)
}

fn pick_driver_button(slot: u8, label: &'static str) -> Element<'static, Message> {
    button(text(label).size(12).color(ON_ACCENT))
        .padding([10, 16])
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
