use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use chrono::{Datelike, Utc};
use iced::time;
use iced::widget::image;
use iced::window::{self, Mode};
use iced::{clipboard, Element, Size, Subscription, Task};

use openf1::Session;

use crate::data::{
    assemble_weekend_from_cache, cache_is_fresh, championship_needs_refresh,
    fetch_season_calendar, fetch_season_championship, fetch_season_drivers,
    fetch_weekend_details, meetings_for_weather, quali_grid_needs_refresh,
    weekend_weather_needs_refresh, CalendarCacheBlob, CalendarData, ChampionshipCacheBlob,
    ChampionshipData, DriversCacheBlob, DriversData, ForecastData, QualiGridCacheBlob,
    QualiGridData, TrackWeatherCacheBlob, TrackWeatherData, WeekendDetailData,
    WeekendFetchInput,
};
use crate::assets::{
    apply_platform_branding, drain_menu_messages, fonts, window_icon, APP_DISPLAY_NAME,
};
use crate::db::{default_db_path, rebuild_database, AssetStore, Database};
use crate::debug;
use crate::domain::calendar_filter::next_session_reminder_at;
use crate::domain::{
    circuit_image_url, is_circuit_image_url, move_pin, pin_driver, prepare_circuit_image,
    standings_signature, unpin_all, unpin_driver,
};
use crate::platform::notifications::{notify_session_reminder, notify_standings_change};
use crate::platform::{install_tray, poll_tray_actions, TrayAction};
use crate::ui::theme::{self, ThemePresetId};
use crate::ui::restore_driver_picker_scroll;
use crate::state::{
    animate_chart_tooltip, apply_chart_hover, AppState, BootStepId, ChampionshipLoadState,
    DriversLoadState, LoadState, LoadedCalendar, LoadedChampionship, LoadedDrivers, LoadedWeekend,
    Message, Overlay, SettingsToggle, WeekendLoadState, WindowAction,
};
use crate::ui::shell;

const TICK_MS: u64 = 50;

pub fn run() -> iced::Result {
    let mut app = iced::application(App::title, App::update, App::view)
        .theme(|_| crate::ui::theme::iced_theme())
        .subscription(App::subscription)
        .default_font(fonts::UI);

    for font in fonts::FONT_BYTES {
        app = app.font(*font);
    }

    app.window(iced::window::Settings {
            size: Size::new(1100.0, 780.0),
            min_size: Some(Size::new(800.0, 600.0)),
            decorations: false,
            icon: Some(window_icon()),
            exit_on_close_request: false,
            ..Default::default()
        })
        .run_with(App::boot)
}

struct App {
    state: AppState,
    db: Database,
    assets: AssetStore,
    title_bar_last_press: Option<Instant>,
    title_bar_drag_pending: bool,
    _tray: Option<tray_icon::TrayIcon>,
}

impl App {
    fn title(_state: &App) -> String {
        APP_DISPLAY_NAME.into()
    }

    fn boot() -> (Self, Task<Message>) {
        apply_platform_branding();
        debug::info(format!("{APP_DISPLAY_NAME} starting"));

        let db = Database::open_default().expect("open sqlite database");
        let assets = AssetStore::open_default().expect("open asset cache");
        let settings = db.load_settings().unwrap_or_default();
        let pinned_drivers = db.list_pinned_drivers().unwrap_or_default();
        let season = settings.effective_season(Utc::now().year());

        debug::info(format!("Loaded settings for season {season}"));

        let mut state = AppState {
            settings,
            pinned_drivers,
            ..AppState::default()
        };
        theme::init_palette(state.settings.theme_id);
        state.show_first_run = !state.settings.first_run_complete;

        let tray = match install_tray() {
            Ok(icon) => Some(icon),
            Err(error) => {
                debug::warn(format!("System tray unavailable: {error}"));
                None
            }
        };

        let mut tasks = vec![window::get_oldest().then(|id| match id {
            Some(window_id) => Task::batch([
                window::get_size(window_id).map(Message::WindowResized),
                Task::done(Message::WindowReady(window_id)),
            ]),
            None => Task::none(),
        })];

        if let Ok(Some(mut data)) = db.calendar_from_cache(season) {
            data.apply_preferences(state.settings.include_testing);
            debug::info("Showing cached calendar on boot");
            let fresh = db
                .cache_entry_for_calendar(season)
                .ok()
                .flatten()
                .map(|entry| cache_is_fresh(&entry, Utc::now()))
                .unwrap_or(false);
            state.load = LoadState::Ready(LoadedCalendar {
                data,
                stale: !fresh,
            });
            if !fresh {
                state.refreshing = true;
                tasks.push(fetch_calendar_task(season));
            }
        } else {
            state.load = LoadState::Loading;
            state.refreshing = true;
            tasks.push(fetch_calendar_task(season));
        }

        if let Ok(Some(data)) = db.drivers_from_cache(season) {
            debug::info("Showing cached drivers on boot");
            let fresh = db
                .cache_entry_for_drivers(season)
                .ok()
                .flatten()
                .map(|entry| cache_is_fresh(&entry, Utc::now()))
                .unwrap_or(false);
            state.drivers = DriversLoadState::Ready(LoadedDrivers {
                data,
                stale: !fresh,
            });
            if !fresh {
                state.drivers_refreshing = true;
                tasks.push(fetch_drivers_task(season));
            }
        } else {
            state.drivers = DriversLoadState::Loading;
            state.drivers_refreshing = true;
            tasks.push(fetch_drivers_task(season));
        }

        if let Some(calendar) = state.calendar() {
            let sessions = calendar.sessions.clone();
            schedule_championship_refresh(
                &mut state,
                &db,
                season,
                sessions.clone(),
                false,
                &mut tasks,
            );
            schedule_weekend_refresh(&mut state, &db, false, &mut tasks);
        } else {
            state.championship = ChampionshipLoadState::Loading;
            state.weekend = WeekendLoadState::Loading;
        }

        state
            .boot
            .complete_step(BootStepId::Settings, format!("Season {season} ready"));
        sync_boot_calendar(&mut state);
        sync_boot_drivers(&mut state);
        sync_boot_championship(&mut state);
        extend_boot_media(&mut state, &assets, &db, &mut tasks);
        finalize_boot_media_step(&mut state);
        state.boot.try_finish();

        (Self {
            state,
            db,
            assets,
            title_bar_last_press: None,
            title_bar_drag_pending: false,
            _tray: tray,
        }, Task::batch(tasks))
    }

    fn maybe_notify_standings(&self, data: &ChampionshipData) {
        if !self.state.settings.notifications_enabled || !self.state.settings.notify_standings {
            return;
        }

        let Some(latest) = data.rounds.last() else {
            return;
        };

        let signature = standings_signature(&latest.drivers);
        let key = crate::db::schema::SETTING_LAST_STANDINGS_SIGNATURE;
        if self
            .db
            .load_setting(key)
            .ok()
            .flatten()
            .as_deref()
            == Some(signature.as_str())
        {
            return;
        }

        let _ = self.db.save_setting(key, &signature);
        let roster = self.state.drivers_roster().unwrap_or(&[]);
        for pin in &self.state.pinned_drivers {
            if let Some(entry) = latest
                .drivers
                .iter()
                .find(|row| row.driver_number == pin.driver_number)
            {
                if let Some(driver) = roster.iter().find(|d| d.driver_number == pin.driver_number)
                {
                    notify_standings_change(
                        &crate::domain::driver_display_name(driver),
                        entry.position,
                        entry.points,
                    );
                }
            }
        }
    }

    fn maybe_notify_upcoming_session(&self) {
        if !self.state.settings.notifications_enabled || !self.state.settings.notify_sessions {
            return;
        }

        let Some(calendar) = self.state.calendar() else {
            return;
        };

        let now = Utc::now();
        let lead = self.state.settings.session_reminder_minutes;
        let Some((remind_at, session_name)) = next_session_reminder_at(
            &calendar.sessions,
            now,
            lead,
        ) else {
            return;
        };

        if now < remind_at {
            return;
        }

        let key = crate::db::schema::SETTING_LAST_SESSION_REMINDER;
        let signature = format!("{session_name}:{lead}");
        if self
            .db
            .load_setting(key)
            .ok()
            .flatten()
            .as_deref()
            == Some(signature.as_str())
        {
            return;
        }

        let _ = self.db.save_setting(key, &signature);
        notify_session_reminder(&session_name, &format!("{lead} minutes"));
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => {
                debug::info("Manual refresh");
                self.state.refreshing = true;
                self.state.drivers_refreshing = true;
                self.state.weekend_refreshing = true;
                let season = self.state.settings.effective_season(Utc::now().year());
                let championship_sessions = self
                    .state
                    .calendar()
                    .map(|calendar| calendar.sessions.clone());
                let mut tasks = vec![
                    fetch_calendar_task(season),
                    fetch_drivers_task(season),
                ];
                if let Some(sessions) = championship_sessions {
                    let cached = self.db.championship_from_cache(season).ok().flatten();
                    if self.state.championship_data().is_none() {
                        self.state.championship = ChampionshipLoadState::Loading;
                    } else if let ChampionshipLoadState::Ready(loaded) = &self.state.championship {
                        self.state.championship = ChampionshipLoadState::Ready(LoadedChampionship {
                            data: loaded.data.clone(),
                            stale: true,
                        });
                    }
                    tasks.push(fetch_championship_task(season, sessions, cached));
                    self.state.championship_refreshing = true;
                }
                schedule_weekend_refresh(&mut self.state, &self.db, true, &mut tasks);
                Task::batch(tasks)
            }
            Message::Fetched(result) => {
                self.state.refreshing = false;
                match result {
                    Ok(mut data) => {
                        debug::info(format!(
                            "Calendar fetched for season {} ({} meetings)",
                            data.season,
                            data.meetings.len()
                        ));
                        self.state.settings.season_year = data.season;
                        let _ = self.db.save_settings(&self.state.settings);
                        let blob = CalendarCacheBlob::from_calendar(&data);
                        let _ = self.db.save_calendar_cache(&blob);
                        data.apply_preferences(self.state.settings.include_testing);
                        self.state.load = LoadState::Ready(LoadedCalendar {
                            data,
                            stale: false,
                        });
                        sync_boot_calendar_done(&mut self.state, "Fetched from OpenF1");
                        let season = self.state.settings.effective_season(Utc::now().year());
                        let sessions = self
                            .state
                            .calendar()
                            .map(|calendar| calendar.sessions.clone())
                            .unwrap_or_default();
                        let mut refresh_tasks =
                            fetch_flag_tasks(&mut self.state, &self.assets, &self.db);
                        schedule_championship_refresh(
                            &mut self.state,
                            &self.db,
                            season,
                            sessions,
                            false,
                            &mut refresh_tasks,
                        );
                        schedule_weekend_refresh(&mut self.state, &self.db, false, &mut refresh_tasks);
                        sync_boot_championship(&mut self.state);
                        extend_boot_media(&mut self.state, &self.assets, &self.db, &mut refresh_tasks);
                        finalize_boot_media_step(&mut self.state);
                        self.state.boot.try_finish();
                        return Task::batch(refresh_tasks);
                    }
                    Err(error) => {
                        debug::error(format!("Calendar fetch failed: {error}"));
                        let season = self.state.settings.effective_season(Utc::now().year());
                        if let LoadState::Ready(loaded) = &self.state.load {
                            self.state.load = LoadState::Ready(LoadedCalendar {
                                data: loaded.data.clone(),
                                stale: true,
                            });
                        } else if let Ok(Some(mut data)) = self.db.calendar_from_cache(season) {
                            data.apply_preferences(self.state.settings.include_testing);
                            self.state.load = LoadState::Ready(LoadedCalendar { data, stale: true });
                        } else {
                            self.state.load = LoadState::Error {
                                message: error,
                                cached: None,
                            };
                            sync_boot_calendar_failed(&mut self.state, "Could not load calendar");
                        }
                    }
                }
                self.state.boot.try_finish();
                Task::none()
            }
            Message::DriversFetched(result) => {
                self.state.drivers_refreshing = false;
                match result {
                    Ok(data) => {
                        debug::info(format!(
                            "Drivers fetched for season {} ({} drivers)",
                            data.season,
                            data.drivers.len()
                        ));
                        let blob = DriversCacheBlob::from_drivers(&data);
                        let _ = self.db.save_drivers_cache(&blob);
                        self.state.drivers = DriversLoadState::Ready(LoadedDrivers {
                            data,
                            stale: false,
                        });
                        sync_boot_drivers_done(&mut self.state, "Fetched from OpenF1");
                        let mut tasks =
                            fetch_driver_media_tasks(&self.state, &self.assets, &self.db);
                        extend_boot_media(&mut self.state, &self.assets, &self.db, &mut tasks);
                        finalize_boot_media_step(&mut self.state);
                        self.state.boot.try_finish();
                        return Task::batch(tasks);
                    }
                    Err(error) => {
                        debug::error(format!("Drivers fetch failed: {error}"));
                        let season = self.state.settings.effective_season(Utc::now().year());
                        if let DriversLoadState::Ready(loaded) = &self.state.drivers {
                            self.state.drivers = DriversLoadState::Ready(LoadedDrivers {
                                data: loaded.data.clone(),
                                stale: true,
                            });
                        } else if let Ok(Some(data)) = self.db.drivers_from_cache(season) {
                            self.state.drivers = DriversLoadState::Ready(LoadedDrivers {
                                data,
                                stale: true,
                            });
                        } else {
                            self.state.drivers = DriversLoadState::Error {
                                message: error,
                                cached: None,
                            };
                            sync_boot_drivers_failed(&mut self.state, "Could not load drivers");
                        }
                    }
                }
                self.state.boot.try_finish();
                Task::none()
            }
            Message::WeekendFetched(result) => {
                self.state.weekend_refreshing = false;
                match result {
                    Ok(data) => {
                        persist_weekend_caches(&self.db, &data);
                        self.state.weekend = WeekendLoadState::Ready(LoadedWeekend {
                            data,
                            stale: false,
                        });
                    }
                    Err(error) => {
                        debug::error(format!("Weekend detail fetch failed: {error}"));
                        if let WeekendLoadState::Ready(loaded) = &self.state.weekend {
                            self.state.weekend = WeekendLoadState::Ready(LoadedWeekend {
                                data: loaded.data.clone(),
                                stale: true,
                            });
                        } else if let Some(data) = load_weekend_from_db(&self.db, &self.state) {
                            self.state.weekend = WeekendLoadState::Ready(LoadedWeekend {
                                data,
                                stale: true,
                            });
                        } else {
                            self.state.weekend = WeekendLoadState::Error {
                                message: error,
                                cached: None,
                            };
                        }
                    }
                }
                Task::none()
            }
            Message::ChampionshipFetched(result) => {
                self.state.championship_refreshing = false;
                match result {
                    Ok(data) => {
                        debug::info(format!(
                            "Championship fetched for season {} ({} rounds)",
                            data.season,
                            data.rounds.len()
                        ));
                        let blob = ChampionshipCacheBlob::from_data(&data);
                        let _ = self.db.save_championship_cache(&blob);
                        self.maybe_notify_standings(&data);
                        self.state.championship = ChampionshipLoadState::Ready(LoadedChampionship {
                            data,
                            stale: false,
                        });
                        sync_boot_championship_done(&mut self.state, "Fetched from OpenF1");
                    }
                    Err(error) => {
                        debug::error(format!("Championship fetch failed: {error}"));
                        let season = self.state.settings.effective_season(Utc::now().year());
                        if let ChampionshipLoadState::Ready(loaded) = &self.state.championship {
                            self.state.championship = ChampionshipLoadState::Ready(LoadedChampionship {
                                data: loaded.data.clone(),
                                stale: true,
                            });
                            sync_boot_championship_done(&mut self.state, "Using cached standings");
                        } else if let Ok(Some(data)) = self.db.championship_from_cache(season) {
                            self.state.championship = ChampionshipLoadState::Ready(LoadedChampionship {
                                data,
                                stale: true,
                            });
                            sync_boot_championship_done(&mut self.state, "Loaded from cache");
                        } else {
                            self.state.championship = ChampionshipLoadState::Error {
                                message: error.clone(),
                                cached: None,
                            };
                            sync_boot_championship_failed(&mut self.state, "Could not load standings");
                        }
                    }
                }
                finalize_boot_media_step(&mut self.state);
                self.state.boot.try_finish();
                Task::none()
            }
            Message::ChampionshipTabSelected(tab) => {
                self.state.championship_tab = tab;
                self.state.chart_tooltip = None;
                Task::none()
            }
            Message::ChampionshipChartModeSelected(mode) => {
                self.state.championship_chart_mode = mode;
                self.state.chart_tooltip = None;
                Task::none()
            }
            Message::ChampionshipChartHover(hit) => {
                apply_chart_hover(&mut self.state, hit);
                Task::none()
            }
            Message::FlagLoaded { url, result } => {
                match result {
                    Ok(bytes) => {
                        let bytes = if is_circuit_image_url(&url) {
                            prepare_circuit_image(&bytes).unwrap_or(bytes)
                        } else {
                            bytes
                        };
                        self.state.flag_images.insert(
                            url,
                            image::Handle::from_bytes(bytes::Bytes::from(bytes)),
                        );
                    }
                    Err(_) => {
                        let _ = self.assets.mark_failed(&url);
                        self.state.asset_fetch_failed.insert(url);
                    }
                }
                if self.state.boot.active {
                    self.state.boot.media_loaded();
                    self.state.boot.try_finish();
                }
                Task::none()
            }
            Message::HeadshotLoaded { url, result } => {
                match result {
                    Ok(bytes) => {
                        self.state.headshot_failed.remove(&url);
                        self.state.headshot_images.insert(
                            url,
                            image::Handle::from_bytes(bytes::Bytes::from(bytes)),
                        );
                    }
                    Err(_) => {
                        let _ = self.assets.mark_failed(&url);
                        self.state.headshot_failed.insert(url);
                    }
                }
                if self.state.boot.active {
                    self.state.boot.media_loaded();
                    self.state.boot.try_finish();
                }
                Task::none()
            }
            Message::Tick => {
                apply_platform_branding();
                self.state.animation_phase = self.state.animation_phase.wrapping_add(1);
                if let Some(tooltip) = &mut self.state.chart_tooltip {
                    animate_chart_tooltip(tooltip);
                }
                self.state.scrollbar_visible.on_tick();
                if let LoadState::Ready(loaded) = &mut self.state.load {
                    loaded.data.countdown = crate::domain::next_season_countdown(
                        &loaded.data.meetings,
                        &loaded.data.sessions,
                        Utc::now(),
                    );
                }

                self.maybe_notify_upcoming_session();

                let mut follow_up = Vec::new();
                for action in poll_tray_actions() {
                    follow_up.push(match action {
                        TrayAction::Show => Message::ShowFromTray,
                        TrayAction::Quit => Message::QuitFromTray,
                    });
                }
                follow_up.extend(drain_menu_messages());

                if follow_up.is_empty() {
                    Task::none()
                } else {
                    Task::batch(follow_up.into_iter().map(|message| self.update(message)))
                }
            }
            Message::ShowFromTray => {
                if let Some(id) = self.state.window_id {
                    let mode = self
                        .state
                        .hidden_window_mode
                        .take()
                        .unwrap_or(Mode::Windowed);
                    return Task::batch([
                        window::change_mode(id, mode),
                        window::gain_focus(id),
                    ]);
                }
                Task::none()
            }
            Message::QuitFromTray => {
                iced::exit()
            }
            Message::TitleBarPressed => {
                let now = Instant::now();
                let is_double = self
                    .title_bar_last_press
                    .is_some_and(|last| now.duration_since(last) <= Duration::from_millis(400));
                self.title_bar_last_press = Some(now);

                if is_double {
                    self.title_bar_drag_pending = false;
                    return window_action_task(self.state.window_id, WindowAction::Maximize);
                }

                self.title_bar_drag_pending = true;
                Task::perform(
                    async {
                        tokio::time::sleep(Duration::from_millis(220)).await;
                    },
                    |_| Message::TitleBarDrag,
                )
            }
            Message::TitleBarDrag => {
                if self.title_bar_drag_pending {
                    self.title_bar_drag_pending = false;
                    return window_action_task(self.state.window_id, WindowAction::Drag);
                }
                Task::none()
            }
            Message::TitleBarControlsHover(hover) => {
                self.state.title_bar_controls_hover = hover;
                Task::none()
            }
            Message::ThemeSelected(theme_id) => {
                self.state.settings.theme_id = theme_id;
                theme::init_palette(theme_id);
                let _ = self.db.save_settings(&self.state.settings);
                Task::none()
            }
            Message::ActivateRivalCompare => {
                if self.state.rival_ready() {
                    self.state.settings.rival_compare_active = true;
                    let _ = self.db.save_settings(&self.state.settings);
                }
                Task::none()
            }
            Message::ExitRivalCompare => {
                self.state.settings.rival_compare_active = false;
                let _ = self.db.save_settings(&self.state.settings);
                Task::none()
            }
            Message::OpenRivalPicker(slot) => {
                self.state.rival_pick_slot = Some(slot);
                self.state.overlay = Overlay::DriverPicker;
                restore_driver_picker_scroll(self.state.driver_picker_scroll)
            }
            Message::RivalDriverSelected { slot, driver_number } => {
                match slot {
                    0 => self.state.settings.rival_driver_first = driver_number,
                    _ => self.state.settings.rival_driver_second = driver_number,
                }
                if self.state.settings.rival_driver_first == self.state.settings.rival_driver_second
                    && self.state.settings.rival_driver_first > 0
                {
                    self.state.settings.rival_compare_active = false;
                } else if !self.state.rival_ready() {
                    self.state.settings.rival_compare_active = false;
                }
                let _ = self.db.save_settings(&self.state.settings);
                self.state.rival_pick_slot = None;
                self.state.overlay = Overlay::None;
                Task::none()
            }
            Message::CompleteFirstRun => {
                self.state.settings.first_run_complete = true;
                self.state.show_first_run = false;
                let _ = self.db.save_settings(&self.state.settings);
                Task::none()
            }
            Message::WindowResized(size) => {
                self.state.viewport = size;
                Task::none()
            }
            Message::WindowReady(id) => {
                apply_platform_branding();
                self.state.window_id = Some(id);
                Task::none()
            }
            Message::WindowCloseRequested(id) => {
                self.state.window_id = Some(id);
                hide_or_close_window(&self.state)
            }
            Message::HideToBackground(current_mode) => {
                let restore = match current_mode {
                    Mode::Hidden => self
                        .state
                        .hidden_window_mode
                        .unwrap_or(Mode::Windowed),
                    mode => mode,
                };
                self.state.hidden_window_mode = Some(restore);
                if let Some(id) = self.state.window_id {
                    window::change_mode(id, Mode::Hidden)
                } else {
                    Task::none()
                }
            }
            Message::WindowAction(action) => {
                if matches!(action, WindowAction::Close) {
                    return hide_or_close_window(&self.state);
                }
                window_action_task(self.state.window_id, action)
            }
            Message::Navigate(screen) => {
                self.state.screen = screen;
                self.state.settings_notice = None;
                Task::none()
            }
            Message::ScrollInteraction => {
                self.state.scrollbar_visible.on_scroll(TICK_MS);
                Task::none()
            }
            Message::DriverPickerScroll(offset) => {
                self.state.driver_picker_scroll = offset;
                self.state.scrollbar_visible.on_scroll(TICK_MS);
                Task::none()
            }
            Message::SettingsToggled(toggle) => {
                match toggle {
                    SettingsToggle::IncludeTesting => {
                        self.state.settings.include_testing =
                            !self.state.settings.include_testing;
                        if let Some(season) = self.state.calendar().map(|calendar| calendar.season) {
                            if let Ok(Some(mut data)) = self.db.calendar_from_cache(season) {
                                data.apply_preferences(self.state.settings.include_testing);
                                match &mut self.state.load {
                                    LoadState::Ready(loaded) => loaded.data = data,
                                    LoadState::Error {
                                        cached: Some(loaded),
                                        ..
                                    } => loaded.data = data,
                                    _ => {}
                                }
                            }
                        }
                    }
                    SettingsToggle::BackgroundOnClose => {
                        self.state.settings.background_on_close =
                            !self.state.settings.background_on_close;
                    }
                    SettingsToggle::NotificationsEnabled => {
                        self.state.settings.notifications_enabled =
                            !self.state.settings.notifications_enabled;
                    }
                    SettingsToggle::NotifyStandings => {
                        self.state.settings.notify_standings =
                            !self.state.settings.notify_standings;
                    }
                    SettingsToggle::NotifySessions => {
                        self.state.settings.notify_sessions =
                            !self.state.settings.notify_sessions;
                    }
                }
                let _ = self.db.save_settings(&self.state.settings);
                Task::none()
            }
            Message::CopyDebugLog => {
                let text = debug::entries_text();
                if text.is_empty() {
                    self.state.settings_notice = Some("Debug log is empty.".into());
                    return Task::none();
                }
                clipboard::write(text)
            }
            Message::OpenAbout => {
                self.state.overlay = Overlay::About;
                Task::none()
            }
            Message::OpenDriverPicker => {
                self.state.rival_pick_slot = None;
                self.state.driver_picker = crate::domain::DriverPickerFilters::default();
                self.state.driver_picker_scroll = iced::widget::scrollable::RelativeOffset::START;
                self.state.overlay = Overlay::DriverPicker;
                let mut tasks = vec![restore_driver_picker_scroll(
                    iced::widget::scrollable::RelativeOffset::START,
                )];
                tasks.extend(fetch_driver_media_tasks(
                    &self.state,
                    &self.assets,
                    &self.db,
                ));
                Task::batch(tasks)
            }
            Message::CloseOverlay => {
                self.state.overlay = Overlay::None;
                self.state.rival_pick_slot = None;
                self.state.driver_picker = crate::domain::DriverPickerFilters::default();
                Task::none()
            }
            Message::OverlayClick => Task::none(),
            Message::DriverPickerSearch(query) => {
                self.state.driver_picker.search = query;
                Task::none()
            }
            Message::DriverPickerSortField(field) => {
                if self.state.driver_picker.sort_field == field {
                    self.state.driver_picker.sort_direction =
                        self.state.driver_picker.sort_direction.toggle();
                } else {
                    self.state.driver_picker.sort_field = field;
                    self.state.driver_picker.sort_direction =
                        crate::domain::SortDirection::Asc;
                }
                Task::none()
            }
            Message::DriverPickerToggleGroup => {
                self.state.driver_picker.group_by_constructor =
                    !self.state.driver_picker.group_by_constructor;
                Task::none()
            }
            Message::PinDriver(driver_number) => {
                if pin_driver(&mut self.state.pinned_drivers, driver_number) {
                    let _ = self.db.replace_pinned_drivers(&self.state.pinned_drivers);
                    debug::info(format!("Pinned driver #{driver_number}"));
                    let mut tasks = fetch_driver_media_tasks(&self.state, &self.assets, &self.db);
                    schedule_weekend_refresh(&mut self.state, &self.db, true, &mut tasks);
                    return Task::batch(tasks);
                }
                Task::none()
            }
            Message::UnpinDriver(driver_number) => {
                if unpin_driver(&mut self.state.pinned_drivers, driver_number) {
                    let _ = self.db.replace_pinned_drivers(&self.state.pinned_drivers);
                    debug::info(format!("Unpinned driver #{driver_number}"));
                    let mut tasks = Vec::new();
                    schedule_weekend_refresh(&mut self.state, &self.db, true, &mut tasks);
                    return Task::batch(tasks);
                }
                Task::none()
            }
            Message::UnpinAll => {
                if unpin_all(&mut self.state.pinned_drivers) {
                    let _ = self.db.replace_pinned_drivers(&self.state.pinned_drivers);
                    debug::info("Unpinned all drivers");
                    let mut tasks = Vec::new();
                    schedule_weekend_refresh(&mut self.state, &self.db, true, &mut tasks);
                    return Task::batch(tasks);
                }
                Task::none()
            }
            Message::MovePin {
                driver_number,
                direction,
            } => {
                if move_pin(&mut self.state.pinned_drivers, driver_number, direction) {
                    let _ = self.db.replace_pinned_drivers(&self.state.pinned_drivers);
                }
                Task::none()
            }
            Message::ClearCache => {
                match self.db.clear_cache() {
                    Ok((api_count, asset_count)) => {
                        debug::info(format!(
                            "Cleared {api_count} cache entries and {asset_count} assets"
                        ));
                        self.state.settings_notice = Some(format!(
                            "Cleared {api_count} API responses and {asset_count} cached assets."
                        ));
                        self.state.flag_images.clear();
                        self.state.headshot_images.clear();
                        self.state.headshot_failed.clear();
                        self.state.load = LoadState::Loading;
                        self.state.drivers = DriversLoadState::Loading;
                        self.state.championship = ChampionshipLoadState::Loading;
                        self.state.weekend = WeekendLoadState::Loading;
                        let season = self.state.settings.effective_season(Utc::now().year());
                        self.state.boot.reset();
                        self.state
                            .boot
                            .complete_step(BootStepId::Settings, format!("Season {season} ready"));
                        self.state.boot.start_step(BootStepId::Calendar, "Fetching from OpenF1...");
                        self.state.boot.start_step(BootStepId::Drivers, "Fetching from OpenF1...");
                        self.state.boot.start_step(BootStepId::Championship, "Fetching from OpenF1...");
                        self.state.refreshing = true;
                        self.state.drivers_refreshing = true;
                        return Task::batch([
                            fetch_calendar_task(season),
                            fetch_drivers_task(season),
                        ]);
                    }
                    Err(error) => {
                        debug::error(format!("Clear cache failed: {error}"));
                        self.state.settings_notice = Some(format!("Clear cache failed: {error}"));
                    }
                }
                Task::none()
            }
            Message::RebuildDatabase => {
                match default_db_path().and_then(|path| rebuild_database(&path)) {
                    Ok(db) => {
                        debug::info("Database rebuilt successfully");
                        self.db = db;
                        self.assets = AssetStore::open_default().expect("open asset cache");
                        self.state.settings = self.db.load_settings().unwrap_or_default();
                        self.state.pinned_drivers =
                            self.db.list_pinned_drivers().unwrap_or_default();
                        self.state.flag_images.clear();
                        self.state.headshot_images.clear();
                        self.state.headshot_failed.clear();
                        self.state.settings_notice =
                            Some("Database rebuilt. Settings and pins preserved.".into());
                        self.state.load = LoadState::Loading;
                        self.state.drivers = DriversLoadState::Loading;
                        self.state.championship = ChampionshipLoadState::Loading;
                        self.state.weekend = WeekendLoadState::Loading;
                        self.state.refreshing = true;
                        self.state.drivers_refreshing = true;
                        self.state.weekend_refreshing = true;
                        self.state.boot.reset();
                        self.state.boot.complete_step(
                            BootStepId::Settings,
                            "Settings and pins preserved",
                        );
                        self.state.boot.start_step(BootStepId::Calendar, "Fetching from OpenF1...");
                        self.state.boot.start_step(BootStepId::Drivers, "Fetching from OpenF1...");
                        self.state.boot.start_step(BootStepId::Championship, "Fetching from OpenF1...");
                        let season = self.state.settings.effective_season(Utc::now().year());
                        return Task::batch([
                            fetch_calendar_task(season),
                            fetch_drivers_task(season),
                        ]);
                    }
                    Err(error) => {
                        debug::error(format!("Database rebuild failed: {error}"));
                        self.state.settings_notice =
                            Some(format!("Database rebuild failed: {error}"));
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        shell(&self.state)
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(Duration::from_millis(TICK_MS)).map(|_| Message::Tick),
            window::resize_events().map(|(_id, size)| Message::WindowResized(size)),
            window::close_requests().map(Message::WindowCloseRequested),
        ])
    }
}

fn hide_or_close_window(state: &AppState) -> Task<Message> {
    if state.settings.background_on_close {
        let Some(id) = state.window_id else {
            return Task::none();
        };
        window::get_mode(id).then(|mode| Task::done(Message::HideToBackground(mode)))
    } else {
        window_action_task(state.window_id, WindowAction::Close)
    }
}

fn window_action_task(window_id: Option<window::Id>, action: WindowAction) -> Task<Message> {
    let Some(id) = window_id else {
        return window::get_oldest().then(move |id| match id {
            Some(id) => window_action_task(Some(id), action),
            None => Task::none(),
        });
    };

    match action {
        WindowAction::Close => window::close(id),
        WindowAction::Minimize => window::minimize(id, true),
        WindowAction::Maximize => window::toggle_maximize(id),
        WindowAction::Fullscreen => window::get_mode(id).then(move |mode| {
            let next = if mode == Mode::Fullscreen {
                Mode::Windowed
            } else {
                Mode::Fullscreen
            };
            window::change_mode(id, next)
        }),
        WindowAction::Drag => window::drag(id),
    }
}

fn fetch_calendar_task(season: i32) -> Task<Message> {
    Task::perform(fetch_calendar(season), Message::Fetched)
}

fn fetch_drivers_task(season: i32) -> Task<Message> {
    Task::perform(fetch_drivers(season), Message::DriversFetched)
}

fn fetch_flag_tasks(
    state: &mut AppState,
    assets: &AssetStore,
    db: &Database,
) -> Vec<Task<Message>> {
    let Some(data) = state.calendar() else {
        return Vec::new();
    };

    let assets_dir = assets.assets_dir();
    let mut urls = HashSet::new();
    if let Some(previous) = &data.triplet.previous {
        urls.insert(previous.country_flag.clone());
        if let Some(url) = circuit_image_url(previous) {
            urls.insert(url.to_string());
        }
    }
    urls.insert(data.triplet.current.country_flag.clone());
    if let Some(url) = circuit_image_url(&data.triplet.current) {
        urls.insert(url.to_string());
    }
    urls.insert(data.triplet.upcoming.country_flag.clone());
    if let Some(url) = circuit_image_url(&data.triplet.upcoming) {
        urls.insert(url.to_string());
    }

    let urls: Vec<String> = urls.into_iter().collect();
    for url in &urls {
        if db.is_asset_failed(url, assets_dir).unwrap_or(false) {
            state.asset_fetch_failed.insert(url.clone());
        }
    }

    urls.into_iter()
        .filter(|url| {
            !url.is_empty()
                && !state.flag_images.contains_key(url)
                && !state.asset_fetch_failed(url)
        })
        .map(|url| {
            Task::perform(
                fetch_image_cached(assets.clone(), url.clone()),
                move |result| Message::FlagLoaded { url: url.clone(), result },
            )
        })
        .collect()
}

fn fetch_driver_media_tasks(
    state: &AppState,
    assets: &AssetStore,
    db: &Database,
) -> Vec<Task<Message>> {
    let assets_dir = assets.assets_dir();
    let mut headshot_urls = HashSet::new();
    let mut icon_urls = HashSet::new();

    if let Some(roster) = state.drivers_roster() {
        for driver in roster {
            if !driver.headshot_url.is_empty() {
                headshot_urls.insert(driver.headshot_url.clone());
            }
            if let Some(url) = crate::domain::driver_flag_url(
                &driver.country_code,
                driver.driver_number,
                &driver.name_acronym,
            ) {
                icon_urls.insert(url);
            }
            if let Some(url) = crate::domain::team_logo_url(&driver.team_name) {
                icon_urls.insert(url);
            }
        }
    }

    let mut tasks: Vec<Task<Message>> = headshot_urls
        .into_iter()
        .filter(|url| {
            !state.headshot_images.contains_key(url)
                && !state.headshot_failed.contains(url)
                && !db.is_asset_failed(url, assets_dir).unwrap_or(false)
        })
        .map(|url| {
            Task::perform(
                fetch_image_cached(assets.clone(), url.clone()),
                move |result| Message::HeadshotLoaded { url: url.clone(), result },
            )
        })
        .collect();

    tasks.extend(
        icon_urls
            .into_iter()
            .filter(|url| {
                !url.is_empty()
                    && !state.flag_images.contains_key(url)
                    && !db.is_asset_failed(url, assets_dir).unwrap_or(false)
            })
            .map(|url| {
                Task::perform(
                    fetch_image_cached(assets.clone(), url.clone()),
                    move |result| Message::FlagLoaded { url: url.clone(), result },
                )
            }),
    );

    tasks
}

async fn fetch_image_cached(assets: AssetStore, url: String) -> Result<Vec<u8>, String> {
    if let Ok(Some(bytes)) = tokio::task::spawn_blocking({
        let assets = assets.clone();
        let url = url.clone();
        move || assets.load_cached(&url)
    })
    .await
    .map_err(|error| error.to_string())?
    {
        return Ok(bytes);
    }

    if tokio::task::spawn_blocking({
        let assets = assets.clone();
        let url = url.clone();
        move || assets.is_failed(&url)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?
    {
        return Err("asset fetch blocked".into());
    }

    let bytes = fetch_image_network(&url).await?;
    let cached = bytes.clone();

    tokio::task::spawn_blocking(move || assets.save_cached(&url, &cached))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;

    Ok(bytes)
}

async fn fetch_image_network(url: &str) -> Result<Vec<u8>, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|error| error.to_string())
}

async fn fetch_calendar(season: i32) -> Result<CalendarData, String> {
    fetch_season_calendar(season)
        .await
        .map_err(|error| error.to_string())
}

async fn fetch_drivers(season: i32) -> Result<DriversData, String> {
    fetch_season_drivers(season)
        .await
        .map_err(|error| error.to_string())
}

fn fetch_championship_task(
    season: i32,
    sessions: Vec<Session>,
    existing: Option<ChampionshipData>,
) -> Task<Message> {
    Task::perform(
        fetch_championship(season, sessions, existing),
        Message::ChampionshipFetched,
    )
}

async fn fetch_championship(
    season: i32,
    sessions: Vec<Session>,
    existing: Option<ChampionshipData>,
) -> Result<ChampionshipData, String> {
    fetch_season_championship(season, &sessions, existing)
        .await
        .map_err(|error| error.to_string())
}

fn schedule_championship_refresh(
    state: &mut AppState,
    db: &Database,
    season: i32,
    sessions: Vec<Session>,
    force: bool,
    tasks: &mut Vec<Task<Message>>,
) {
    let now = Utc::now();
    let cached = db.championship_from_cache(season).ok().flatten();
    let fresh = db
        .cache_entry_for_championship(season)
        .ok()
        .flatten()
        .map(|entry| cache_is_fresh(&entry, now))
        .unwrap_or(false);
    let needs_refresh = force
        || !fresh
        || championship_needs_refresh(cached.as_ref(), &sessions, now);

    if let Some(data) = cached.clone() {
        if state.championship_data().is_none() {
            state.championship = ChampionshipLoadState::Ready(LoadedChampionship {
                data,
                stale: !fresh,
            });
        } else if let ChampionshipLoadState::Ready(loaded) = &mut state.championship {
            loaded.stale = !fresh;
        }
    } else if state.championship_data().is_none() {
        state.championship = ChampionshipLoadState::Loading;
    }

    if !needs_refresh {
        debug::info("Skipping championship refresh; cache is fresh");
        return;
    }

    let existing = state.championship_data().cloned().or(cached);
    if state.championship_data().is_some() {
        if let ChampionshipLoadState::Ready(loaded) = &mut state.championship {
            loaded.stale = true;
        }
    }

    tasks.push(fetch_championship_task(season, sessions, existing));
    state.championship_refreshing = true;
    if state.boot.active {
        state
            .boot
            .start_step(BootStepId::Championship, "Fetching from OpenF1...");
    }
}

fn sync_boot_calendar(state: &mut AppState) {
    if !state.boot.active {
        return;
    }

    match &state.load {
        LoadState::Loading | LoadState::Error { cached: None, .. } if state.refreshing => {
            state
                .boot
                .start_step(BootStepId::Calendar, "Fetching from OpenF1...");
        }
        LoadState::Ready(loaded) if state.refreshing => {
            state.boot.start_step(
                BootStepId::Calendar,
                if loaded.stale {
                    "Refreshing cached calendar..."
                } else {
                    "Fetching from OpenF1..."
                },
            );
        }
        LoadState::Ready(_) => {
            state.boot.complete_step(BootStepId::Calendar, "Loaded from cache");
        }
        LoadState::Error { cached: Some(_), .. } => {
            state
                .boot
                .fail_step(BootStepId::Calendar, "Using cached calendar");
        }
        LoadState::Loading => {
            state
                .boot
                .start_step(BootStepId::Calendar, "Fetching from OpenF1...");
        }
        LoadState::Error { .. } => {
            state
                .boot
                .fail_step(BootStepId::Calendar, "Could not load calendar");
        }
    }
}

fn sync_boot_calendar_done(state: &mut AppState, detail: &str) {
    if state.boot.active {
        state.boot.complete_step(BootStepId::Calendar, detail);
    }
}

fn sync_boot_calendar_failed(state: &mut AppState, detail: &str) {
    if state.boot.active {
        state.boot.fail_step(BootStepId::Calendar, detail);
    }
}

fn sync_boot_drivers(state: &mut AppState) {
    if !state.boot.active {
        return;
    }

    match &state.drivers {
        DriversLoadState::Loading | DriversLoadState::Error { cached: None, .. }
            if state.drivers_refreshing =>
        {
            state
                .boot
                .start_step(BootStepId::Drivers, "Fetching from OpenF1...");
        }
        DriversLoadState::Ready(loaded) if state.drivers_refreshing => {
            state.boot.start_step(
                BootStepId::Drivers,
                if loaded.stale {
                    "Refreshing cached roster..."
                } else {
                    "Fetching from OpenF1..."
                },
            );
        }
        DriversLoadState::Ready(_) => {
            state.boot.complete_step(BootStepId::Drivers, "Loaded from cache");
        }
        DriversLoadState::Error { cached: Some(_), .. } => {
            state.boot.fail_step(BootStepId::Drivers, "Using cached roster");
        }
        DriversLoadState::Loading => {
            state
                .boot
                .start_step(BootStepId::Drivers, "Fetching from OpenF1...");
        }
        DriversLoadState::Error { .. } => {
            state
                .boot
                .fail_step(BootStepId::Drivers, "Could not load drivers");
        }
    }
}

fn sync_boot_drivers_done(state: &mut AppState, detail: &str) {
    if state.boot.active {
        state.boot.complete_step(BootStepId::Drivers, detail);
    }
}

fn sync_boot_drivers_failed(state: &mut AppState, detail: &str) {
    if state.boot.active {
        state.boot.fail_step(BootStepId::Drivers, detail);
    }
}

fn sync_boot_championship(state: &mut AppState) {
    if !state.boot.active {
        return;
    }

    match &state.championship {
        ChampionshipLoadState::Ready(loaded) if state.championship_refreshing => {
            state
                .boot
                .start_step(BootStepId::Championship, "Fetching from OpenF1...");
        }
        ChampionshipLoadState::Ready(loaded) => {
            state.boot.complete_step(
                BootStepId::Championship,
                if loaded.stale {
                    "Loaded from cache"
                } else {
                    "Up to date"
                },
            );
        }
        ChampionshipLoadState::Loading if state.championship_refreshing => {
            state
                .boot
                .start_step(BootStepId::Championship, "Fetching from OpenF1...");
        }
        ChampionshipLoadState::Loading => {
            if state.calendar().is_some() {
                state
                    .boot
                    .start_step(BootStepId::Championship, "Waiting to start...");
            } else {
                state
                    .boot
                    .start_step(BootStepId::Championship, "Waiting for calendar...");
            }
        }
        ChampionshipLoadState::Error { cached: Some(_), .. } => {
            state
                .boot
                .fail_step(BootStepId::Championship, "Using cached standings");
        }
        ChampionshipLoadState::Error { .. } => {
            state
                .boot
                .fail_step(BootStepId::Championship, "Could not load standings");
        }
    }
}

fn sync_boot_championship_done(state: &mut AppState, detail: &str) {
    if state.boot.active {
        state.boot.complete_step(BootStepId::Championship, detail);
    }
}

fn sync_boot_championship_failed(state: &mut AppState, detail: &str) {
    if state.boot.active {
        state.boot.fail_step(BootStepId::Championship, detail);
    }
}

fn register_boot_media_batch(state: &mut AppState, count: u32) {
    if !state.boot.active || count == 0 {
        return;
    }

    if state.boot.media_total == 0 {
        state.boot.begin_media(count);
    } else {
        state.boot.media_total += count;
        state.boot.start_step(
            BootStepId::Media,
            format!(
                "Downloading assets ({}/{})",
                state.boot.media_done, state.boot.media_total
            ),
        );
    }
}

fn extend_boot_media(
    state: &mut AppState,
    assets: &AssetStore,
    db: &Database,
    tasks: &mut Vec<Task<Message>>,
) {
    if !state.boot.active {
        return;
    }

    let flags = fetch_flag_tasks(state, assets, db);
    let media = fetch_driver_media_tasks(state, assets, db);
    let batch: Vec<Task<Message>> = flags.into_iter().chain(media).collect();
    register_boot_media_batch(state, batch.len() as u32);
    tasks.extend(batch);
}

fn finalize_boot_media_step(state: &mut AppState) {
    if !state.boot.active {
        return;
    }

    let media = state.boot.step_mut(BootStepId::Media);
    if media.status == crate::state::BootStepStatus::Pending {
        state.boot.complete_step(BootStepId::Media, "Nothing to download");
    }
}

fn schedule_weekend_refresh(
    state: &mut AppState,
    db: &Database,
    force: bool,
    tasks: &mut Vec<Task<Message>>,
) {
    let calendar_snapshot = state.calendar().cloned();
    let Some(calendar) = calendar_snapshot else {
        state.weekend = WeekendLoadState::Loading;
        return;
    };

    let now = Utc::now();
    let meetings = meetings_for_weather(&calendar.triplet);
    let focus_meeting_key = calendar.triplet.current.meeting_key;
    let pinned_numbers: Vec<i64> = state
        .pinned_drivers
        .iter()
        .map(|pin| pin.driver_number)
        .collect();

    let cached_grid = db
        .load_quali_grid_cache(focus_meeting_key)
        .ok()
        .flatten();
    let mut cached_track = HashMap::new();
    let mut cached_forecasts = HashMap::new();
    let mut weather_stale = false;

    for meeting in &meetings {
        if let Ok(Some(track)) = db.load_track_weather_cache(meeting.meeting_key) {
            cached_track.insert(meeting.meeting_key, track);
        }
        if let Ok(Some(forecast)) = db.load_forecast_cache(meeting.meeting_key) {
            cached_forecasts.insert(meeting.meeting_key, forecast);
        }

        let forecast_fresh = db
            .cache_entry_for_forecast(meeting.meeting_key)
            .ok()
            .flatten()
            .map(|entry| cache_is_fresh(&entry, now))
            .unwrap_or(false);
        let track_fresh = db
            .cache_entry_for_track_weather(meeting.meeting_key)
            .ok()
            .flatten()
            .map(|entry| cache_is_fresh(&entry, now))
            .unwrap_or(false);

        if !forecast_fresh || !track_fresh {
            weather_stale = true;
        }
    }

    let grid_stale = quali_grid_needs_refresh(
        &calendar.triplet.current,
        &calendar.sessions,
        cached_grid.as_ref(),
        now,
    );

    let weather_needs_refresh = force
        || weather_stale
        || weekend_weather_needs_refresh(
            &meetings,
            now,
            |meeting_key| {
                db.cache_entry_for_forecast(meeting_key)
                    .ok()
                    .flatten()
                    .map(|entry| cache_is_fresh(&entry, now))
                    .unwrap_or(false)
            },
            |meeting_key| {
                db.cache_entry_for_track_weather(meeting_key)
                    .ok()
                    .flatten()
                    .map(|entry| cache_is_fresh(&entry, now))
                    .unwrap_or(false)
            },
        );

    let needs_refresh = force || grid_stale || weather_needs_refresh;

    if let Some(data) = assemble_weekend_from_cache(
        focus_meeting_key,
        &meetings,
        &calendar.sessions,
        &pinned_numbers,
        cached_grid.clone(),
        None,
        &cached_track,
        &cached_forecasts,
    ) {
        if state.weekend_data().is_none() {
            state.weekend = WeekendLoadState::Ready(LoadedWeekend {
                data,
                stale: !needs_refresh,
            });
        } else if let WeekendLoadState::Ready(loaded) = &mut state.weekend {
            loaded.stale = weather_stale || grid_stale;
        }
    } else if state.weekend_data().is_none() {
        state.weekend = WeekendLoadState::Loading;
    }

    if !needs_refresh {
        return;
    }

    if state.weekend_data().is_some() {
        if let WeekendLoadState::Ready(loaded) = &mut state.weekend {
            loaded.stale = true;
        }
    }

    tasks.push(fetch_weekend_task(
        focus_meeting_key,
        meetings,
        calendar.sessions.clone(),
        pinned_numbers,
        cached_grid,
        cached_track,
        cached_forecasts,
    ));
    state.weekend_refreshing = true;
}

fn fetch_weekend_task(
    focus_meeting_key: i64,
    meetings: Vec<openf1::Meeting>,
    sessions: Vec<Session>,
    pinned_numbers: Vec<i64>,
    cached_grid: Option<QualiGridData>,
    cached_track: HashMap<i64, TrackWeatherData>,
    cached_forecasts: HashMap<i64, ForecastData>,
) -> Task<Message> {
    Task::perform(
        async move {
            fetch_weekend_details(WeekendFetchInput {
                focus_meeting_key,
                meetings,
                sessions,
                pinned_numbers,
                cached_grid,
                cached_sprint_grid: None,
                cached_track,
                cached_forecasts,
            })
            .await
            .map_err(|error| error.to_string())
        },
        Message::WeekendFetched,
    )
}

fn persist_weekend_caches(db: &Database, data: &WeekendDetailData) {
    if let Some(grid) = &data.quali_grid {
        let blob = QualiGridCacheBlob::from_data(grid);
        let _ = db.save_quali_grid_cache(&blob);
    }

    for forecast in &data.forecasts_to_cache {
        let _ = db.save_forecast_cache(forecast);
    }

    for track in &data.tracks_to_cache {
        let blob = TrackWeatherCacheBlob::from_data(track);
        let _ = db.save_track_weather_cache(&blob);
    }
}

fn load_weekend_from_db(db: &Database, state: &AppState) -> Option<WeekendDetailData> {
    let calendar = state.calendar()?;
    let meetings = meetings_for_weather(&calendar.triplet);
    let focus_meeting_key = calendar.triplet.current.meeting_key;
    let pinned_numbers: Vec<i64> = state
        .pinned_drivers
        .iter()
        .map(|pin| pin.driver_number)
        .collect();

    let cached_grid = db.load_quali_grid_cache(focus_meeting_key).ok().flatten()?;
    let mut cached_track = HashMap::new();
    let mut cached_forecasts = HashMap::new();

    for meeting in &meetings {
        if let Ok(Some(track)) = db.load_track_weather_cache(meeting.meeting_key) {
            cached_track.insert(meeting.meeting_key, track);
        }
        if let Ok(Some(forecast)) = db.load_forecast_cache(meeting.meeting_key) {
            cached_forecasts.insert(meeting.meeting_key, forecast);
        }
    }

    assemble_weekend_from_cache(
        focus_meeting_key,
        &meetings,
        &calendar.sessions,
        &pinned_numbers,
        Some(cached_grid),
        None,
        &cached_track,
        &cached_forecasts,
    )
}
