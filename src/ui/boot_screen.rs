use iced::widget::{column, container, progress_bar, row, text, Space};
use iced::{Element, Length};

use crate::assets::boot_brand;
use crate::state::bootstrap::{BootState, BootStepStatus};
use crate::state::{AppState, Message};
use crate::ui::icons::{icon, Icon};
use crate::ui::theme::{ACCENT, BORDER, LIVE, MUTED, SURFACE, TEXT};

pub fn boot_screen(state: &AppState) -> Element<'_, Message> {
    let boot = &state.boot;

    container(
        column![
            boot_brand(),
            Space::with_height(8),
            text("Preparing your season dashboard").size(14).color(MUTED),
            Space::with_height(28),
            boot_card(boot),
        ]
        .align_x(iced::Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn boot_card(boot: &BootState) -> Element<'static, Message> {
    let progress = boot.progress();

    container(
        column![
            text(boot.current_label()).size(13).color(TEXT),
            Space::with_height(12),
            progress_bar(0.0..=1.0, progress),
            Space::with_height(6),
            text(format!("{:.0}%", progress * 100.0))
                .size(11)
                .color(MUTED),
            Space::with_height(20),
            column(boot.steps.iter().map(step_row).collect::<Vec<_>>()).spacing(8),
        ]
        .width(Length::Fill),
    )
    .padding(24)
    .width(Length::Fixed(460.0))
    .style(|_| container::Style {
        background: Some(SURFACE.into()),
        border: iced::Border {
            color: BORDER,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn step_row(step: &crate::state::bootstrap::BootStep) -> Element<'static, Message> {
    let (marker, marker_color, label_color) = match step.status {
        BootStepStatus::Done => (Icon::Check, LIVE, TEXT),
        BootStepStatus::Skipped => (Icon::Minus, MUTED, MUTED),
        BootStepStatus::Failed => (Icon::Alert, ACCENT, TEXT),
        BootStepStatus::Running => (Icon::Loader, ACCENT, TEXT),
        BootStepStatus::Pending => (Icon::Circle, MUTED, MUTED),
    };

    let detail = if step.detail.is_empty() {
        String::new()
    } else {
        format!(" - {}", step.detail)
    };

    row![
        container(icon(marker, 14.0, marker_color))
            .width(Length::Fixed(16.0))
            .align_x(iced::Alignment::Center),
        text(format!("{}{detail}", step.label))
            .size(12)
            .color(label_color),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .into()
}
