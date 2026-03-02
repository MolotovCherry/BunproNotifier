mod daily;
mod hourly;

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
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

pub struct StopRun {
    abort_token: Arc<AbortToken>,
    stop_flag: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
}

impl StopRun {
    pub fn stop(&self) {
        // do not stop unless already running. otherwise when it does run next, it'll immediately exit
        if self.running.swap(false, Ordering::Relaxed) {
            self.stop_flag.store(true, Ordering::Release);
            self.abort_token.abort();
        }
    }
}

pub struct Run {
    abort_token: Arc<AbortToken>,
    abortable: Option<AbortableSleep>,
    stop_flag: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
}

impl Run {
    pub fn new() -> Self {
        let (abortable, abort_token) = AbortableSleep::new();
        Self {
            abort_token: Arc::new(abort_token),
            stop_flag: Arc::default(),
            abortable: Some(abortable),
            running: Arc::default(),
        }
    }

    pub fn abort_token(&self) -> Arc<AbortToken> {
        self.abort_token.clone()
    }

    pub fn stop_guard(&self) -> StopRun {
        StopRun {
            abort_token: self.abort_token.clone(),
            stop_flag: self.stop_flag.clone(),
            running: self.running.clone(),
        }
    }

    pub fn run(&mut self, config: Config) -> bool {
        let Some(abortable) = self.abortable.take() else {
            return false;
        };
        let abort_token = self.abort_token().clone();
        let stop_flag = self.stop_flag.clone();
        let running = self.running.clone();

        let handle = thread::spawn(move || {
            let mut n = Notification::new("Bunpro");

            if !config.forecast.grammar && !config.forecast.vocab {
                n
                .summary("Error")
                .body("Both forecast.grammar and forecast.vocab cannot be false. Please set one or both to true")
                .show();

                return abortable;
            }

            let config = Arc::new(config);
            let poll_rate = config.forecast.update_rate.clamp(1, u16::MAX) as u64;
            let mut run_abort_token = None::<AbortToken>;
            let mut initial_notify = config.forecast.initial_notify;

            loop {
                // abort current running thread
                if let Some(token) = run_abort_token.take() {
                    trace!("aborting run task");
                    token.abort();
                }

                if stop_flag.swap(false, Ordering::Relaxed) {
                    break abortable;
                }

                running.store(true, Ordering::Relaxed);

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

                trace!("sleeping {poll_rate} minutes until next poll");

                initial_notify = false;

                // keep scanning for set amount of time, unless
                // runnable aborts this to request fresh data (for example, if it ran out)
                abortable.sleep(Duration::from_mins(poll_rate));
            }
        });

        match handle.join() {
            Ok(abortable) => {
                self.abortable = Some(abortable);
                true
            }

            // panic
            Err(_) => false,
        }
    }
}
