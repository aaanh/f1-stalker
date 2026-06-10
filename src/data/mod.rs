pub mod cache;
pub mod cache_policy;
pub mod championship;
pub mod drivers;
pub mod grid;
pub mod openf1;
pub mod open_meteo;
pub mod track_weather;
pub mod weekend;

pub use cache::{
    calendar_from_cache, drivers_from_cache, CalendarCacheBlob, CacheError, DriversCacheBlob,
};
pub use cache_policy::{
    cache_is_fresh, championship_needs_refresh, quali_grid_needs_refresh,
    weekend_weather_needs_refresh,
};
pub use championship::{
    championship_from_cache, fetch_season_championship, ChampionshipCacheBlob, ChampionshipData,
    FetchError as ChampionshipFetchError,
};
pub use drivers::{fetch_season_drivers, DriversData, FetchError as DriversFetchError};
pub use grid::{
    fetch_quali_grid, quali_grid_from_cache, QualiGridCacheBlob, QualiGridData,
    FetchError as GridFetchError,
};
pub use openf1::{fetch_season_calendar, CalendarData, FetchError};
pub use open_meteo::{fetch_meeting_forecast, ForecastData, FetchError as OpenMeteoFetchError};
pub use track_weather::{
    fetch_track_weather, track_weather_from_cache, TrackWeatherCacheBlob, TrackWeatherData,
    FetchError as TrackWeatherFetchError,
};
pub use weekend::{
    assemble_weekend_from_cache, fetch_weekend_details, meetings_for_weather, WeekendDetailData,
    WeekendFetchInput,
};
