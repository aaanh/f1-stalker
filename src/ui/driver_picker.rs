use iced::widget::{button, column, container, row, stack, text, text_input, Space};
use iced::{Element, Length};

use crate::domain::{organize_roster, DriverSortField};
use crate::state::{AppState, DriversLoadState, Message, Overlay};
use crate::ui::components::{modal_overlay, secondary_button_icon};
use crate::ui::icons::{icon, icon_label, Icon};
use crate::ui::fonts::MONO;
use crate::ui::scroll::driver_picker_scroll;
use crate::ui::driver_card::driver_picker_row;
use crate::ui::layout::LayoutConfig;
use crate::ui::theme::{accent, border, muted, surface, text_color};

pub fn driver_picker_overlay(state: &AppState) -> Element<'_, Message> {
    let layout = LayoutConfig::from_viewport(state.viewport, state.settings.font_scale);

    let body: Element<Message> = match &state.drivers {
        DriversLoadState::Loading => text("Loading driver roster…")
            .size(layout.text(15))
            .color(muted())
            .into(),
        DriversLoadState::Error { message, cached: None } => column![
            text("Could not load drivers").size(layout.text(16)).color(text_color()),
            text(message).size(layout.text(14)).color(muted()),
        ]
        .spacing(6)
        .into(),
        DriversLoadState::Ready(_) | DriversLoadState::Error { cached: Some(_), .. } => {
            let Some(roster) = state.drivers_roster() else {
                return text("Driver roster unavailable.")
                    .size(layout.text(15))
                    .color(muted())
                    .into();
            };

            if roster.is_empty() {
                text("No drivers returned for the latest session.")
                    .size(layout.text(15))
                    .color(muted())
                    .into()
            } else {
                driver_list(state, roster, layout)
            }
        }
    };

    let modal_width = (state.viewport.width - 80.0).clamp(520.0, 680.0);
    let pin_count = state.pinned_drivers.len();

    let rival_pick = state.rival_pick_slot;
    let (title, subtitle) = if let Some(slot) = rival_pick {
        (
            format!("Choose rival driver {}", slot + 1),
            "Pick any driver from the season roster.".into(),
        )
    } else {
        (
            "Pin drivers".into(),
            format!(
                "{pin_count} pinned · choose drivers from the latest season session."
            ),
        )
    };

    let mut header_actions: Vec<Element<Message>> = Vec::new();
    if rival_pick.is_none() && pin_count > 0 {
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
                icon_label(
                    if rival_pick.is_some() {
                        Icon::Users
                    } else {
                        Icon::Pin
                    },
                    20.0,
                    text_color(),
                    title,
                    20,
                    text_color(),
                ),
                Space::with_width(Length::Fill),
                row(header_actions)
                    .spacing(0)
                    .align_y(iced::Alignment::Center),
            ]
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
            Space::with_height(8),
            text(subtitle).size(layout.text(14)).color(muted()),
            Space::with_height(12),
            search_bar(state, layout),
            Space::with_height(10),
            sort_controls(state, layout),
            Space::with_height(12),
            body,
        ]
        .width(Length::Fill),
    )
    .padding(24)
    .width(Length::Fixed(modal_width))
    .style(|_| container::Style {
        background: Some(surface().into()),
        border: iced::Border {
            color: border(),
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

fn search_bar(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    container(
        row![
            icon(Icon::Search, 16.0, muted()),
            text_input("Search by name, code, or team...", &state.driver_picker.search)
                .on_input(Message::DriverPickerSearch)
                .padding(10)
                .size(layout.text(15))
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
            ..surface()
        }.into()),
        border: iced::Border {
            color: border(),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn sort_controls(state: &AppState, layout: LayoutConfig) -> Element<'_, Message> {
    let filters = &state.driver_picker;
    let grouped = filters.group_by_constructor;

    row![
        sort_button("First", DriverSortField::FirstName, filters, layout),
        Space::with_width(6),
        sort_button("Last", DriverSortField::LastName, filters, layout),
        Space::with_width(6),
        sort_button("Code", DriverSortField::Code, filters, layout),
        Space::with_width(6),
        sort_button("Team", DriverSortField::Constructor, filters, layout),
        Space::with_width(Length::Fill),
        group_button(grouped, layout),
    ]
    .align_y(iced::Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn sort_button(
    label: &'static str,
    field: DriverSortField,
    filters: &crate::domain::DriverPickerFilters,
    layout: LayoutConfig,
) -> Element<'static, Message> {
    let active = filters.sort_field == field;
    let caption = if active {
        format!("{label} {}", filters.sort_direction.arrow())
    } else {
        label.to_string()
    };

    button(
        text(caption)
            .size(layout.text(13))
            .font(MONO)
            .color(if active { text_color() } else { muted() }),
    )
        .padding([6, 10])
        .on_press(Message::DriverPickerSortField(field))
        .style(move |_, status| sort_chip_style(active, status))
        .into()
}

fn group_button(grouped: bool, layout: LayoutConfig) -> Element<'static, Message> {
    let label = if grouped {
        "Grouped by team"
    } else {
        "Group by team"
    };

    button(
        text(label)
            .size(layout.text(13))
            .color(if grouped { text_color() } else { muted() }),
    )
        .padding([6, 10])
        .on_press(Message::DriverPickerToggleGroup)
        .style(move |_, status| sort_chip_style(grouped, status))
        .into()
}

fn sort_chip_style(active: bool, status: button::Status) -> button::Style {
    let bg = match (active, status) {
        (true, _) => iced::Background::Color(iced::Color {
            a: 0.35,
            ..accent()
        }),
        (false, button::Status::Hovered) => iced::Background::Color(iced::Color {
            a: 0.35,
            ..surface()
        }),
        _ => iced::Background::Color(iced::Color::TRANSPARENT),
    };

    button::Style {
        background: Some(bg),
        text_color: if active { text_color() } else { muted() },
        border: iced::Border {
            color: if active { accent() } else { border() },
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

fn driver_list(
    state: &AppState,
    roster: &[openf1::Driver],
    layout: LayoutConfig,
) -> Element<'static, Message> {
    let groups = organize_roster(roster, &state.driver_picker);
    let total_matches: usize = groups.iter().map(|group| group.drivers.len()).sum();

    if total_matches == 0 {
        return text("No drivers match your search.")
            .size(layout.text(15))
            .color(muted())
            .into();
    }

    let mut items: Vec<Element<Message>> = Vec::new();
    let grouped = state.driver_picker.group_by_constructor;

    for group in &groups {
        if grouped {
            items.push(
                container(text(group.team_name.clone()).size(layout.text(15)).color(accent()))
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
    driver_picker_scroll(
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
