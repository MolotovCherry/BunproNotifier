use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use jiff::Zoned;
use log::trace;
use notify_rust::Notification;

use crate::{
    config::{Config, ForecastCount},
    objects::{CardCount, ForecastHourly, TotalDue},
    parker::{AbortToken, AbortableSleep, WakeReason},
};

pub struct Hourly;

impl Hourly {
    pub fn run(
        data: ForecastHourly,
        total_due: Option<TotalDue>,
        config: Arc<Config>,
        initial_notify: bool,
        token: AbortToken,
        mut notification: Notification,
    ) -> AbortToken {
        let (abortable, abort_token) = AbortableSleep::new();

        thread::spawn(move || {
            let mut now = Now::new();

            let mut data = combine_records(data, total_due, &config, now);

            // do initial notify, and remove current hour from data
            if initial_notify && let Some(data) = data.remove(&now.hour) {
                notify(&data, &mut notification, &config);
            }

            while let WakeReason::Timeout = sleep_until_next_hour(&abortable, now) {
                // there should always be a next hour. If there isn't, we've hit the end of our available data
                // and need to repoll
                let Some(count) = data.remove(&now.hour) else {
                    trace!("hit next hour, but there was no data left; repolling");

                    // artificially cause repoll
                    token.abort();
                    break;
                };

                notify(&count, &mut notification, &config);

                now = Now::new();
            }

            trace!("aborting run");
        });

        abort_token
    }
}

fn notify(count: &Count, notification: &mut Notification, config: &Config) {
    let needs_grammar = config.forecast.grammar && count.grammar > 0;
    let needs_vocab = config.forecast.vocab && count.vocab > 0;
    if !needs_grammar && !needs_vocab {
        return;
    }

    let body = {
        let reviews_text = match config.forecast.count {
            ForecastCount::TotalReviews => "total reviews",
            ForecastCount::NewOnly => "new reviews",
        };

        let mut body = String::new();

        if config.forecast.grammar && count.grammar > 0 {
            body.push_str(&format!(
                "Grammar: {count} {reviews_text}",
                count = count.grammar
            ));
        }

        if config.forecast.vocab && count.vocab > 0 {
            body.push_str(&format!(
                "\nVocab: {count} {reviews_text}",
                count = count.vocab
            ));
        }

        body
    };

    _ = notification.summary("Reviews Due").body(&body).show();
}

/// Sleep until the next hour
/// Returns the next hour and the wake reason
fn sleep_until_next_hour(abortable: &AbortableSleep, now: Now) -> WakeReason {
    let Now { minute, second, .. } = now;

    // the time in seconds in the current hour
    let now_secs = minute as u64 * 60 + second as u64;
    // how long in seconds is left until the next hour tick
    let wait_for = (60u64 * 60).saturating_sub(now_secs);

    trace!("sleeping for {wait_for} seconds til next hour");

    abortable.sleep(Duration::from_secs(wait_for))
}

type Hour = i8;
type Minute = i8;
type Second = i8;

#[derive(Copy, Clone)]
struct Now {
    hour: Hour,
    minute: Minute,
    second: Second,
}

impl Now {
    fn new() -> Self {
        let now = Zoned::now();

        trace!(
            "now() got {hour}:{minute}:{second}",
            hour = now.hour(),
            minute = now.minute(),
            second = now.second()
        );

        Now {
            hour: now.hour(),
            minute: now.minute(),
            second: now.second(),
        }
    }
}

impl Default for Now {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Debug)]
struct Count {
    grammar: CardCount,
    vocab: CardCount,
}

fn combine_records(
    data: ForecastHourly,
    total_due: Option<TotalDue>,
    config: &Config,
    now: Now,
) -> HashMap<Hour, Count> {
    let mut records = HashMap::new();

    for (zone, count) in data.grammar.rest {
        let hour = zone.0.hour();
        let record: &mut Count = records.entry(hour).or_default();
        record.grammar = count;
    }

    for (zone, count) in data.vocab.rest {
        let hour = zone.0.hour();
        let record = records.entry(hour).or_default();
        record.vocab = count;
    }

    // hour wraparound iterator; stops iteration at now.hour - 1
    let hours = (0..24).map(|x| (x + now.hour) % 24);

    let mut total_reviews_grammar = total_due.map(|t| t.total_due_grammar).unwrap_or(0);
    let mut total_reviews_vocab = total_due.map(|t| t.total_due_vocab).unwrap_or(0);

    // start at current hour, then wrap around
    for h in hours {
        let entry = records.entry(h).or_default();

        if matches!(config.forecast.count, ForecastCount::TotalReviews) {
            total_reviews_grammar += entry.grammar;
            entry.grammar = total_reviews_grammar;
            total_reviews_vocab += entry.vocab;
            entry.vocab = total_reviews_vocab;
        }
    }

    records
}
