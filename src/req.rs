use const_format::formatcp;
use snafu::{ResultExt as _, Whatever};
use ureq::{Body, http::Response};

use crate::{
    config::{Config, ForecastInterval},
    objects::{ForecastDaily, ForecastHourly, TotalDue},
};

const BASE: &str = "https://api.bunpro.jp/api";
const FRONTEND: &str = formatcp!("{BASE}/frontend");
const USER: &str = formatcp!("{FRONTEND}/user");
const DUE: &str = formatcp!("{USER}/due");
const USER_STATS: &str = formatcp!("{FRONTEND}/user_stats");
const FORECAST_DAILY: &str = formatcp!("{USER_STATS}/forecast_daily");
const FORECAST_HOURLY: &str = formatcp!("{USER_STATS}/forecast_hourly");

const USER_AGENT: &str = formatcp!("bp-notifier/{version}", version = env!("CARGO_PKG_VERSION"));

pub enum Forecast {
    Hourly(ForecastHourly),
    Daily(ForecastDaily),
}

impl From<ForecastHourly> for Forecast {
    fn from(value: ForecastHourly) -> Self {
        Self::Hourly(value)
    }
}

impl From<ForecastDaily> for Forecast {
    fn from(value: ForecastDaily) -> Self {
        Self::Daily(value)
    }
}

pub fn get_forecast(config: &Config) -> Result<Forecast, Whatever> {
    let fc = match config.forecast.interval {
        ForecastInterval::Hourly => get_query(FORECAST_HOURLY, config)?
            .body_mut()
            .read_json::<ForecastHourly>()
            .whatever_context("forecast hourly json parse failed")?
            .into(),

        ForecastInterval::Daily { .. } => get_query(FORECAST_DAILY, config)?
            .body_mut()
            .read_json::<ForecastDaily>()
            .whatever_context("forecast daily json parse failed")?
            .into(),
    };

    Ok(fc)
}

pub fn get_due(config: &Config) -> Result<TotalDue, Whatever> {
    get_query(DUE, config)?
        .body_mut()
        .read_json()
        .whatever_context("forecast daily json parse failed")
}

fn get_query(uri: &str, config: &Config) -> Result<Response<Body>, Whatever> {
    let token = format!("Token token={token}", token = config.account.api_token);

    ureq::get(uri)
        .header("Authorization", token)
        .header("User-Agent", USER_AGENT)
        .call()
        .whatever_context("api call failed")
}
