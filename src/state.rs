use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use iced::widget::image;
use iced::widget::scrollable::RelativeOffset;
use iced::window;
use iced::{Color, Size};
use openf1::Driver;

use crate::data::CalendarData;
use crate::data::ChampionshipData;
use crate::data::DriversData;
use crate::data::WeekendDetailData;
use crate::domain::QualiGridVisibility;
use crate::domain::{ChampionshipTab, ChartMode, DriverPickerFilters, DriverSortField, PinDirection, SeasonPhase};
use crate::db::{PinnedDriver, Settings};

pub mod bootstrap;
pub use bootstrap::{BootState, BootStepId, BootStepStatus};

pub const SCROLLBAR_HIDE_DELAY_MS: u64 = 700;

#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollbarVisibility {
    pub visible: bool,
    hide_after_ticks: u32,
}

impl ScrollbarVisibility {
    pub fn on_scroll(&mut self, tick_ms: u64) {
        self.visible = true;
        self.hide_after_ticks =
            (SCROLLBAR_HIDE_DELAY_MS / tick_ms.max(1)).max(1) as u32;
    }

    pub fn on_tick(&mut self) {
        if !self.visible {
            return;
        }

        if self.hide_after_ticks > 0 {
            self.hide_after_ticks -= 1;
        }

        if self.hide_after_ticks == 0 {
            self.visible = false;
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    None,
    About,
    DriverPicker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowAction {
    Close,
    Minimize,
    Maximize,
    Fullscreen,
    Drag,
}

#[derive(Debug, Clone)]
pub struct LoadedDrivers {
    pub data: DriversData,
    pub stale: bool,
}

#[derive(Debug, Clone)]
pub enum DriversLoadState {
    Loading,
    Ready(LoadedDrivers),
    Error {
        message: String,
        cached: Option<LoadedDrivers>,
    },
}

#[derive(Debug, Clone)]
pub enum ChampionshipLoadState {
    Loading,
    Ready(LoadedChampionship),
    Error {
        message: String,
        cached: Option<LoadedChampionship>,
    },
}

#[derive(Debug, Clone)]
pub struct LoadedChampionship {
    pub data: ChampionshipData,
    pub stale: bool,
}

#[derive(Debug, Clone)]
pub struct LoadedWeekend {
    pub data: WeekendDetailData,
    pub stale: bool,
}

#[derive(Debug, Clone)]
pub enum WeekendLoadState {
    Loading,
    Ready(LoadedWeekend),
    Error {
        message: String,
        cached: Option<LoadedWeekend>,
    },
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub screen: Screen,
    pub overlay: Overlay,
    pub window_id: Option<window::Id>,
    pub load: LoadState,
    pub drivers: DriversLoadState,
    pub championship: ChampionshipLoadState,
    pub weekend: WeekendLoadState,
    pub championship_tab: ChampionshipTab,
    pub championship_chart_mode: ChartMode,
    pub settings: Settings,
    pub pinned_drivers: Vec<PinnedDriver>,
    pub refreshing: bool,
    pub drivers_refreshing: bool,
    pub championship_refreshing: bool,
    pub weekend_refreshing: bool,
    pub settings_notice: Option<String>,
    pub animation_phase: u32,
    pub flag_images: HashMap<String, image::Handle>,
    pub headshot_images: HashMap<String, image::Handle>,
    pub headshot_failed: HashSet<String>,
    pub asset_fetch_failed: HashSet<String>,
    pub viewport: Size,
    pub driver_picker: DriverPickerFilters,
    pub driver_picker_scroll: RelativeOffset,
    pub scrollbar_visible: ScrollbarVisibility,
    pub boot: BootState,
    pub chart_tooltip: Option<ChartTooltip>,
    pub title_bar_controls_hover: bool,
    pub rival_pick_slot: Option<u8>,
    pub show_first_run: bool,
    pub hidden_window_mode: Option<window::Mode>,
}

#[derive(Debug, Clone)]
pub struct ChartTickEntry {
    pub code: String,
    pub color: Color,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct ChartTooltip {
    pub round: u32,
    pub x: f32,
    pub target_x: f32,
    pub entries: Vec<ChartTickEntry>,
}

#[derive(Debug, Clone)]
pub struct ChartHoverHit {
    pub round: u32,
    pub x: f32,
    pub entries: Vec<ChartTickEntry>,
}

pub fn apply_chart_hover(state: &mut AppState, hit: Option<ChartHoverHit>) {
    match hit {
        None => state.chart_tooltip = None,
        Some(hit) => match &mut state.chart_tooltip {
            None => {
                state.chart_tooltip = Some(ChartTooltip {
                    round: hit.round,
                    x: hit.x,
                    target_x: hit.x,
                    entries: hit.entries,
                });
            }
            Some(tooltip) => {
                tooltip.round = hit.round;
                tooltip.target_x = hit.x;
                tooltip.entries = hit.entries;
            }
        },
    }
}

pub fn animate_chart_tooltip(tooltip: &mut ChartTooltip) {
    const SMOOTH: f32 = 0.34;
    tooltip.x += (tooltip.target_x - tooltip.x) * SMOOTH;
}

#[derive(Debug, Clone)]
pub struct LoadedCalendar {
    pub data: CalendarData,
    pub stale: bool,
}

#[derive(Debug, Clone)]
pub enum LoadState {
    Loading,
    Ready(LoadedCalendar),
    Error {
        message: String,
        cached: Option<LoadedCalendar>,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    Fetched(Result<CalendarData, String>),
    DriversFetched(Result<DriversData, String>),
    ChampionshipFetched(Result<ChampionshipData, String>),
    WeekendFetched(Result<WeekendDetailData, String>),
    ChampionshipTabSelected(ChampionshipTab),
    ChampionshipChartModeSelected(ChartMode),
    StandingsTabSelected(ChampionshipTab),
    StandingsModeSelected(ChartMode),
    ChampionshipChartHover(Option<ChartHoverHit>),
    Tick,
    WindowResized(Size),
    WindowReady(window::Id),
    WindowCloseRequested(window::Id),
    HideToBackground(window::Mode),
    WindowAction(WindowAction),
    TitleBarPressed,
    TitleBarReleased,
    TitleBarMoved,
    TitleBarControlsHover(bool),
    FontScaleDelta(i8),
    FontScaleReset,
    ThemeSelected(crate::ui::theme::ThemePresetId),
    ActivateRivalCompare,
    ExitRivalCompare,
    RivalDriverSelected {
        slot: u8,
        driver_number: i64,
    },
    CompleteFirstRun,
    Navigate(Screen),
    OpenAbout,
    OpenDriverPicker,
    OpenRivalPicker(u8),
    CloseOverlay,
    OverlayClick,
    ClearCache,
    RebuildDatabase,
    PinDriver(i64),
    UnpinDriver(i64),
    UnpinAll,
    MovePin {
        driver_number: i64,
        direction: PinDirection,
    },
    FlagLoaded {
        url: String,
        result: Result<Vec<u8>, String>,
    },
    HeadshotLoaded {
        url: String,
        result: Result<Vec<u8>, String>,
    },
    DriverPickerSearch(String),
    DriverPickerSortField(DriverSortField),
    DriverPickerToggleGroup,
    ScrollInteraction,
    DriverPickerScroll(RelativeOffset),
    CopyDebugLog,
    SettingsToggled(SettingsToggle),
    ShowFromTray,
    QuitFromTray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsToggle {
    IncludeTesting,
    BackgroundOnClose,
    NotificationsEnabled,
    NotifyStandings,
    NotifySessions,
}

impl AppState {
    pub fn calendar(&self) -> Option<&CalendarData> {
        match &self.load {
            LoadState::Ready(loaded) => Some(&loaded.data),
            LoadState::Error { cached: Some(loaded), .. } => Some(&loaded.data),
            _ => None,
        }
    }

    pub fn drivers_roster(&self) -> Option<&[Driver]> {
        match &self.drivers {
            DriversLoadState::Ready(loaded) => Some(&loaded.data.drivers),
            DriversLoadState::Error { cached: Some(loaded), .. } => Some(&loaded.data.drivers),
            _ => None,
        }
    }

    pub fn championship_data(&self) -> Option<&ChampionshipData> {
        match &self.championship {
            ChampionshipLoadState::Ready(loaded) => Some(&loaded.data),
            ChampionshipLoadState::Error { cached: Some(loaded), .. } => Some(&loaded.data),
            _ => None,
        }
    }

    pub fn weekend_data(&self) -> Option<&WeekendDetailData> {
        match &self.weekend {
            WeekendLoadState::Ready(loaded) => Some(&loaded.data),
            WeekendLoadState::Error { cached: Some(loaded), .. } => Some(&loaded.data),
            _ => None,
        }
    }

    pub fn quali_visibility(&self) -> QualiGridVisibility {
        self.weekend_data()
            .map(|data| data.quali_visibility)
            .unwrap_or(QualiGridVisibility::Hidden)
    }

    pub fn sprint_visibility(&self) -> QualiGridVisibility {
        self.weekend_data()
            .map(|data| data.sprint_visibility)
            .unwrap_or(QualiGridVisibility::Hidden)
    }

    pub fn rival_drivers(&self) -> (i64, i64) {
        (
            self.settings.rival_driver_first,
            self.settings.rival_driver_second,
        )
    }

    pub fn rival_ready(&self) -> bool {
        let (first, second) = self.rival_drivers();
        first > 0 && second > 0 && first != second
    }

    pub fn rival_compare_active(&self) -> bool {
        self.settings.rival_compare_active && self.rival_ready()
    }

    pub fn is_stale(&self) -> bool {
        self.is_any_stale()
    }

    pub fn is_any_stale(&self) -> bool {
        calendar_stale(self)
            || drivers_stale(self)
            || championship_stale(self)
            || weekend_stale(self)
    }

    pub fn last_fetched_at(&self) -> Option<DateTime<Utc>> {
        [
            calendar_fetched_at(self),
            drivers_fetched_at(self),
            championship_fetched_at(self),
            weekend_fetched_at(self),
        ]
        .into_iter()
        .flatten()
        .max()
    }

    pub fn season_phase(&self) -> Option<SeasonPhase> {
        let data = self.calendar()?;
        Some(crate::domain::season_phase(
            &data.triplet,
            &data.countdown,
        ))
    }

    pub fn flag_handle(&self, url: &str) -> Option<image::Handle> {
        self.flag_images.get(url).cloned()
    }

    pub fn asset_fetch_failed(&self, url: &str) -> bool {
        self.asset_fetch_failed.contains(url)
    }

    pub fn headshot_handle(&self, url: &str) -> Option<image::Handle> {
        self.headshot_images.get(url).cloned()
    }

    pub fn headshot_failed(&self, url: &str) -> bool {
        self.headshot_failed.contains(url)
    }

}

fn calendar_stale(state: &AppState) -> bool {
    match &state.load {
        LoadState::Ready(loaded) => loaded.stale,
        LoadState::Error { cached: Some(loaded), .. } => loaded.stale,
        _ => false,
    }
}

fn drivers_stale(state: &AppState) -> bool {
    match &state.drivers {
        DriversLoadState::Ready(loaded) => loaded.stale,
        DriversLoadState::Error { cached: Some(loaded), .. } => loaded.stale,
        _ => false,
    }
}

fn championship_stale(state: &AppState) -> bool {
    match &state.championship {
        ChampionshipLoadState::Ready(loaded) => loaded.stale,
        ChampionshipLoadState::Error { cached: Some(loaded), .. } => loaded.stale,
        _ => false,
    }
}

fn weekend_stale(state: &AppState) -> bool {
    match &state.weekend {
        WeekendLoadState::Ready(loaded) => loaded.stale,
        WeekendLoadState::Error { cached: Some(loaded), .. } => loaded.stale,
        _ => false,
    }
}

fn calendar_fetched_at(state: &AppState) -> Option<DateTime<Utc>> {
    match &state.load {
        LoadState::Ready(loaded) => Some(loaded.data.fetched_at),
        LoadState::Error { cached: Some(loaded), .. } => Some(loaded.data.fetched_at),
        _ => None,
    }
}

fn drivers_fetched_at(state: &AppState) -> Option<DateTime<Utc>> {
    match &state.drivers {
        DriversLoadState::Ready(loaded) => Some(loaded.data.fetched_at),
        DriversLoadState::Error { cached: Some(loaded), .. } => Some(loaded.data.fetched_at),
        _ => None,
    }
}

fn championship_fetched_at(state: &AppState) -> Option<DateTime<Utc>> {
    match &state.championship {
        ChampionshipLoadState::Ready(loaded) => Some(loaded.data.fetched_at),
        ChampionshipLoadState::Error { cached: Some(loaded), .. } => Some(loaded.data.fetched_at),
        _ => None,
    }
}

fn weekend_fetched_at(state: &AppState) -> Option<DateTime<Utc>> {
    match &state.weekend {
        WeekendLoadState::Ready(loaded) => Some(loaded.data.fetched_at),
        WeekendLoadState::Error { cached: Some(loaded), .. } => Some(loaded.data.fetched_at),
        _ => None,
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Dashboard,
            overlay: Overlay::None,
            window_id: None,
            load: LoadState::Loading,
            drivers: DriversLoadState::Loading,
            championship: ChampionshipLoadState::Loading,
            weekend: WeekendLoadState::Loading,
            championship_tab: ChampionshipTab::default(),
            championship_chart_mode: ChartMode::default(),
            settings: Settings::default(),
            pinned_drivers: Vec::new(),
            refreshing: false,
            drivers_refreshing: false,
            championship_refreshing: false,
            weekend_refreshing: false,
            settings_notice: None,
            animation_phase: 0,
            flag_images: HashMap::new(),
            headshot_images: HashMap::new(),
            headshot_failed: HashSet::new(),
            asset_fetch_failed: HashSet::new(),
            viewport: Size::new(1100.0, 780.0),
            driver_picker: DriverPickerFilters::default(),
            driver_picker_scroll: RelativeOffset::START,
            scrollbar_visible: ScrollbarVisibility::default(),
            boot: BootState::new(),
            chart_tooltip: None,
            title_bar_controls_hover: false,
            rival_pick_slot: None,
            show_first_run: false,
            hidden_window_mode: None,
        }
    }
}
