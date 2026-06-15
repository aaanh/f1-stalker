use iced::alignment;
use iced::widget::canvas::{self, event, Canvas, Event, Fill, Frame, Geometry, Path, Stroke, Text};
use iced::widget::{button, column, container, row, text, Space};
use iced::{mouse, Color, Element, Length, Point, Rectangle, Size, Theme};

use crate::domain::{
    build_championship_charts, ChampionshipCharts, ChampionshipTab, ChartMode, ChartSeries,
    PositionAxis,
};
use crate::state::{
    AppState, ChampionshipLoadState, ChartHoverHit, ChartTickEntry, ChartTooltip, Message,
};
use crate::ui::components::secondary_button_icon;
use crate::ui::icons::{section_heading, subtitle_text, Icon};
use crate::ui::fonts::MONO;
use crate::ui::layout::{scale_px, scale_text, LayoutConfig};
use crate::ui::theme::{accent, border, muted, surface, text_color};

const CHART_HEIGHT: f32 = 280.0;
const LABEL_EDGE_PAD: f32 = 6.0;
const LABEL_CHAR_WIDTH: f32 = 5.5;
const AXIS_FONT_SIZE: f32 = 10.0;
const ENTRY_LABEL_GAP: f32 = 10.0;

pub fn championship_charts_section(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let scale = layout.font_scale;
    let mode_tabs = row![
        mode_button("Championship", ChartMode::Championship, state.championship_chart_mode, scale),
        mode_button(
            "Race standing",
            ChartMode::RaceStanding,
            state.championship_chart_mode,
            scale,
        ),
    ]
    .spacing(8);

    let subject_tabs = row![
        tab_button("Drivers", ChampionshipTab::Drivers, state.championship_tab, scale),
        tab_button(
            "Constructors",
            ChampionshipTab::Constructors,
            state.championship_tab,
            scale,
        ),
    ]
    .spacing(8);

    let subtitle = match state.championship_chart_mode {
        ChartMode::Championship => {
            "Championship position by Grand Prix - updates after each race"
        }
        ChartMode::RaceStanding => {
            "Race finish position by Grand Prix - drivers use session results, constructors ranked by race points"
        }
    };

    let body = match &state.championship {
        ChampionshipLoadState::Loading => text("Loading championship data…")
            .size(scale_text(13, scale))
            .color(muted())
            .into(),
        ChampionshipLoadState::Error { message, cached: None } => column![
            text("Could not load championship data").size(scale_text(14, scale)),
            text(message).size(scale_text(12, scale)).color(muted()),
            Space::with_height(8),
            secondary_button_icon(Some(Icon::Refresh), "Retry", Message::Refresh),
        ]
        .spacing(6)
        .into(),
        ChampionshipLoadState::Ready(_) | ChampionshipLoadState::Error { cached: Some(_), .. } => {
            chart_body(state, scale)
        }
    };

    let stale_note = championship_stale_note(state, scale);

    let mut header = row![
        section_heading(
            Icon::Trophy,
            "Pinned drivers/constructors",
            Some(subtitle_text(subtitle)),
        ),
        Space::with_width(Length::Fill),
    ];
    if let Some(control) = rival_compare_control(state) {
        header = header.push(control).push(Space::with_width(8));
    }
    header = header.push(
        column![mode_tabs, subject_tabs]
            .spacing(8)
            .align_x(iced::Alignment::End),
    );

    container(
        column![
            header
                .align_y(iced::Alignment::Center)
                .width(Length::Fill),
            Space::with_height(12),
            body,
            stale_note,
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

fn rival_compare_control(state: &AppState) -> Option<Element<'_, Message>> {
    if state.rival_compare_active() {
        Some(secondary_button_icon(
            Some(Icon::Close),
            "Stop comparing on charts",
            Message::ExitRivalCompare,
        ))
    } else if state.rival_ready() {
        Some(secondary_button_icon(
            Some(Icon::Trophy),
            "Compare rivals",
            Message::ActivateRivalCompare,
        ))
    } else {
        None
    }
}

fn mode_button(
    label: &'static str,
    mode: ChartMode,
    active: ChartMode,
    scale: f32,
) -> Element<'static, Message> {
    tab_button_inner(label, mode == active, Message::ChampionshipChartModeSelected(mode), scale)
}

fn tab_button(
    label: &'static str,
    tab: ChampionshipTab,
    active: ChampionshipTab,
    scale: f32,
) -> Element<'static, Message> {
    tab_button_inner(label, tab == active, Message::ChampionshipTabSelected(tab), scale)
}

pub(crate) fn tab_button_inner(
    label: &'static str,
    selected: bool,
    message: Message,
    scale: f32,
) -> Element<'static, Message> {
    button(text(label).size(scale_text(13, scale)))
        .padding([8, 14])
        .on_press(message)
        .style(move |_, status| {
            use button::Status::{Active, Disabled, Hovered, Pressed};
            let (background, text_color, border_color) = if selected {
                (
                    iced::Background::Color(iced::Color { a: 0.35, ..accent() }),
                    text_color(),
                    accent(),
                )
            } else {
                match status {
                    Active => (iced::Background::Color(Color::TRANSPARENT), text_color(), border()),
                    Hovered => (
                        iced::Background::Color(Color { a: 0.35, ..surface() }),
                        text_color(),
                        accent(),
                    ),
                    Pressed => (
                        iced::Background::Color(Color { a: 0.55, ..surface() }),
                        text_color(),
                        accent(),
                    ),
                    Disabled => (
                        iced::Background::Color(Color::TRANSPARENT),
                        Color { a: 0.45, ..muted() },
                        border(),
                    ),
                }
            };
            button::Style {
                background: Some(background),
                text_color,
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn championship_stale_note(state: &AppState, scale: f32) -> Element<'static, Message> {
    let stale = match &state.championship {
        ChampionshipLoadState::Ready(loaded) => loaded.stale,
        ChampionshipLoadState::Error { cached: Some(loaded), .. } => loaded.stale,
        _ => false,
    };

    if stale {
        text("Showing cached standings · refresh to update.")
            .size(scale_text(12, scale))
            .color(muted())
            .into()
    } else {
        Space::new(Length::Shrink, Length::Fixed(0.0)).into()
    }
}

fn chart_body(state: &AppState, scale: f32) -> Element<'_, Message> {
    let Some(data) = state.championship_data() else {
        return text("Championship data unavailable")
            .size(scale_text(13, scale))
            .color(muted())
            .into();
    };

    let roster = state.drivers_roster().unwrap_or(&[]);
    let (meetings, sessions) = state
        .calendar()
        .map(|calendar| (calendar.meetings.as_slice(), calendar.sessions.as_slice()))
        .unwrap_or((&[], &[]));
    let rival_focus = if state.rival_compare_active() {
        let (first, second) = state.rival_drivers();
        Some([first, second])
    } else {
        None
    };
    let focus_drivers = rival_focus.as_ref().map(|pair| pair.as_slice());
    let charts = build_championship_charts(
        &data.rounds,
        &state.pinned_drivers,
        roster,
        meetings,
        sessions,
        focus_drivers,
    );

    let (series, axis) = match (state.championship_chart_mode, state.championship_tab) {
        (ChartMode::Championship, ChampionshipTab::Drivers) => (
            &charts.driver_championship_series,
            charts.driver_championship_axis,
        ),
        (ChartMode::Championship, ChampionshipTab::Constructors) => (
            &charts.constructor_championship_series,
            charts.constructor_championship_axis,
        ),
        (ChartMode::RaceStanding, ChampionshipTab::Drivers) => (
            &charts.driver_race_series,
            charts.driver_race_axis,
        ),
        (ChartMode::RaceStanding, ChampionshipTab::Constructors) => (
            &charts.constructor_race_series,
            charts.constructor_race_axis,
        ),
    };

    match state.championship_tab {
        ChampionshipTab::Drivers
            if state.pinned_drivers.is_empty() && !state.rival_compare_active() =>
        {
            column![
            empty_chart_axes(
                charts.round_count,
                PositionAxis { min: 1, max: 10 },
                &charts.round_labels,
                scale,
            ),
            Space::with_height(12),
            text("Pin drivers to see their progress on this chart.")
                .size(scale_text(13, scale))
                .color(muted()),
        ]
        .into()
        }
        ChampionshipTab::Drivers => render_chart(
            &charts,
            &charts.round_labels,
            series,
            axis,
            state.chart_tooltip.as_ref(),
            scale,
        ),
        ChampionshipTab::Constructors
            if state.pinned_drivers.is_empty() && !state.rival_compare_active() =>
        {
            column![
                empty_chart_axes(
                    charts.round_count.max(1),
                    PositionAxis { min: 1, max: 10 },
                    &charts.round_labels,
                    scale,
                ),
                Space::with_height(12),
                text("Pin drivers to see their constructors on this chart.")
                    .size(scale_text(13, scale))
                    .color(muted()),
            ]
            .into()
        }
        ChampionshipTab::Constructors if charts.round_count == 0 => {
            empty_message("No race data yet", scale).into()
        }
        ChampionshipTab::Constructors if charts.constructor_championship_series.is_empty() => {
            empty_message("No constructor data for pinned drivers", scale).into()
        }
        ChampionshipTab::Constructors => render_chart(
            &charts,
            &charts.round_labels,
            series,
            axis,
            state.chart_tooltip.as_ref(),
            scale,
        ),
    }
}

fn render_chart(
    charts: &ChampionshipCharts,
    round_labels: &[String],
    series: &[ChartSeries],
    axis: PositionAxis,
    tooltip: Option<&ChartTooltip>,
    scale: f32,
) -> Element<'static, Message> {
    if charts.round_count == 0 {
        return empty_message("No race data yet", scale);
    }

    if series.iter().all(|entry| entry.points.is_empty()) {
        return empty_message("No race data yet", scale);
    }

    column![
        position_chart(
            charts.round_count,
            axis,
            round_labels,
            series,
            tooltip,
            scale,
        ),
        Space::with_height(12),
        chart_legend(series, scale),
    ]
    .into()
}

fn empty_chart_axes(
    round_count: u32,
    axis: PositionAxis,
    round_labels: &[String],
    scale: f32,
) -> Element<'static, Message> {
    position_chart(round_count.max(1), axis, round_labels, &[], None, scale)
}

fn empty_message(message: &'static str, scale: f32) -> Element<'static, Message> {
    column![
        position_chart(1, PositionAxis { min: 1, max: 10 }, &[], &[], None, scale),
        Space::with_height(12),
        text(message).size(scale_text(13, scale)).color(muted()),
    ]
    .into()
}

fn position_chart(
    round_count: u32,
    axis: PositionAxis,
    round_labels: &[String],
    series: &[ChartSeries],
    tooltip: Option<&ChartTooltip>,
    scale: f32,
) -> Element<'static, Message> {
    Canvas::new(PositionChart {
        round_count: round_count.max(1),
        axis,
        round_labels: round_labels.to_vec(),
        series: series.to_vec(),
        tooltip: tooltip.cloned(),
        axis_font_size: scale_px(AXIS_FONT_SIZE, scale),
    })
    .width(Length::Fill)
    .height(Length::Fixed(CHART_HEIGHT))
    .into()
}

struct PositionChart {
    round_count: u32,
    axis: PositionAxis,
    round_labels: Vec<String>,
    series: Vec<ChartSeries>,
    tooltip: Option<ChartTooltip>,
    axis_font_size: f32,
}

impl canvas::Program<Message> for PositionChart {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: event::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (event::Status, Option<Message>) {
        let Event::Mouse(mouse_event) = event else {
            return (event::Status::Ignored, None);
        };

        match mouse_event {
            iced::mouse::Event::CursorMoved { .. }
            | iced::mouse::Event::CursorEntered { .. } => {
                let Some(position) = cursor.position_in(bounds) else {
                    return (event::Status::Ignored, Some(Message::ChampionshipChartHover(None)));
                };

                let layout = ChartLayout::new(bounds.size(), self.round_count, self.axis);
                let hit = hit_test_tick(position, &layout, &self.series);
                (
                    event::Status::Captured,
                    Some(Message::ChampionshipChartHover(hit)),
                )
            }
            iced::mouse::Event::CursorLeft => {
                (event::Status::Captured, Some(Message::ChampionshipChartHover(None)))
            }
            _ => (event::Status::Ignored, None),
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let layout = ChartLayout::new(bounds.size(), self.round_count, self.axis);

        draw_grid(&mut frame, &layout);
        draw_axes(&mut frame, &layout, &self.round_labels, self.axis_font_size);

        for series in &self.series {
            draw_series(&mut frame, &layout, series);
        }

        if let Some(tooltip) = &self.tooltip {
            draw_tick_hover(&mut frame, &layout, tooltip, self.axis_font_size);
        }

        vec![frame.into_geometry()]
    }
}

struct ChartLayout {
    plot: Rectangle,
    frame_width: f32,
    round_count: u32,
    axis: PositionAxis,
}

impl ChartLayout {
    fn new(size: iced::Size, round_count: u32, axis: PositionAxis) -> Self {
        let left = 36.0;
        let right = 20.0;
        let top = 16.0;
        let bottom = 36.0;
        Self {
            plot: Rectangle {
                x: left,
                y: top,
                width: (size.width - left - right).max(1.0),
                height: (size.height - top - bottom).max(1.0),
            },
            frame_width: size.width,
            round_count: round_count.max(1),
            axis,
        }
    }

    fn x_for_round(&self, round: u32) -> f32 {
        if self.round_count <= 1 {
            return self.plot.x + self.plot.width / 2.0;
        }
        let progress = (round.saturating_sub(1) as f32) / (self.round_count - 1) as f32;
        self.plot.x + progress * self.plot.width
    }

    fn y_for_position(&self, position: i64) -> f32 {
        let min = i64::from(self.axis.min);
        let max = i64::from(self.axis.max);
        let clamped = position.clamp(min, max) as f32;
        let progress = (clamped - min as f32) / (max as f32 - min as f32).max(1.0);
        self.plot.y + progress * self.plot.height
    }
}

fn should_label_position(axis: PositionAxis, position: u32) -> bool {
    let span = axis.max.saturating_sub(axis.min);
    if span <= 10 {
        return true;
    }
    position == axis.min || position == axis.max || (position - axis.min) % 5 == 0
}

fn draw_grid(frame: &mut Frame, layout: &ChartLayout) {
    let grid_color = Color { a: 0.35, ..border() };
    let stroke = Stroke::default().with_color(grid_color).with_width(1.0);

    for position in layout.axis.min..=layout.axis.max {
        if !should_label_position(layout.axis, position) {
            continue;
        }
        let y = layout.y_for_position(i64::from(position));
        let path = Path::line(
            Point::new(layout.plot.x, y),
            Point::new(layout.plot.x + layout.plot.width, y),
        );
        frame.stroke(&path, stroke);
    }

    for round in 1..=layout.round_count {
        let x = layout.x_for_round(round);
        let path = Path::line(
            Point::new(x, layout.plot.y),
            Point::new(x, layout.plot.y + layout.plot.height),
        );
        frame.stroke(&path, stroke);
    }
}

fn draw_axes(frame: &mut Frame, layout: &ChartLayout, round_labels: &[String], axis_font_size: f32) {
    let slot_width = if layout.round_count <= 1 {
        layout.plot.width
    } else {
        layout.plot.width / (layout.round_count - 1) as f32
    };
    for position in layout.axis.min..=layout.axis.max {
        if !should_label_position(layout.axis, position) {
            continue;
        }
        let y = layout.y_for_position(i64::from(position));
        frame.fill_text(Text {
            content: format!("P{position}"),
            position: Point::new(layout.plot.x - 8.0, y),
            color: muted(),
            size: iced::Pixels(axis_font_size),
            font: MONO,
            horizontal_alignment: alignment::Horizontal::Right,
            vertical_alignment: alignment::Vertical::Center,
            ..Text::default()
        });
    }

    for round in 1..=layout.round_count {
        let x = layout.x_for_round(round);
        let label = round_labels
            .get(round.saturating_sub(1) as usize)
            .map(String::as_str)
            .unwrap_or("");
        let (content, anchor_x, align) =
            axis_label_layout(x, label, slot_width, layout.frame_width);
        frame.fill_text(Text {
            content,
            position: Point::new(anchor_x, layout.plot.y + layout.plot.height + 14.0),
            color: muted(),
            size: iced::Pixels(axis_font_size),
            font: MONO,
            horizontal_alignment: align,
            vertical_alignment: alignment::Vertical::Top,
            ..Text::default()
        });
    }
}

fn axis_label_layout(
    tick_x: f32,
    label: &str,
    slot_width: f32,
    frame_width: f32,
) -> (String, f32, alignment::Horizontal) {
    let space_left = (tick_x - LABEL_EDGE_PAD).max(0.0);
    let space_right = (frame_width - LABEL_EDGE_PAD - tick_x).max(0.0);

    let (align, anchor_x, budget) = if space_right + 1.0 < space_left {
        (
            alignment::Horizontal::Right,
            tick_x.min(frame_width - LABEL_EDGE_PAD),
            space_right.max(slot_width * 0.45),
        )
    } else if space_left + 1.0 < space_right {
        (
            alignment::Horizontal::Left,
            tick_x.max(LABEL_EDGE_PAD),
            space_left.max(slot_width * 0.45),
        )
    } else {
        (
            alignment::Horizontal::Center,
            tick_x,
            (space_left.min(space_right) * 2.0).min(slot_width * 0.92),
        )
    };

    let content = truncate_axis_label(label, budget);
    (content, anchor_x, align)
}

fn side_label_box(tick_x: f32, label_width: f32, frame_width: f32) -> f32 {
    let right_box_x = tick_x + ENTRY_LABEL_GAP;
    if right_box_x + label_width <= frame_width - LABEL_EDGE_PAD {
        return right_box_x;
    }

    let left_box_x = tick_x - ENTRY_LABEL_GAP - label_width;
    left_box_x.max(LABEL_EDGE_PAD)
}

fn truncate_axis_label(label: &str, max_width: f32) -> String {
    let max_chars = (max_width / LABEL_CHAR_WIDTH).floor() as usize;
    if label.chars().count() <= max_chars.max(3) {
        return label.to_string();
    }

    let truncated: String = label.chars().take(max_chars.saturating_sub(1).max(1)).collect();
    format!("{truncated}…")
}

fn hit_test_tick(
    cursor: Point,
    layout: &ChartLayout,
    series: &[ChartSeries],
) -> Option<ChartHoverHit> {
    if cursor.y < layout.plot.y
        || cursor.y > layout.plot.y + layout.plot.height
        || cursor.x < layout.plot.x
        || cursor.x > layout.plot.x + layout.plot.width
    {
        return None;
    }

    let round = nearest_round(cursor.x, layout)?;
    let x = layout.x_for_round(round);
    let mut entries: Vec<ChartTickEntry> = series
        .iter()
        .filter_map(|entry| {
            entry.points.iter().find(|point| point.round == round).map(|point| {
                ChartTickEntry {
                    code: entry.code.clone(),
                    color: entry.color,
                    y: layout.y_for_position(point.position),
                }
            })
        })
        .collect();

    if entries.is_empty() {
        return None;
    }

    entries.sort_by(|left, right| {
        left.y
            .partial_cmp(&right.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Some(ChartHoverHit { round, x, entries })
}

fn nearest_round(cursor_x: f32, layout: &ChartLayout) -> Option<u32> {
    let mut best: Option<(f32, u32)> = None;
    for round in 1..=layout.round_count {
        let x = layout.x_for_round(round);
        let distance = (cursor_x - x).abs();
        if best
            .map(|(best_distance, _)| distance < best_distance)
            .unwrap_or(true)
        {
            best = Some((distance, round));
        }
    }

    let (distance, round) = best?;
    let tolerance = if layout.round_count <= 1 {
        layout.plot.width
    } else {
        layout.plot.width / (layout.round_count - 1) as f32 * 0.55
    };

    if distance <= tolerance {
        Some(round)
    } else {
        None
    }
}

fn draw_tick_hover(frame: &mut Frame, layout: &ChartLayout, tooltip: &ChartTooltip, axis_font_size: f32) {
    let band = Path::rectangle(
        Point::new(tooltip.x - 1.0, layout.plot.y),
        Size::new(2.0, layout.plot.height),
    );
    frame.fill(
        &band,
        Fill::from(Color {
            a: 0.28,
            ..accent()
        }),
    );

    for entry in &tooltip.entries {
        let point = Point::new(tooltip.x, entry.y);
        frame.fill(
            &Path::circle(point, 5.5),
            Fill::from(Color {
                a: 0.24,
                ..entry.color
            }),
        );
        frame.stroke(
            &Path::circle(point, 5.5),
            Stroke::default()
                .with_color(entry.color)
                .with_width(2.0),
        );
        draw_entry_label(frame, layout, tooltip.x, entry.y, entry, axis_font_size);
    }
}

fn draw_entry_label(
    frame: &mut Frame,
    layout: &ChartLayout,
    tick_x: f32,
    anchor_y: f32,
    entry: &ChartTickEntry,
    axis_font_size: f32,
) {
    let text_width = (entry.code.len() as f32 * 7.2 + 12.0).max(28.0);
    let height = 18.0;
    let x = side_label_box(tick_x, text_width, layout.frame_width);
    let y = anchor_y - height / 2.0;
    let background = Path::rectangle(Point::new(x, y), Size::new(text_width, height));

    frame.fill(
        &background,
        Fill::from(Color {
            a: 0.94,
            ..surface()
        }),
    );
    frame.stroke(
        &background,
        Stroke::default()
            .with_color(entry.color)
            .with_width(1.5),
    );
    frame.fill_text(Text {
        content: entry.code.clone(),
        position: Point::new(x + text_width / 2.0, anchor_y),
        color: text_color(),
        size: iced::Pixels(axis_font_size),
        font: MONO,
        horizontal_alignment: alignment::Horizontal::Center,
        vertical_alignment: alignment::Vertical::Center,
        ..Text::default()
    });
}

fn draw_series(frame: &mut Frame, layout: &ChartLayout, series: &ChartSeries) {
    if series.points.is_empty() {
        return;
    }

    let path = Path::new(|builder| {
        for (index, point) in series.points.iter().enumerate() {
            let position = Point::new(
                layout.x_for_round(point.round),
                layout.y_for_position(point.position),
            );
            if index == 0 {
                builder.move_to(position);
            } else {
                builder.line_to(position);
            }
        }
    });
    frame.stroke(
        &path,
        Stroke::default()
            .with_color(series.color)
            .with_width(2.0)
            .with_line_cap(canvas::LineCap::Round)
            .with_line_join(canvas::LineJoin::Round),
    );

    for point in &series.points {
        let position = Point::new(
            layout.x_for_round(point.round),
            layout.y_for_position(point.position),
        );
        frame.fill(&Path::circle(position, 3.5), Fill::from(series.color));
    }
}

fn chart_legend(series: &[ChartSeries], scale: f32) -> Element<'static, Message> {
    let mut items = row![].spacing(12).width(Length::Fill);
    for entry in series {
        let color = entry.color;
        let label = entry.label.clone();
        items = items.push(
            row![
                container(Space::new(Length::Fixed(10.0), Length::Fixed(10.0)))
                    .width(Length::Fixed(10.0))
                    .height(Length::Fixed(10.0))
                    .style(move |_| container::Style {
                        background: Some(color.into()),
                        border: iced::Border {
                            color: border(),
                            width: 1.0,
                            radius: 999.0.into(),
                        },
                        ..Default::default()
                    }),
                text(label).size(scale_text(11, scale)).color(text_color()),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        );
    }
    items.into()
}
