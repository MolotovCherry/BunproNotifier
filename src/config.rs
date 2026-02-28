use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
};

use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use snafu::{Whatever, prelude::*};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub account: Account,
    pub forecast: Forecast,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Account {
    /// TODO: Explain how to get it
    pub api_token: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Forecast {
    /// Notify for reviews hourly or daily (every 24 hours)
    pub interval: ForecastInterval,
    /// Show total review count or new only count
    pub count: ForecastCount,
    /// 0-23 : 24 hour to send notification for Daily interval
    /// defaults to 6 if invalid
    pub daily_time: i8,
    /// 1-65536 : How many hours between each poll (updating information from online)
    pub poll_rate: u16,
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
            daily_time: 6,
            poll_rate: 1,
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
    Daily,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum ForecastCount {
    TotalReviews,
    #[default]
    NewOnly,
}

impl Config {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, Whatever> {
        let path = path.into();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .read(true)
            .open(&path)
            .with_whatever_context(|_| {
                format!("Failed to load config file @ {path}", path = path.display())
            })?;

        let mut data = String::new();
        let read = file.read_to_string(&mut data).with_whatever_context(|_| {
            format!(
                "Failed to read config file as String @ {path}",
                path = path.display()
            )
        })?;

        // if it's a new file, we need to write default config to it
        if read == 0 {
            let config = Self::default();
            let ser = ron::ser::to_string_pretty(&config, PrettyConfig::default())
                .with_whatever_context(|_| {
                    format!(
                        "Failed to serialize config file @ {path}",
                        path = path.display()
                    )
                })?;
            file.write_all(ser.as_bytes()).with_whatever_context(|_| {
                format!(
                    "Failed to write config file @ {path}",
                    path = path.display()
                )
            })?;

            data = ser;
        }

        let config = ron::from_str::<Self>(&data)
            .map_err(Box::new)
            .with_whatever_context(|_| {
                format!(
                    "Failed to deserialize config file @ {path}",
                    path = path.display()
                )
            })?;

        Ok(config)
    }
}
