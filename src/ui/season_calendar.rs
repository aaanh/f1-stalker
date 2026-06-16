use chrono::{Datelike, Local};
use iced::widget::scrollable::Direction;
use iced::widget::{column, container, row, scrollable, text, tooltip, Space};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::domain::{
    build_season_calendar, DaySession, RacePhase, SeasonCalendarDay,
};
use crate::state::{AppState, Message};
use crate::ui::icons::{section_heading, subtitle_text, Icon};
use crate::ui::layout::{scale_px, LayoutConfig};
use crate::ui::scroll::scrollbar_style;
use crate::ui::theme::{accent, border, live, muted, surface, text_color};

const WEEKDAYS: [&str; 7] = ["M", "T", "W", "T", "F", "S", "S"];
const WEEK_ROWS: usize = 6;
const CELL_BASE: f32 = 32.0;
const GRID_GAP: f32 = 2.0;
const MONTH_PAD: f32 = 10.0;

#[derive(Clone, Copy)]
struct CalendarGridMetrics {
    cell: f32,
    grid_width: f32,
    month_width: f32,
    month_height: f32,
    weekday_height: f32,
}

impl CalendarGridMetrics {
    fn from_layout(layout: LayoutConfig) -> Self {
        let cell = scale_px(CELL_BASE, layout.font_scale).max(26.0);
        let grid_width = cell * 7.0 + GRID_GAP * 6.0;
        let month_width = grid_width + MONTH_PAD * 2.0;
        let weekday_height = scale_px(16.0, layout.font_scale);
        let title_block = scale_px(18.0, layout.font_scale) + 6.0 + weekday_height + 4.0;
        let grid_height = WEEK_ROWS as f32 * cell + (WEEK_ROWS.saturating_sub(1)) as f32 * GRID_GAP;
        let month_height = MONTH_PAD * 2.0 + title_block + grid_height;

        Self {
            cell,
            grid_width,
            month_width,
            month_height,
            weekday_height,
        }
    }
}

pub fn season_calendar_section(
    state: &AppState,
    layout: LayoutConfig,
) -> Option<Element<'_, Message>> {
    let data = state.calendar()?;
    let calendar = build_season_calendar(
        &data.meetings,
        &data.sessions,
        &data.triplet,
        chrono::Utc::now(),
    )?;

    let metrics = CalendarGridMetrics::from_layout(layout);
    let months = calendar.months.clone();

    Some(
        column![
            section_heading(
                Icon::Calendar,
                "Season calendar",
                Some(subtitle_text(
                    "First race to last · hover a date for sessions"
                )),
            ),
            legend_row(layout),
            horizontal_month_strip(months, layout, metrics),
        ]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Shrink)
        .into(),
    )
}

fn legend_row(layout: LayoutConfig) -> Element<'static, Message> {
    let past = legend_chip("Past", phase_color(RacePhase::Past), layout);
    let current = legend_chip(
        "Current / upcoming",
        phase_color(RacePhase::Current),
        layout,
    );
    let future = legend_chip("Future", phase_color(RacePhase::Future), layout);

    if layout.stack_header {
        column![past, current, future].spacing(6).into()
    } else {
        row![past, current, future].spacing(12).into()
    }
}

fn legend_chip(
    label: &'static str,
    color: Color,
    layout: LayoutConfig,
) -> Element<'static, Message> {
    row![
        container(Space::new(Length::Fixed(10.0), Length::Fixed(10.0)))
            .width(Length::Fixed(10.0))
            .height(Length::Fixed(10.0))
            .style(move |_| container::Style {
                background: Some(color.into()),
                border: iced::Border {
                    color,
                    width: 1.0,
                    radius: 999.0.into(),
                },
                ..Default::default()
            }),
        text(label).size(layout.text(11)).color(muted()),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

fn horizontal_month_strip(
    months: Vec<crate::domain::SeasonCalendarMonth>,
    layout: LayoutConfig,
    metrics: CalendarGridMetrics,
) -> Element<'static, Message> {
    let months = row(months
        .into_iter()
        .map(|month| month_column(month, layout, metrics)))
    .spacing(16)
    .padding(Padding::from([4.0, 0.0]))
    .width(Length::Shrink)
    .height(Length::Fixed(metrics.month_height));

    scrollable(months)
        .direction(Direction::Horizontal(
            scrollable::Scrollbar::new()
                .width(5.0)
                .scroller_width(5.0)
                .margin(4.0),
        ))
        .style(move |theme, status| scrollbar_style(theme, status, true))
        .width(Length::Fill)
        .height(Length::Fixed(metrics.month_height))
        .into()
}

fn month_column(
    month: crate::domain::SeasonCalendarMonth,
    layout: LayoutConfig,
    metrics: CalendarGridMetrics,
) -> Element<'static, Message> {
    let title = text(month_title(month.year, month.month))
        .size(layout.text(12))
        .color(text_color());

    let weekday_row = row(WEEKDAYS
        .iter()
        .map(|label| weekday_heading(label, layout, metrics)))
    .spacing(GRID_GAP as u16)
    .width(Length::Fixed(metrics.grid_width))
    .height(Length::Fixed(metrics.weekday_height));

    let mut grid = column![].spacing(GRID_GAP as u16);
    let mut week = row![].spacing(GRID_GAP as u16);
    let mut cells_in_week = 0usize;

    let mut week_count = 0usize;

    for _ in 0..month.leading_blanks {
        week = week.push(empty_cell(metrics));
        cells_in_week += 1;
    }

    for day in month.days {
        if cells_in_week == 7 {
            grid = grid.push(finish_week_row(week, metrics));
            week = row![].spacing(GRID_GAP as u16);
            cells_in_week = 0;
            week_count += 1;
        }
        week = week.push(day_cell(day, layout, metrics));
        cells_in_week += 1;
    }

    if cells_in_week > 0 {
        while cells_in_week < 7 {
            week = week.push(empty_cell(metrics));
            cells_in_week += 1;
        }
        grid = grid.push(finish_week_row(week, metrics));
        week_count += 1;
    }

    while week_count < WEEK_ROWS {
        grid = grid.push(empty_week_row(metrics));
        week_count += 1;
    }

    container(
        column![
            title,
            Space::with_height(6),
            weekday_row,
            Space::with_height(4),
            grid
        ]
        .spacing(0)
        .width(Length::Fixed(metrics.grid_width))
        .height(Length::Shrink)
        .align_x(Alignment::Start),
    )
    .padding(MONTH_PAD)
    .width(Length::Fixed(metrics.month_width))
    .height(Length::Fixed(metrics.month_height))
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

fn finish_week_row(
    week: iced::widget::Row<'static, Message>,
    metrics: CalendarGridMetrics,
) -> Element<'static, Message> {
    week.width(Length::Fixed(metrics.grid_width))
        .height(Length::Fixed(metrics.cell))
        .into()
}

fn empty_week_row(metrics: CalendarGridMetrics) -> Element<'static, Message> {
    let week = row((0..7).map(|_| empty_cell(metrics))).spacing(GRID_GAP as u16);
    finish_week_row(week, metrics)
}

fn weekday_heading(
    label: &'static str,
    layout: LayoutConfig,
    metrics: CalendarGridMetrics,
) -> Element<'static, Message> {
    container(
        text(label)
            .size(layout.text(10))
            .color(muted())
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
    )
    .width(Length::Fixed(metrics.cell))
    .height(Length::Fixed(metrics.weekday_height))
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .into()
}

fn day_cell(
    day: SeasonCalendarDay,
    layout: LayoutConfig,
    metrics: CalendarGridMetrics,
) -> Element<'static, Message> {
    let highlighted = day.phase.is_some();
    let phase = day.phase;

    let cell = container(
        text(format!("{}", day.date.day()))
            .size(layout.text(if highlighted { 12 } else { 11 }))
            .color(if highlighted {
                phase_text_color(phase)
            } else {
                muted()
            }),
    )
    .width(Length::Fixed(metrics.cell))
    .height(Length::Fixed(metrics.cell))
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |_| container::Style {
        background: phase
            .map(phase_fill_color)
            .filter(|_| highlighted)
            .map(Into::into),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    });

    tooltip(
        cell,
        day_popover(day, layout),
        tooltip::Position::Top,
    )
    .gap(4)
    .snap_within_viewport(true)
    .into()
}

fn empty_cell(metrics: CalendarGridMetrics) -> Element<'static, Message> {
    Space::new(Length::Fixed(metrics.cell), Length::Fixed(metrics.cell)).into()
}

fn day_popover(day: SeasonCalendarDay, layout: LayoutConfig) -> Element<'static, Message> {
    let size = layout.text(12);
    let mut rows = column![].spacing(4);

    if day.sessions.is_empty() {
        rows = rows.push(text("No session scheduled").size(size).color(muted()));
    } else {
        if let (Some(name), Some(round)) = (day.meeting_name, day.round) {
            rows = rows.push(
                text(format!("{name} · R{round:02}"))
                    .size(size)
                    .color(text_color()),
            );
        }
        for session in day.sessions {
            rows = rows.push(
                text(format_session_line(&session))
                    .size(size)
                    .color(muted()),
            );
        }
    }

    container(rows)
        .padding([8, 10])
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

fn format_session_line(session: &DaySession) -> String {
    let local = session.starts_at.with_timezone(&Local);
    format!(
        "{} · {}",
        session.session_name,
        local.format("%a %d %b · %H:%M")
    )
}

fn month_title(year: i32, month: u32) -> String {
    let date = chrono::NaiveDate::from_ymd_opt(year, month, 1).expect("valid month");
    date.format("%b %Y").to_string()
}

fn phase_color(phase: RacePhase) -> Color {
    match phase {
        RacePhase::Past => muted(),
        RacePhase::Current => live(),
        RacePhase::Future => accent(),
    }
}

fn phase_text_color(phase: Option<RacePhase>) -> Color {
    match phase {
        Some(RacePhase::Past) => text_color(),
        Some(RacePhase::Current) => live(),
        Some(RacePhase::Future) => accent(),
        None => muted(),
    }
}

fn phase_fill_color(phase: RacePhase) -> Color {
    let base = phase_color(phase);
    Color { a: 0.22, ..base }
}
