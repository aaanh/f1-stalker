use iced::widget::{column, container, image, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use openf1::Meeting;

use crate::domain::{
    build_championship_narrative, build_season_complete_narrative, circuit_image_url,
    countdown_segments, countdown_segments_pending, countdown_segments_zero,
    format_fetched_at_long, format_meeting_date_range, format_session_start, podium_for_meeting,
    season_phase, CountdownSegment, CountdownTarget, PodiumEntry, RaceTripletSlot, SeasonPhase,
};
use crate::state::{AppState, LoadState, Message};
use crate::ui::championship_charts::championship_charts_section;
use crate::ui::components::action_button_icon;
use crate::ui::decals::{intermission_panel, signal_flag, signal_flag_sized, FlagSignal};
use crate::ui::driver_card::driver_portrait_sized;
use crate::ui::fonts::MONO;
use crate::ui::icons::Icon;
use crate::ui::layout::LayoutConfig;
use crate::ui::pinned_drivers::pinned_drivers_section;
use crate::ui::rival_mode::rival_section;
use crate::ui::scroll::scrollable_page;
use crate::ui::standings_table::standings_section;
use crate::ui::theme::{
    accent, border, muted, surface, text_color, FLAG_BLUE, FLAG_GREEN, FLAG_YELLOW,
};
use crate::ui::weather_panel::meeting_weather_panel;

pub fn dashboard(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let footer = data_footer(state, layout);

    let body = main_body(state, layout);

    let scroll_content = container(
        column![body, Space::with_height(12), footer]
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Shrink),
    )
    .padding(layout.padding)
    .width(Length::Fill)
    .height(Length::Shrink);

    scrollable_page(
        scrollable::Id::new("dashboard-scroll"),
        scroll_content.into(),
        state.scrollbar_visible.visible,
    )
}

fn main_body(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let spacing = if layout.viewport.height < 720.0 {
        12
    } else {
        16
    };

    column![
        calendar_section(state, layout),
        rival_section(state, layout),
        pinned_drivers_section(state, layout),
        championship_narrative_banner(state, layout),
        championship_charts_section(state, layout),
        standings_section(state, layout),
    ]
    .spacing(spacing)
    .width(Length::Fill)
    .height(Length::Shrink)
    .into()
}

fn calendar_section(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    match &state.load {
        LoadState::Loading => column![
            text("Loading season calendar…").color(muted()),
            Space::with_height(16),
            skeleton_cards(layout),
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Shrink)
        .into(),
        LoadState::Error {
            message,
            cached: None,
        } => column![
            signal_flag(FlagSignal::Alert, "RED"),
            text("Could not load calendar").size(layout.text(18)),
            text(message).color(muted()),
            action_button_icon(Some(Icon::Refresh), "Retry", Message::Refresh),
        ]
        .spacing(8)
        .width(Length::Fill)
        .height(Length::Shrink)
        .into(),
        LoadState::Ready(_)
        | LoadState::Error {
            cached: Some(_), ..
        } => {
            let mut section = column![].spacing(if layout.viewport.height < 720.0 {
                12
            } else {
                16
            });
            if let Some(banner) = season_banner(state, layout) {
                section = section.push(banner);
            }
            section = section.push(countdown_hero(state, layout));
            section = section.push(race_cards(state, state.animation_phase, layout));
            section.width(Length::Fill).height(Length::Shrink).into()
        }
    }
}

fn countdown_hero(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let Some(data) = state.calendar() else {
        return Space::new(Length::Fill, Length::Fixed(1.0)).into();
    };

    let (session_label, timer_segments) = match &data.countdown {
        CountdownTarget::NextSession {
            session_name,
            starts_at,
        } => (
            format!(
                "NEXT SESSION · {session_name} · {}",
                format_session_start(*starts_at)
            ),
            countdown_segments(chrono::Utc::now(), *starts_at),
        ),
        CountdownTarget::SchedulePending => (
            "NEXT SESSION · Schedule pending".into(),
            countdown_segments_pending(),
        ),
        CountdownTarget::SeasonComplete => (
            "OFF-SEASON · Season complete".into(),
            countdown_segments_zero(),
        ),
    };

    let hero_padding: [f32; 2] = if layout.viewport.height < 680.0 {
        [12.0, 16.0]
    } else if layout.countdown_size < 46 {
        [16.0, 20.0]
    } else {
        [20.0, 24.0]
    };

    container(
        column![
            row![
                signal_flag(FlagSignal::Alert, "LIGHTS OUT"),
                Space::with_width(Length::Fill),
                text(session_label).size(layout.text(13)).color(muted()),
            ]
            .align_y(iced::Alignment::Center),
            countdown_clock(timer_segments, layout),
            row![
                text("T-").size(layout.text(14)).color(FLAG_YELLOW),
                text("COUNTDOWN").size(layout.text(12)).color(muted()),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(8)
        .align_x(iced::Alignment::Center),
    )
    .padding(hero_padding)
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(surface().into()),
        border: iced::Border {
            color: accent(),
            width: 2.0,
            radius: 10.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color {
                a: 0.35,
                ..accent()
            },
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    })
    .into()
}

fn countdown_clock(
    segments: Vec<CountdownSegment>,
    layout: LayoutConfig,
) -> Element<'static, Message> {
    let value_size = layout.countdown_size;
    let label_size = value_size.saturating_sub(value_size / 3).max(9);
    let label_row = label_size as f32 + 4.0;

    let mut clock = row![].spacing(0).align_y(iced::Alignment::Start);
    let segment_count = segments.len();

    for (index, segment) in segments.into_iter().enumerate() {
        if index > 0 {
            let separator = if index == segment_count - 1 { "." } else { ":" };
            clock = clock.push(
                container(text(separator).size(value_size).font(MONO).color(accent())).padding(
                    Padding {
                        top: 0.0,
                        right: 4.0,
                        bottom: label_row,
                        left: 4.0,
                    },
                ),
            );
        }

        clock = clock.push(
            column![
                text(segment.value)
                    .size(value_size)
                    .font(MONO)
                    .color(accent()),
                text(segment.label).size(label_size).color(muted()),
            ]
            .spacing(2)
            .align_x(iced::Alignment::Center),
        );
    }

    clock.into()
}

fn race_cards(state: &AppState, frame: u32, layout: LayoutConfig) -> Element<'_, Message> {
    let Some(data) = state.calendar() else {
        return skeleton_cards(layout);
    };

    let previous = race_card(
        state,
        RaceTripletSlot::Previous,
        data.triplet.previous.as_ref(),
        CardMode::Previous,
        0,
        layout,
    );
    let current = race_card(
        state,
        RaceTripletSlot::Current,
        Some(&data.triplet.current),
        if data.triplet.current_is_weekend {
            CardMode::Live
        } else {
            CardMode::Intermission
        },
        frame,
        layout,
    );
    let upcoming = race_card(
        state,
        RaceTripletSlot::Upcoming,
        Some(&data.triplet.upcoming),
        CardMode::Upcoming,
        0,
        layout,
    );

    if layout.stack_cards {
        column![previous, current, upcoming]
            .spacing(12)
            .width(Length::Fill)
            .into()
    } else {
        row![previous, current, upcoming]
            .spacing(12)
            .width(Length::Fill)
            .align_y(iced::Alignment::Start)
            .into()
    }
}

#[derive(Debug, Clone, Copy)]
enum CardMode {
    Previous,
    Live,
    Intermission,
    Upcoming,
}

fn race_card<'a>(
    state: &'a AppState,
    slot: RaceTripletSlot,
    meeting: Option<&'a Meeting>,
    mode: CardMode,
    frame: u32,
    layout: LayoutConfig,
) -> Element<'a, Message> {
    let heading = match slot {
        RaceTripletSlot::Previous => "Previous",
        RaceTripletSlot::Current => "Current",
        RaceTripletSlot::Upcoming => "Upcoming",
    };

    let (border_color, border_width) = match mode {
        CardMode::Previous => (border(), 1.0),
        CardMode::Live => (FLAG_GREEN, 2.0),
        CardMode::Intermission => (FLAG_YELLOW, 2.0),
        CardMode::Upcoming => (FLAG_GREEN, 1.5),
    };

    let status = match mode {
        CardMode::Previous => None,
        CardMode::Live => Some(signal_flag_sized(
            FlagSignal::Live,
            "live()",
            layout.card_detail_size,
        )),
        CardMode::Intermission => Some(signal_flag_sized(
            FlagSignal::Intermission,
            "SC",
            layout.card_detail_size,
        )),
        CardMode::Upcoming => Some(signal_flag_sized(
            FlagSignal::Next,
            "NEXT",
            layout.card_detail_size,
        )),
    };

    let body: Element<'a, Message> = match (meeting, mode) {
        (Some(_), CardMode::Intermission) => card_body(
            column![
                card_header(heading, status, layout),
                intermission_panel(frame, intermission_subtitle(state), layout),
            ]
            .spacing(10),
            layout,
        ),
        (Some(m), CardMode::Previous | CardMode::Live) => card_body(
            column![
                card_header(heading, status, layout),
                meeting_header(state, m, layout),
                text(&m.circuit_short_name)
                    .size(layout.card_body_size)
                    .color(muted()),
                text(format!("{}, {}", m.location, m.country_name))
                    .size(layout.card_detail_size)
                    .color(muted()),
                text(format_date_range(m))
                    .size(layout.card_detail_size)
                    .color(muted()),
                podium_panel(state, m.meeting_key, layout),
                meeting_weather_panel(state, m, layout),
                track_outline(state, m, layout),
            ]
            .spacing(8),
            layout,
        ),
        (Some(m), CardMode::Upcoming) => card_body(
            column![
                card_header(heading, status, layout),
                meeting_header(state, m, layout),
                text(&m.circuit_short_name)
                    .size(layout.card_body_size)
                    .color(muted()),
                text(format!("{}, {}", m.location, m.country_name))
                    .size(layout.card_detail_size)
                    .color(muted()),
                text(format_date_range(m))
                    .size(layout.card_detail_size)
                    .color(muted()),
                meeting_weather_panel(state, m, layout),
                track_outline(state, m, layout),
            ]
            .spacing(8),
            layout,
        ),
        (None, _) => card_body(
            column![
                card_header(heading, status, layout),
                text("No previous round")
                    .size(layout.card_body_size)
                    .color(muted()),
            ]
            .spacing(8),
            layout,
        ),
    };

    container(body)
        .padding(layout.card_padding)
        .width(layout.card_width)
        .height(layout.card_height())
        .style(move |_| container::Style {
            background: Some(surface().into()),
            border: iced::Border {
                color: border_color,
                width: border_width,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn intermission_subtitle(state: &AppState) -> &'static str {
    let Some(data) = state.calendar() else {
        return "Next session on the clock";
    };

    match season_phase(&data.triplet, &data.countdown) {
        SeasonPhase::PreSeason => "Pre-season · Next race weekend on the calendar",
        SeasonPhase::SeasonComplete => "Season complete · Final round in the rear-view",
        SeasonPhase::OffWeekend | SeasonPhase::RaceWeekend => {
            "Off-weekend · Next session on the clock"
        }
    }
}

fn card_header<'a>(
    heading: &'a str,
    status: Option<Element<'a, Message>>,
    layout: LayoutConfig,
) -> Element<'a, Message> {
    let heading = text(heading).size(layout.card_heading_size).color(muted());

    match status {
        Some(status) => row![heading, Space::with_width(Length::Fill), status]
            .align_y(iced::Alignment::Center)
            .into(),
        None => row![heading, Space::with_width(Length::Fill)]
            .align_y(iced::Alignment::Center)
            .into(),
    }
}

fn card_body<'a>(
    content: iced::widget::Column<'a, Message>,
    layout: LayoutConfig,
) -> Element<'a, Message> {
    if layout.stack_cards {
        return content.into();
    }

    content
        .push(Space::new(Length::Fill, Length::Shrink))
        .width(Length::Fill)
        .into()
}

fn meeting_header<'a>(
    state: &'a AppState,
    meeting: &'a Meeting,
    layout: LayoutConfig,
) -> Element<'a, Message> {
    row![
        country_flag(state, meeting, layout),
        Space::with_width(10),
        text(&meeting.meeting_name).size(layout.meeting_title_size),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}

fn country_flag<'a>(
    state: &'a AppState,
    meeting: &'a Meeting,
    layout: LayoutConfig,
) -> Element<'a, Message> {
    if let Some(handle) = state.flag_handle(&meeting.country_flag) {
        return container(
            image(handle)
                .width(Length::Fixed(layout.card_flag_width))
                .height(Length::Fixed(layout.card_flag_height))
                .content_fit(iced::ContentFit::Contain),
        )
        .padding(2)
        .style(|_| container::Style {
            border: iced::Border {
                color: border(),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into();
    }

    container(
        text(&meeting.country_code)
            .size(layout.card_detail_size)
            .color(FLAG_BLUE),
    )
    .padding([8, 10])
    .style(|_| container::Style {
        background: Some(surface().into()),
        border: iced::Border {
            color: border(),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn track_outline<'a>(
    state: &'a AppState,
    meeting: &'a Meeting,
    layout: LayoutConfig,
) -> Element<'a, Message> {
    let height = layout.card_track_height;

    let Some(url) = circuit_image_url(meeting) else {
        return track_map_not_found(layout, height);
    };

    if let Some(handle) = state.flag_handle(url) {
        return container(
            image(handle)
                .width(Length::Fill)
                .height(Length::Fixed(height))
                .content_fit(iced::ContentFit::Contain),
        )
        .width(Length::Fill)
        .align_x(iced::Alignment::Center)
        .into();
    }

    if state.asset_fetch_failed(url) {
        return track_map_not_found(layout, height);
    }

    container(
        container(text("…").size(layout.card_detail_size).color(muted()))
            .center_x(height)
            .width(Length::Fill)
            .height(Length::Fixed(height))
            .style(|_| container::Style {
                border: iced::Border {
                    color: border(),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
    )
    .width(Length::Fill)
    .align_x(iced::Alignment::Center)
    .into()
}

fn track_map_not_found(layout: LayoutConfig, height: f32) -> Element<'static, Message> {
    container(
        container(
            text("Map not found")
                .size(layout.card_detail_size)
                .color(muted()),
        )
        .center_x(height)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .style(|_| container::Style {
            border: iced::Border {
                color: border(),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }),
    )
    .width(Length::Fill)
    .align_x(iced::Alignment::Center)
    .into()
}

fn skeleton_cards(layout: LayoutConfig) -> Element<'static, Message> {
    let cards = [
        skeleton_card(layout),
        skeleton_card(layout),
        skeleton_card(layout),
    ];

    if layout.stack_cards {
        column(cards).spacing(12).width(Length::Fill).into()
    } else {
        row(cards).spacing(12).width(Length::Fill).into()
    }
}

fn skeleton_card(layout: LayoutConfig) -> Element<'static, Message> {
    container(column![text("…").color(muted()), text("Loading").color(muted()),].spacing(8))
        .padding(layout.card_padding)
        .width(layout.card_width)
        .height(layout.card_height())
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

fn format_date_range(meeting: &Meeting) -> String {
    format_meeting_date_range(&meeting.date_start, &meeting.date_end)
}

fn podium_panel(state: &AppState, meeting_key: i64, layout: LayoutConfig) -> Element<'_, Message> {
    let Some(data) = state.championship_data() else {
        return Space::new(Length::Shrink, Length::Fixed(0.0)).into();
    };

    let roster = state.drivers_roster().unwrap_or(&[]);
    let podium = podium_for_meeting(&data.rounds, meeting_key, roster);

    if podium.is_empty() {
        return Space::new(Length::Shrink, Length::Fixed(0.0)).into();
    }

    let cards: Vec<Element<'_, Message>> = podium
        .iter()
        .filter_map(|entry| {
            roster
                .iter()
                .find(|driver| driver.driver_number == entry.driver_number)
                .map(|driver| podium_driver_card(state, entry, driver, layout))
        })
        .collect();

    if cards.is_empty() {
        return Space::new(Length::Shrink, Length::Fixed(0.0)).into();
    }

    column![
        text("Podium").size(layout.card_detail_size).color(muted()),
        row(cards)
            .spacing(8)
            .width(Length::Fill)
            .align_y(iced::Alignment::Start),
    ]
    .spacing(6)
    .width(Length::Fill)
    .into()
}

fn podium_driver_card<'a>(
    state: &'a AppState,
    entry: &PodiumEntry,
    driver: &openf1::Driver,
    layout: LayoutConfig,
) -> Element<'a, Message> {
    let portrait_size = (56.0 * layout.font_scale).max(44.0);
    let position_label = format!("P{}", entry.position);

    container(
        column![
            driver_portrait_sized(state, driver, portrait_size),
            text(position_label)
                .size(layout.card_detail_size)
                .font(MONO)
                .color(entry.accent),
            text(entry.code.clone())
                .size(layout.card_detail_size)
                .color(text_color()),
        ]
        .spacing(4)
        .align_x(iced::Alignment::Center)
        .width(Length::Fill),
    )
    .padding([8, 6])
    .width(Length::FillPortion(1))
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

fn season_banner(state: &AppState, layout: LayoutConfig) -> Option<Element<'static, Message>> {
    let phase = state.season_phase()?;
    let (title, detail) = match phase {
        SeasonPhase::PreSeason => (
            "Pre-season",
            "No race weekend is active yet. The cards below show the first Grand Prix on the calendar.",
        ),
        SeasonPhase::SeasonComplete => (
            "Off-season",
            "The season is over. Standings and grids may still update for about 24 hours after the final race.",
        ),
        SeasonPhase::OffWeekend | SeasonPhase::RaceWeekend => return None,
    };

    Some(
        container(
            column![
                row![
                    signal_flag(FlagSignal::Intermission, title),
                    Space::with_width(Length::Fill),
                ],
                text(detail).size(layout.text(13)).color(muted()),
            ]
            .spacing(8)
            .width(Length::Fill),
        )
        .padding(12)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(surface().into()),
            border: iced::Border {
                color: FLAG_YELLOW,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into(),
    )
}

fn championship_narrative_banner(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let Some(data) = state.championship_data() else {
        return Space::new(Length::Shrink, Length::Fixed(0.0)).into();
    };

    if data.rounds.is_empty() {
        return Space::new(Length::Shrink, Length::Fixed(0.0)).into();
    }

    let roster = state.drivers_roster().unwrap_or(&[]);
    let narrative = match state.season_phase() {
        Some(SeasonPhase::SeasonComplete) => build_season_complete_narrative(&data.rounds, roster),
        _ => build_championship_narrative(&data.rounds, roster),
    };

    let Some(narrative) = narrative else {
        return Space::new(Length::Shrink, Length::Fixed(0.0)).into();
    };

    container(
        column![
            text(narrative.headline)
                .size(layout.text(24))
                .color(text_color()),
            text(narrative.detail).size(layout.text(18)).color(muted()),
        ]
        .spacing(4)
        .width(Length::Fill),
    )
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

fn data_footer(state: &AppState, layout: LayoutConfig) -> Element<'static, Message> {
    let delay = "OpenF1 (approx. 24h delay)";
    let updated = state
        .last_fetched_at()
        .map(format_fetched_at_long)
        .unwrap_or_else(|| "unknown".into());

    let label = if state.is_any_stale() {
        format!("Showing cached data · last updated {updated} · {delay}")
    } else {
        format!("Data via {delay} · Updated {updated}")
    };

    text(label)
        .size(layout.text(12))
        .color(if state.is_any_stale() {
            FLAG_YELLOW
        } else {
            muted()
        })
        .into()
}
