use iced::widget::{container, scrollable};
use iced::widget::scrollable::{Direction, Id, RelativeOffset, Scrollbar, Status, Style};
use iced::{Color, Element, Length, Task, Theme};

use crate::state::Message;
use crate::ui::theme::accent;

pub fn driver_picker_scroll_id() -> Id {
    Id::new("driver-picker-scroll")
}

fn vertical_scrollbar() -> Scrollbar {
    Scrollbar::new()
        .width(5.0)
        .scroller_width(5.0)
        .margin(6.0)
}

fn scrollbar_rail(scroller_alpha: f32) -> scrollable::Rail {
    scrollable::Rail {
        background: None,
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        scroller: scrollable::Scroller {
            color: Color {
                r: 0.88,
                g: 0.88,
                b: 0.91,
                a: scroller_alpha,
            },
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 999.0.into(),
            },
        },
    }
}

fn scrollbar_interactive(status: Status) -> bool {
    matches!(
        status,
        Status::Hovered { .. } | Status::Dragged { .. }
    )
}

pub fn scrollbar_style(_theme: &Theme, status: Status, visible: bool) -> Style {
    if !visible && !scrollbar_interactive(status) {
        let hidden = scrollbar_rail(0.0);
        return Style {
            container: container::Style::default(),
            vertical_rail: hidden,
            horizontal_rail: hidden,
            gap: None,
        };
    }

    let mut vertical = scrollbar_rail(0.24);
    let mut horizontal = scrollbar_rail(0.24);

    match status {
        Status::Hovered {
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_hovered,
        } => {
            if is_vertical_scrollbar_hovered {
                vertical.scroller.color.a = 0.42;
            }
            if is_horizontal_scrollbar_hovered {
                horizontal.scroller.color.a = 0.42;
            }
        }
        Status::Dragged {
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_dragged,
        } => {
            if is_vertical_scrollbar_dragged {
                vertical.scroller.color = Color {
                    a: 0.85,
                    ..accent()
                };
            }
            if is_horizontal_scrollbar_dragged {
                horizontal.scroller.color = Color {
                    a: 0.85,
                    ..accent()
                };
            }
        }
        Status::Active => {}
    }

    Style {
        container: container::Style::default(),
        vertical_rail: vertical,
        horizontal_rail: horizontal,
        gap: None,
    }
}

pub fn vertical_scroll<'a>(
    content: Element<'a, Message>,
    visible: bool,
) -> scrollable::Scrollable<'a, Message, Theme> {
    scrollable(content)
        .direction(Direction::Vertical(vertical_scrollbar()))
        .style(move |theme, status| scrollbar_style(theme, status, visible))
        .on_scroll(|_| Message::ScrollInteraction)
}

pub fn driver_picker_scroll<'a>(
    content: Element<'a, Message>,
    visible: bool,
) -> scrollable::Scrollable<'a, Message, Theme> {
    scrollable(content)
        .id(driver_picker_scroll_id())
        .direction(Direction::Vertical(vertical_scrollbar()))
        .style(move |theme, status| scrollbar_style(theme, status, visible))
        .on_scroll(|viewport| Message::DriverPickerScroll(viewport.relative_offset()))
}

pub fn restore_driver_picker_scroll(offset: RelativeOffset) -> Task<Message> {
    scrollable::snap_to(driver_picker_scroll_id(), offset)
}

pub fn scrollable_page<'a>(
    scroll_id: Id,
    content: Element<'a, Message>,
    scrollbar_visible: bool,
) -> Element<'a, Message> {
    scrollable(content)
        .id(scroll_id)
        .direction(Direction::Vertical(vertical_scrollbar()))
        .style(move |theme, status| scrollbar_style(theme, status, scrollbar_visible))
        .width(Length::Fill)
        .height(Length::Fill)
        .on_scroll(|_| Message::ScrollInteraction)
        .into()
}
