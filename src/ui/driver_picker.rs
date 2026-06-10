use iced::widget::{button, column, container, row, stack, text, text_input, Space};
use iced::{Element, Length};

use crate::domain::{organize_roster, DriverSortField};
use crate::state::{AppState, DriversLoadState, Message, Overlay};
use crate::ui::components::{modal_overlay, secondary_button_icon};
use crate::ui::icons::{icon, icon_label, Icon};
use crate::ui::fonts::MONO;
use crate::ui::scroll::vertical_scroll;
use crate::ui::driver_card::driver_picker_row;
use crate::ui::theme::{ACCENT, BORDER, MUTED, SURFACE, TEXT};

pub fn driver_picker_overlay(state: &AppState) -> Element<'_, Message> {
    let body: Element<Message> = match &state.drivers {
        DriversLoadState::Loading => text("Loading driver roster…")
            .size(13)
            .color(MUTED)
            .into(),
        DriversLoadState::Error { message, cached: None } => column![
            text("Could not load drivers").size(14).color(TEXT),
            text(message).size(12).color(MUTED),
        ]
        .spacing(6)
        .into(),
        DriversLoadState::Ready(_) | DriversLoadState::Error { cached: Some(_), .. } => {
            let Some(roster) = state.drivers_roster() else {
                return text("Driver roster unavailable.")
                    .size(13)
                    .color(MUTED)
                    .into();
            };

            if roster.is_empty() {
                text("No drivers returned for the latest session.")
                    .size(13)
                    .color(MUTED)
                    .into()
            } else {
                driver_list(state, roster)
            }
        }
    };

    let modal_width = (state.viewport.width - 80.0).clamp(520.0, 680.0);
    let pin_count = state.pinned_drivers.len();
    let max_pins = crate::state::MAX_PINNED_DRIVERS;

    let mut header_actions: Vec<Element<Message>> = Vec::new();
    if pin_count > 0 {
        header_actions.push(secondary_button_icon(
            Some(Icon::PinOff),
            "Unpin all",
            Message::UnpinAll,
        ));
        header_actions.push(Space::with_width(8).into());
    }
    header_actions.push(secondary_button_icon(
        Some(Icon::Close),
        "Close",
        Message::CloseOverlay,
    ));

    let card = container(
        column![
            row![
                icon_label(Icon::Pin, 20.0, TEXT, "Pin drivers", 20, TEXT),
                Space::with_width(Length::Fill),
                row(header_actions)
                    .spacing(0)
                    .align_y(iced::Alignment::Center),
            ]
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
            Space::with_height(8),
            text(format!(
                "{pin_count}/{max_pins} pinned · choose drivers from the latest season session."
            ))
            .size(12)
            .color(MUTED),
            Space::with_height(12),
            search_bar(state),
            Space::with_height(10),
            sort_controls(state),
            Space::with_height(12),
            body,
        ]
        .width(Length::Fill),
    )
    .padding(24)
    .width(Length::Fixed(modal_width))
    .style(|_| container::Style {
        background: Some(SURFACE.into()),
        border: iced::Border {
            color: BORDER,
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: iced::Shadow {
            color: iced::Color {
                a: 0.5,
                ..iced::Color::BLACK
            },
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    });

    modal_overlay(card.into())
}

fn search_bar(state: &AppState) -> Element<'_, Message> {
    container(
        row![
            icon(Icon::Search, 16.0, MUTED),
            text_input("Search by name, code, or team...", &state.driver_picker.search)
                .on_input(Message::DriverPickerSearch)
                .padding(10)
                .size(14)
                .width(Length::Fill),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(iced::Color {
            a: 0.35,
            ..SURFACE
        }.into()),
        border: iced::Border {
            color: BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn sort_controls(state: &AppState) -> Element<'_, Message> {
    let filters = &state.driver_picker;
    let grouped = filters.group_by_constructor;

    row![
        sort_button("First", DriverSortField::FirstName, filters),
        Space::with_width(6),
        sort_button("Last", DriverSortField::LastName, filters),
        Space::with_width(6),
        sort_button("Code", DriverSortField::Code, filters),
        Space::with_width(6),
        sort_button("Team", DriverSortField::Constructor, filters),
        Space::with_width(Length::Fill),
        group_button(grouped),
    ]
    .align_y(iced::Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn sort_button(
    label: &'static str,
    field: DriverSortField,
    filters: &crate::domain::DriverPickerFilters,
) -> Element<'static, Message> {
    let active = filters.sort_field == field;
    let caption = if active {
        format!("{label} {}", filters.sort_direction.arrow())
    } else {
        label.to_string()
    };

    button(text(caption).size(12).font(MONO).color(if active { TEXT } else { MUTED }))
        .padding([6, 10])
        .on_press(Message::DriverPickerSortField(field))
        .style(move |_, status| sort_chip_style(active, status))
        .into()
}

fn group_button(grouped: bool) -> Element<'static, Message> {
    let label = if grouped {
        "Grouped by team"
    } else {
        "Group by team"
    };

    button(text(label).size(12).color(if grouped { TEXT } else { MUTED }))
        .padding([6, 10])
        .on_press(Message::DriverPickerToggleGroup)
        .style(move |_, status| sort_chip_style(grouped, status))
        .into()
}

fn sort_chip_style(active: bool, status: button::Status) -> button::Style {
    let bg = match (active, status) {
        (true, _) => iced::Background::Color(iced::Color {
            a: 0.35,
            ..ACCENT
        }),
        (false, button::Status::Hovered) => iced::Background::Color(iced::Color {
            a: 0.35,
            ..SURFACE
        }),
        _ => iced::Background::Color(iced::Color::TRANSPARENT),
    };

    button::Style {
        background: Some(bg),
        text_color: if active { TEXT } else { MUTED },
        border: iced::Border {
            color: if active { ACCENT } else { BORDER },
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

fn driver_list(state: &AppState, roster: &[openf1::Driver]) -> Element<'static, Message> {
    let groups = organize_roster(roster, &state.driver_picker);
    let total_matches: usize = groups.iter().map(|group| group.drivers.len()).sum();

    if total_matches == 0 {
        return text("No drivers match your search.")
            .size(13)
            .color(MUTED)
            .into();
    }

    let mut items: Vec<Element<Message>> = Vec::new();
    let grouped = state.driver_picker.group_by_constructor;

    for group in &groups {
        if grouped {
            items.push(
                container(text(group.team_name.clone()).size(13).color(ACCENT))
                    .padding([8, 4])
                    .width(Length::Fill)
                    .into(),
            );
        }

        for driver in &group.drivers {
            items.push(driver_picker_row(state, driver.clone()));
        }

        if grouped {
            items.push(Space::with_height(4).into());
        }
    }

    let list_height = (state.viewport.height * 0.48).clamp(360.0, 520.0);
    vertical_scroll(
        column(items)
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into(),
        state.scrollbar_visible.visible,
    )
        .height(Length::Fixed(list_height))
        .width(Length::Fill)
        .into()
}

pub fn overlay_stack<'a>(
    main: Element<'a, Message>,
    overlay: Overlay,
    state: &'a AppState,
) -> Element<'a, Message> {
    match overlay {
        Overlay::None => main,
        Overlay::About => stack![main, crate::ui::about::about_overlay()].into(),
        Overlay::DriverPicker => stack![main, driver_picker_overlay(state)].into(),
    }
}
