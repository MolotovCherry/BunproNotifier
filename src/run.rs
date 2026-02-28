mod daily;
mod hourly;

use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use log::trace;

use crate::{
    config::{Config, ForecastCount},
    notification::Notification,
    parker::{AbortToken, AbortableSleep},
    req::{Forecast, get_due, get_forecast},
};
use daily::Daily;
use hourly::Hourly;

pub static ABORT_TOKEN: OnceLock<AbortToken> = OnceLock::new();

pub fn run(config: Config) {
    let mut n = Notification::new("Bunpro");

    if !config.forecast.grammar && !config.forecast.vocab {
        n
            .summary("Error")
            .body("Both forecast.grammar and forecast.vocab cannot be false. Please set one or both to true")
            .show();

        return;
    }

    let config = Arc::new(config);
    let poll_rate = config.forecast.poll_rate.clamp(1, u16::MAX) as u64;
    let mut run_abort_token = None::<AbortToken>;
    let mut initial_notify = config.forecast.initial_notify;
    let (abortable, abort_token) = AbortableSleep::new();

    ABORT_TOKEN.set(abort_token.clone()).unwrap();

    loop {
        // abort current running thread
        if let Some(token) = run_abort_token.take() {
            trace!("aborting run task");
            token.abort();
        }

        trace!("run get_forecast()");

        let forecast = match get_forecast(&config) {
            Ok(v) => Some(v),
            Err(e) => {
                trace!("failed to get forecast data: {e}");
                n.summary("Error")
                    .body(&format!("Failed to get forecast: {e}"))
                    .show();

                None
            }
        };

        let total_due = if config.forecast.count == ForecastCount::TotalReviews {
            trace!("run get_due()");

            match get_due(&config) {
                Ok(v) => Some(v),
                Err(e) => {
                    trace!("failed to get total due data: {e}");
                    n.summary("Error")
                        .body(&format!("Failed to get total due: {e}"))
                        .show();

                    None
                }
            }
        } else {
            None
        };

        match forecast {
            Some(Forecast::Daily(daily)) => {
                let token = Daily::run(
                    daily,
                    total_due,
                    config.clone(),
                    initial_notify,
                    abort_token.clone(),
                    n.clone(),
                );
                run_abort_token = Some(token);
            }

            Some(Forecast::Hourly(hourly)) => {
                let token = Hourly::run(
                    hourly,
                    total_due,
                    config.clone(),
                    initial_notify,
                    abort_token.clone(),
                    n.clone(),
                );
                run_abort_token = Some(token);
            }

            None => (),
        }

        trace!("sleeping {poll_rate} hours until next poll");

        initial_notify = false;

        // keep scanning for set amount of time, unless
        // runnable aborts this to request fresh data (for example, if it ran out)
        abortable.sleep(Duration::from_hours(poll_rate));
    }
}
