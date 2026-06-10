use iced::widget::{column, container, row, text, Space};
use iced::{Color, Element, Length};

use crate::state::Message;
use crate::ui::icons::{icon, Icon};
use crate::ui::theme::{
    CHECKER_DARK, CHECKER_LIGHT, FLAG_BLACK, FLAG_GREEN, FLAG_RED,
    FLAG_YELLOW, SURFACE, TEXT,
};

const CELL: f32 = 8.0;

/// Slow animation tick derived from the 50ms frame counter (~1.5s per step).
fn slow_phase(frame: u32) -> u32 {
    frame / 30
}

use crate::assets::header_brand;

pub fn header_branding() -> Element<'static, Message> {
    header_brand()
}

fn checkered_patch(cols: usize, rows: usize, shift: u32) -> Element<'static, Message> {
    column((0..rows).map(|row| checkered_row(cols, row, shift)))
        .spacing(0)
        .into()
}

fn checkered_row(cols: usize, row_index: usize, shift: u32) -> Element<'static, Message> {
    row((0..cols).map(|col| {
        let dark = (col as u32 + row_index as u32 + shift) % 2 == 0;
        checkered_cell(dark)
    }))
    .spacing(0)
    .into()
}

fn checkered_cell(dark: bool) -> Element<'static, Message> {
    container(Space::new(Length::Fixed(CELL), Length::Fixed(CELL)))
        .width(Length::Fixed(CELL))
        .height(Length::Fixed(CELL))
        .style(move |_| container::Style {
            background: Some(if dark { CHECKER_DARK } else { CHECKER_LIGHT }.into()),
            ..Default::default()
        })
        .into()
}

#[derive(Debug, Clone, Copy)]
pub enum FlagSignal {
    Live,
    Intermission,
    Next,
    Finished,
    Alert,
}

pub fn signal_flag<'a>(signal: FlagSignal, label: &'a str) -> Element<'a, Message> {
    signal_flag_sized(signal, label, 11)
}

pub fn signal_flag_sized<'a>(
    signal: FlagSignal,
    label: &'a str,
    label_size: u16,
) -> Element<'a, Message> {
    let color = match signal {
        FlagSignal::Live => FLAG_GREEN,
        FlagSignal::Intermission => FLAG_YELLOW,
        FlagSignal::Next => FLAG_GREEN,
        FlagSignal::Finished => FLAG_BLACK,
        FlagSignal::Alert => FLAG_RED,
    };

    container(
        row![solid_swatch(color), text(label).size(label_size).color(color)]
            .spacing(6)
            .align_y(iced::Alignment::Center),
    )
    .padding([5, 10])
    .style(move |_| container::Style {
        background: Some(SURFACE.into()),
        border: iced::Border {
            color,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn solid_swatch(color: Color) -> Element<'static, Message> {
    container(Space::new(Length::Fixed(10.0), Length::Fixed(14.0)))
        .width(Length::Fixed(10.0))
        .height(Length::Fixed(14.0))
        .style(move |_| container::Style {
            background: Some(color.into()),
            ..Default::default()
        })
        .into()
}

fn centered_checkered(cols: usize, rows: usize, shift: u32) -> Element<'static, Message> {
    container(checkered_patch(cols, rows, shift))
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .into()
}

pub fn intermission_panel(
    frame: u32,
    subtitle: &'static str,
    layout: crate::ui::layout::LayoutConfig,
) -> Element<'static, Message> {
    let slow = slow_phase(frame);
    let pulse_icon = if slow % 2 == 0 {
        Icon::CircleFilled
    } else {
        Icon::Circle
    };
    let title_size = layout.meeting_title_size.saturating_add(2);
    let pulse_size = layout.card_body_size.saturating_add(6) as f32;

    container(
        column![
            centered_checkered(12, 2, slow),
            Space::with_height(10),
            icon(pulse_icon, pulse_size, FLAG_YELLOW),
            text("INTERMISSION").size(title_size).color(FLAG_YELLOW),
            text(subtitle).size(layout.card_detail_size).color(TEXT),
            Space::with_height(8),
            centered_checkered(12, 2, slow),
        ]
        .spacing(6)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center),
    )
    .padding(14)
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(
            Color {
                a: 0.35,
                ..FLAG_YELLOW
            }
            .into(),
        ),
        border: iced::Border {
            color: FLAG_YELLOW,
            width: 1.5,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}
