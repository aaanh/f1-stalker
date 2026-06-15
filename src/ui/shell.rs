use iced::widget::container;
use iced::{Element, Length};

use crate::state::{AppState, Message};
use crate::ui::boot_screen::boot_screen;
use crate::ui::dashboard::dashboard;
use crate::ui::driver_picker::overlay_stack;
use crate::ui::first_run::{first_run_overlay, should_show_first_run};
use crate::ui::layout::LayoutConfig;
use crate::ui::settings::settings_page;
use crate::ui::theme::bg;
use crate::ui::title_bar::title_bar;

pub fn shell(state: &AppState) -> Element<'_, Message> {
    if state.boot.active {
        return container(
            iced::widget::column![title_bar(state), boot_screen(state)]
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(bg().into()),
            ..Default::default()
        })
        .into();
    }

    let layout = LayoutConfig::from_viewport(state.viewport, state.settings.font_scale);

    let content: Element<Message> = match state.screen {
        crate::state::Screen::Dashboard => dashboard(state, layout),
        crate::state::Screen::Settings => settings_page(state, layout),
    };

    let main = container(
        iced::widget::column![title_bar(state), content]
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_| container::Style {
        background: Some(bg().into()),
        ..Default::default()
    });

    let stacked = overlay_stack(main.into(), state.overlay, state);
    if should_show_first_run(state) {
        iced::widget::stack![stacked, first_run_overlay(state)].into()
    } else {
        stacked
    }
}
