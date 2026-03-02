use std::{
    fs::OpenOptions,
    io::{self, ErrorKind, Read, Write},
    path::PathBuf,
};

use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display("couldn't find config file config.ron next to binary"))]
    PathNotFound { source: io::Error },
    #[snafu(display("Failed to read config file as String"))]
    String { source: io::Error },
    #[snafu(display("Failed to serialize config file"))]
    Serialize { source: ron::Error },
    #[snafu(display("Failed to deserialize config file"))]
    Deserialize { source: ron::de::SpannedError },
    #[snafu(display("{source}"))]
    Io { source: io::Error },
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub account: Account,
    pub forecast: Forecast,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Account {
    /// bunpro api token
    pub api_token: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Forecast {
    /// 1-65536 : How many minutes between each data update.
    /// Grabs updated information from bunpro api; you'll want this
    /// at quicker rates if actively doing reviews and interval is set
    /// to hourly, as the program's cached info could get stale.
    ///
    /// if interval is daily, interval should be set much higher, as
    /// there's no need to refresh every minute
    pub update_rate: u16,
    /// Notify for reviews hourly or daily (every 24 hours)
    pub interval: ForecastInterval,
    /// Show total review count or new only count
    pub count: ForecastCount,
    /// Notify for new grammar reviews
    pub grammar: bool,
    /// Notify for new vocab reviews
    pub vocab: bool,
    /// Notify what reviews are available on startup
    pub initial_notify: bool,
}

impl Default for Forecast {
    fn default() -> Self {
        Self {
            interval: Default::default(),
            count: Default::default(),
            update_rate: 1,
            grammar: true,
            vocab: true,
            initial_notify: true,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum ForecastInterval {
    #[default]
    Hourly,
    /// 0-23 : What hour to send notification for Daily interval
    /// defaults to 6 if invalid
    Daily { hour: i8 },
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum ForecastCount {
    TotalReviews,
    #[default]
    NewOnly,
}

impl Config {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let path = path.into();
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .read(true)
            .open(&path);

        let mut file = match file {
            Ok(f) => f,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => return Err(e).context(PathNotFoundSnafu),
                _ => return Err(e).context(IoSnafu),
            },
        };

        let mut data = String::new();
        let read = file.read_to_string(&mut data).context(StringSnafu)?;

        // if it's a new file, we need to write default config to it
        if read == 0 {
            let config = Self::default();
            let ser = ron::ser::to_string_pretty(&config, PrettyConfig::default())
                .context(SerializeSnafu)?;
            file.write_all(ser.as_bytes()).context(IoSnafu)?;

            data = ser;
        }

        let config = ron::from_str::<Self>(&data).context(DeserializeSnafu)?;

        Ok(config)
    }
}
