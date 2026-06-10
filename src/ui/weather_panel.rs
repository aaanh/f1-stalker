use iced::widget::{column, row, text, Space};
use iced::Element;

use openf1::Meeting;

use crate::domain::{
    format_forecast_summary, format_track_summary, ForecastState, TrackState, WeatherPanel,
};
use crate::state::{AppState, Message};
use crate::ui::layout::LayoutConfig;
use crate::ui::theme::{FLAG_BLUE, FLAG_GREEN, FLAG_YELLOW, MUTED};

pub fn meeting_weather_panel<'a>(
    state: &'a AppState,
    meeting: &Meeting,
    layout: LayoutConfig,
) -> Element<'a, Message> {
    let panel = state
        .weekend_data()
        .and_then(|data| data.weather_by_meeting.get(&meeting.meeting_key));

    let forecast = forecast_column(panel, layout);
    let track = track_column(panel, layout);

    if layout.stack_cards {
        column![forecast, track].spacing(10).into()
    } else {
        row![forecast, Space::with_width(16), track].into()
    }
}

fn forecast_column(
    panel: Option<&WeatherPanel>,
    layout: LayoutConfig,
) -> Element<'static, Message> {
    let label_size = layout.card_detail_size.saturating_sub(1);
    let value_size = layout.card_detail_size;

    let body = match panel.map(|panel| &panel.forecast) {
        None => text("Loading…").size(value_size).color(MUTED),
        Some(ForecastState::Loading) => text("Loading…").size(value_size).color(MUTED),
        Some(ForecastState::Ready(forecast)) => {
            text(format_forecast_summary(forecast)).size(value_size).color(FLAG_BLUE)
        }
        Some(ForecastState::Error(message)) => {
            text(message.clone()).size(value_size).color(FLAG_YELLOW)
        }
    };

    column![
        text("Forecast").size(label_size).color(MUTED),
        body,
    ]
    .spacing(4)
    .into()
}

fn track_column(panel: Option<&WeatherPanel>, layout: LayoutConfig) -> Element<'static, Message> {
    let label_size = layout.card_detail_size.saturating_sub(1);
    let value_size = layout.card_detail_size;

    let body = match panel.map(|panel| &panel.track) {
        None => text("Loading…").size(value_size).color(MUTED),
        Some(TrackState::NoSessionData) => {
            text("No session data yet").size(value_size).color(MUTED)
        }
        Some(TrackState::Loading) => text("Loading…").size(value_size).color(MUTED),
        Some(TrackState::Ready(track)) => {
            text(format_track_summary(track)).size(value_size).color(FLAG_GREEN)
        }
        Some(TrackState::Error(message)) => {
            text(message.clone()).size(value_size).color(FLAG_YELLOW)
        }
    };

    column![
        text("Track").size(label_size).color(MUTED),
        body,
    ]
    .spacing(4)
    .into()
}
