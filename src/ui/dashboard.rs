use iced::widget::{column, container, image, row, scrollable, text, Space};
use iced::{Element, Length, Padding};

use openf1::Meeting;

use crate::domain::{
    circuit_image_url, countdown_segments, countdown_segments_pending,
    countdown_segments_zero, format_meeting_date_range, format_session_start, CountdownSegment,
    CountdownTarget, RaceTripletSlot,
};
use crate::state::{AppState, LoadState, Message};
use crate::ui::components::action_button_icon;
use crate::ui::icons::Icon;
use crate::ui::pinned_drivers::pinned_drivers_section;
use crate::ui::weather_panel::meeting_weather_panel;
use crate::ui::championship_charts::championship_charts_section;
use crate::ui::decals::{
    intermission_panel, signal_flag, signal_flag_sized, FlagSignal,
};
use crate::ui::fonts::MONO;
use crate::ui::layout::LayoutConfig;
use crate::ui::scroll::scrollable_page;
use crate::ui::theme::{ACCENT, BORDER, FLAG_BLUE, FLAG_GREEN, FLAG_YELLOW, MUTED, SURFACE};

pub fn dashboard(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let footer = if state.is_stale() {
        text("Showing cached data · OpenF1 (approx. 24h delay)")
            .size(12)
            .color(FLAG_YELLOW)
    } else {
        text("Data via OpenF1 (approx. 24h delay)")
            .size(12)
            .color(MUTED)
    };

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
    let spacing = if layout.viewport.height < 720.0 { 12 } else { 16 };

    column![
        calendar_section(state, layout),
        pinned_drivers_section(state, layout),
        championship_charts_section(state),
    ]
    .spacing(spacing)
    .width(Length::Fill)
    .height(Length::Shrink)
    .into()
}

fn calendar_section(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    match &state.load {
        LoadState::Loading => column![
            text("Loading season calendar…").color(MUTED),
            Space::with_height(16),
            skeleton_cards(layout),
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Shrink)
        .into(),
        LoadState::Error { message, .. } => column![
            signal_flag(FlagSignal::Alert, "RED"),
            text("Could not load calendar").size(18),
            text(message).color(MUTED),
            action_button_icon(Some(Icon::Refresh), "Retry", Message::Refresh),
        ]
        .spacing(8)
        .width(Length::Fill)
        .height(Length::Shrink)
        .into(),
        LoadState::Ready(_) => column![
            countdown_hero(state, layout),
            race_cards(state, state.animation_phase, layout),
        ]
        .spacing(if layout.viewport.height < 720.0 { 12 } else { 16 })
        .width(Length::Fill)
        .height(Length::Shrink)
        .into(),
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
            "SEASON COMPLETE".into(),
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
                text(session_label).size(13).color(MUTED),
            ]
            .align_y(iced::Alignment::Center),
            countdown_clock(timer_segments, layout),
            row![
                text("T-").size(14).color(FLAG_YELLOW),
                text("COUNTDOWN").size(12).color(MUTED),
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
        background: Some(SURFACE.into()),
        border: iced::Border {
            color: ACCENT,
            width: 2.0,
            radius: 10.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color {
                a: 0.35,
                ..ACCENT
            },
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    })
    .into()
}

fn countdown_clock(segments: Vec<CountdownSegment>, layout: LayoutConfig) -> Element<'static, Message> {
    let value_size = layout.countdown_size;
    let label_size = value_size.saturating_sub(value_size / 3).max(9);
    let label_row = label_size as f32 + 4.0;

    let mut clock = row![].spacing(0).align_y(iced::Alignment::Start);
    let segment_count = segments.len();

    for (index, segment) in segments.into_iter().enumerate() {
        if index > 0 {
            let separator = if index == segment_count - 1 { "." } else { ":" };
            clock = clock.push(
                container(
                    text(separator)
                        .size(value_size)
                        .font(MONO)
                        .color(ACCENT),
                )
                .padding(Padding {
                    top: 0.0,
                    right: 4.0,
                    bottom: label_row,
                    left: 4.0,
                }),
            );
        }

        clock = clock.push(
            column![
                text(segment.value)
                    .size(value_size)
                    .font(MONO)
                    .color(ACCENT),
                text(segment.label)
                    .size(label_size)
                    .color(MUTED),
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
        CardMode::Previous => (BORDER, 1.0),
        CardMode::Live => (FLAG_GREEN, 2.0),
        CardMode::Intermission => (FLAG_YELLOW, 2.0),
        CardMode::Upcoming => (FLAG_GREEN, 1.5),
    };

    let status = match mode {
        CardMode::Previous => None,
        CardMode::Live => Some(signal_flag_sized(FlagSignal::Live, "LIVE", layout.card_detail_size)),
        CardMode::Intermission => {
            Some(signal_flag_sized(FlagSignal::Intermission, "SC", layout.card_detail_size))
        }
        CardMode::Upcoming => Some(signal_flag_sized(FlagSignal::Next, "NEXT", layout.card_detail_size)),
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
        (Some(m), _) => card_body(
            column![
                card_header(heading, status, layout),
                meeting_header(state, m, layout),
                text(&m.circuit_short_name)
                    .size(layout.card_body_size)
                    .color(MUTED),
                text(format!("{}, {}", m.location, m.country_name))
                    .size(layout.card_detail_size)
                    .color(MUTED),
                text(format_date_range(m))
                    .size(layout.card_detail_size)
                    .color(MUTED),
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
                    .color(MUTED),
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
            background: Some(SURFACE.into()),
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

    if data.triplet.previous.is_none() {
        "Pre-season · Next race weekend on the calendar"
    } else {
        "Off-weekend · Next session on the clock"
    }
}

fn card_header<'a>(
    heading: &'a str,
    status: Option<Element<'a, Message>>,
    layout: LayoutConfig,
) -> Element<'a, Message> {
    let heading = text(heading).size(layout.card_heading_size).color(MUTED);

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
                color: BORDER,
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
        background: Some(SURFACE.into()),
        border: iced::Border {
            color: BORDER,
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
        container(
            text("…")
                .size(layout.card_detail_size)
                .color(MUTED),
        )
        .center_x(height)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .style(|_| container::Style {
            border: iced::Border {
                color: BORDER,
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
                .color(MUTED),
        )
        .center_x(height)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .style(|_| container::Style {
            border: iced::Border {
                color: BORDER,
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
    container(
        column![
            text("…").color(MUTED),
            text("Loading").color(MUTED),
        ]
        .spacing(8),
    )
    .padding(layout.card_padding)
    .width(layout.card_width)
    .height(layout.card_height())
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

fn format_date_range(meeting: &Meeting) -> String {
    format_meeting_date_range(&meeting.date_start, &meeting.date_end)
}
