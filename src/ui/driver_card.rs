use iced::widget::{button, column, container, image, row, text, Space};
use iced::{alignment, Element, Length};

use openf1::Driver;

use crate::domain::{driver_display_name, driver_flag_iso2, driver_flag_url, team_colour, team_logo_url, PinDirection};
use crate::state::{AppState, Message};
use crate::ui::components::icon_button_group_sized;
use crate::ui::icons::{icon, icon_label, Icon};
use crate::ui::layout::LayoutConfig;
use crate::ui::theme::{accent, border, muted, surface, text_color};

const HEADSHOT_SIZE: f32 = 112.0;
const FLAG_BADGE_WIDTH: f32 = 30.0;
const FLAG_BADGE_HEIGHT: f32 = 22.0;
const FLAG_IMAGE_WIDTH: f32 = 24.0;
const FLAG_IMAGE_HEIGHT: f32 = 18.0;
const LOGO_BADGE_SIZE: f32 = 34.0;
const LOGO_IMAGE_SIZE: f32 = 28.0;
const BADGE_PADDING: f32 = 2.0;
const BADGE_RADIUS: f32 = 5.0;
const BADGE_BG: iced::Color = iced::Color::from_rgb(0.96, 0.96, 0.98);
const BADGE_BORDER: iced::Color = iced::Color::from_rgb(0.82, 0.84, 0.88);
const BADGE_TEXT: iced::Color = iced::Color::from_rgb(0.28, 0.30, 0.36);
const CORNER_RADIUS: f32 = 10.0;
const BORDER_WIDTH: f32 = 3.0;
const ROW_PADDING: [u16; 2] = [12, 16];
const DRIVER_ACRONYM_SIZE: u16 = 12;
const REORDER_ICON_SIZE: f32 = 22.0;

pub fn driver_portrait(state: &AppState, driver: &Driver) -> Element<'static, Message> {
    driver_portrait_sized(state, driver, HEADSHOT_SIZE)
}

pub fn rival_driver_portrait(state: &AppState, driver: &Driver) -> Element<'static, Message> {
    driver_portrait_sized(state, driver, 128.0)
}

fn driver_portrait_sized(
    state: &AppState,
    driver: &Driver,
    size: f32,
) -> Element<'static, Message> {
    let colour = team_colour(&driver.team_colour);
    let url = driver.headshot_url.clone();
    let label = driver_display_name(driver).to_string();
    let inner = size - BORDER_WIDTH * 2.0;
    let fallback_size = (size * 0.25).max(18.0) as u16;

    let content: Element<'static, Message> = if !url.is_empty() {
        if let Some(handle) = state.headshot_handle(&url) {
            image(handle)
                .width(Length::Fixed(inner))
                .height(Length::Fixed(inner))
                .content_fit(iced::ContentFit::Cover)
                .into()
        } else {
            portrait_fallback(&label, colour, inner, fallback_size)
        }
    } else {
        portrait_fallback(&label, colour, inner, fallback_size)
    };

    container(content)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .padding(BORDER_WIDTH as u16)
        .clip(true)
        .style(move |_| container::Style {
            background: Some(surface().into()),
            border: iced::Border {
                color: colour,
                width: BORDER_WIDTH,
                radius: CORNER_RADIUS.into(),
            },
            ..Default::default()
        })
        .into()
}

fn portrait_fallback(
    label: &str,
    colour: iced::Color,
    size: f32,
    font_size: u16,
) -> Element<'static, Message> {
    container(text(label.to_string()).size(font_size).color(colour))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
}

fn icon_badge(
    content: Element<'static, Message>,
    width: f32,
    height: f32,
) -> Element<'static, Message> {
    container(content)
        .padding(BADGE_PADDING as u16)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(|_| container::Style {
            background: Some(BADGE_BG.into()),
            border: iced::Border {
                color: BADGE_BORDER,
                width: 1.0,
                radius: BADGE_RADIUS.into(),
            },
            ..Default::default()
        })
        .into()
}

fn cached_image(
    state: &AppState,
    url: &str,
    width: f32,
    height: f32,
) -> Option<Element<'static, Message>> {
    state.flag_handle(url).map(|handle| {
        image(handle)
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .content_fit(iced::ContentFit::Contain)
            .into()
    })
}

pub fn driver_nationality_flag(state: &AppState, driver: &Driver) -> Element<'static, Message> {
    let iso2 = driver_flag_iso2(
        &driver.country_code,
        driver.driver_number,
        &driver.name_acronym,
    );

    if let Some(url) = driver_flag_url(
        &driver.country_code,
        driver.driver_number,
        &driver.name_acronym,
    ) {
        if let Some(flag) = cached_image(state, &url, FLAG_IMAGE_WIDTH, FLAG_IMAGE_HEIGHT) {
            return icon_badge(flag, FLAG_BADGE_WIDTH, FLAG_BADGE_HEIGHT);
        }
    }

    let label = iso2
        .map(|code| code.to_ascii_uppercase())
        .unwrap_or_else(|| {
            if driver.country_code.is_empty() {
                "—".to_string()
            } else {
                driver.country_code.clone()
            }
        });

    icon_badge(
        text(label).size(9).color(BADGE_TEXT).into(),
        FLAG_BADGE_WIDTH,
        FLAG_BADGE_HEIGHT,
    )
}

pub fn team_logo(state: &AppState, driver: &Driver) -> Element<'static, Message> {
    let colour = team_colour(&driver.team_colour);

    if let Some(url) = team_logo_url(&driver.team_name) {
        if let Some(logo) = cached_image(state, &url, LOGO_IMAGE_SIZE, LOGO_IMAGE_SIZE) {
            return icon_badge(logo, LOGO_BADGE_SIZE, LOGO_BADGE_SIZE);
        }
    }

    let initial = driver
        .team_name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    icon_badge(
        text(initial).size(12).color(colour).into(),
        LOGO_BADGE_SIZE,
        LOGO_BADGE_SIZE,
    )
}

fn driver_identity_column(
    state: &AppState,
    driver: &Driver,
    name_size: u16,
    centered: bool,
) -> Element<'static, Message> {
    let acronym = driver.name_acronym.trim();
    let full_name = driver.full_name.clone();
    let team_name = driver.team_name.clone();
    let show_acronym = !acronym.is_empty() && !acronym.eq_ignore_ascii_case(&full_name);

    let mut identity = column![
        row![
            driver_nationality_flag(state, driver),
            Space::with_width(8),
            text(full_name).size(name_size).color(text_color()),
        ]
        .spacing(0)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(4);

    if show_acronym {
        identity = identity.push(
            text(acronym.to_uppercase())
                .size(DRIVER_ACRONYM_SIZE)
                .color(muted()),
        );
    }

    identity = identity.push(
        row![
            team_logo(state, driver),
            Space::with_width(8),
            text(team_name).size(11).color(muted()),
        ]
        .spacing(0)
        .align_y(iced::Alignment::Center),
    );

    identity
        .width(Length::Fill)
        .height(Length::Shrink)
        .align_x(if centered {
            iced::Alignment::Center
        } else {
            iced::Alignment::Start
        })
        .into()
}

pub fn driver_picker_row(state: &AppState, driver: Driver) -> Element<'static, Message> {
    let driver_number = driver.driver_number;

    if let Some(slot) = state.rival_pick_slot {
        let row_content = row![
            driver_portrait(state, &driver),
            Space::with_width(12),
            driver_identity_column(state, &driver, 18, false),
            Space::with_width(Length::Fill),
            text("Select").size(12).color(accent()),
        ]
        .align_y(iced::Alignment::Center)
        .width(Length::Fill);

        return button(
            container(row_content)
                .padding(ROW_PADDING)
                .width(Length::Fill)
                .style(|_| container::Style {
                    background: Some(surface().into()),
                    border: iced::Border {
                        color: border(),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }),
        )
        .width(Length::Fill)
        .padding(0)
        .on_press(Message::RivalDriverSelected {
            slot,
            driver_number,
        })
        .style(|_, status| pin_row_style(status))
        .into();
    }

    let pinned = state
        .pinned_drivers
        .iter()
        .any(|pin| pin.driver_number == driver.driver_number);

    let action: Element<'static, Message> = if pinned {
        row_action_button("Unpin", Message::UnpinDriver(driver_number))
    } else if state.can_add_pin() {
        icon_label(Icon::Pin, 13.0, accent(), "Pin", 12, accent()).into()
    } else {
        text("Full").size(11).color(muted()).into()
    };

    let row_content = row![
        driver_portrait(state, &driver),
        Space::with_width(12),
        driver_identity_column(state, &driver, 18, false),
        Space::with_width(8),
        container(action)
            .width(Length::Shrink)
            .align_x(iced::Alignment::Center),
    ]
    .align_y(iced::Alignment::Center)
    .width(Length::Fill);

    let framed = container(row_content)
        .padding(ROW_PADDING)
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
        });

    if !pinned && state.can_add_pin() {
        button(framed)
            .width(Length::Fill)
            .padding(0)
            .on_press(Message::PinDriver(driver_number))
            .style(|_, status| pin_row_style(status))
            .into()
    } else {
        framed.into()
    }
}

fn row_action_button(label: &'static str, message: Message) -> Element<'static, Message> {
    button(
        row![
            icon(Icon::PinOff, 14.0, accent()),
            text(label).size(13).color(accent()),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
        .padding([8, 12])
        .on_press(message)
        .style(|_, status| {
            let bg = if status == button::Status::Hovered {
                iced::Background::Color(iced::Color {
                    a: 0.18,
                    ..accent()
                })
            } else {
                iced::Background::Color(iced::Color::TRANSPARENT)
            };
            button::Style {
                background: Some(bg),
                text_color: accent(),
                border: iced::Border {
                    color: accent(),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn pin_row_style(status: button::Status) -> button::Style {
    let bg = if status == button::Status::Hovered {
        iced::Background::Color(iced::Color {
            a: 0.35,
            ..surface()
        })
    } else {
        iced::Background::Color(iced::Color::TRANSPARENT)
    };
    button::Style {
        background: Some(bg),
        text_color: text_color(),
        border: iced::Border {
            radius: 8.0.into(),
            width: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn pinned_driver_card(
    state: &AppState,
    driver: &Driver,
    index: usize,
    total: usize,
    layout: LayoutConfig,
) -> Element<'static, Message> {
    let driver_number = driver.driver_number;
    let portrait =
        driver_portrait_sized(state, driver, layout.pin_portrait_size);
    let identity =
        driver_identity_column(state, driver, layout.pin_name_size, layout.pin_card_stacked);

    let mut controls: Vec<(Icon, Message)> = Vec::new();
    if index > 0 {
        controls.push((Icon::ChevronLeft, Message::MovePin {
            driver_number,
            direction: PinDirection::Left,
        }));
    }
    if index + 1 < total {
        controls.push((Icon::ChevronRight, Message::MovePin {
            driver_number,
            direction: PinDirection::Right,
        }));
    }

    let reorder_controls: Element<'static, Message> = if controls.is_empty() {
        Space::new(Length::Shrink, Length::Fixed(0.0)).into()
    } else {
        icon_button_group_sized(&controls, REORDER_ICON_SIZE, [10, 14])
    };

    let unpin = row_action_button("Unpin", Message::UnpinDriver(driver_number));

    let body: Element<'static, Message> = if layout.pin_card_stacked {
        column![
            container(portrait).align_x(iced::Alignment::Center),
            identity,
            container(
                row![unpin, reorder_controls]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .align_x(iced::Alignment::Center),
        ]
        .spacing(10)
        .align_x(iced::Alignment::Center)
        .width(Length::Fill)
        .into()
    } else {
        row![
            portrait,
            Space::with_width(12),
            identity,
            Space::with_width(8),
            container(
                column![unpin, reorder_controls]
                    .spacing(8)
                    .align_x(iced::Alignment::Center),
            )
            .width(Length::Shrink)
            .align_x(iced::Alignment::Center),
        ]
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
        .into()
    };

    container(body)
        .padding(ROW_PADDING)
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
