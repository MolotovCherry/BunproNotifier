use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use jiff::{Span, ToSpan, Unit, Zoned, civil::Time};
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
            let mut now = Zoned::now().time();

            let mut data = combine_records(data, total_due, &config, now);

            // do initial notify, and remove current hour from data
            if initial_notify && let Some(data) = data.remove(&now.hour()) {
                notify(&data, &mut notification, &config);
            }

            while let WakeReason::Timeout = sleep_until_next_hour(&abortable, now) {
                now = Zoned::now().time();

                // there should always be a next hour. If there isn't, we've hit the end of our available data
                // and need to repoll
                let Some(count) = data.remove(&now.hour()) else {
                    trace!("hit next hour, but there was no data left; repolling");

                    // artificially cause repoll
                    token.abort();
                    break;
                };

                notify(&count, &mut notification, &config);
            }

            trace!("aborting run");
        });

        abort_token
    }
}

fn notify(count: &Count, notification: &mut Notification, config: &Config) {
    let has_grammar = config.forecast.grammar && count.grammar > 0;
    let has_vocab = config.forecast.vocab && count.vocab > 0;
    if !has_grammar && !has_vocab {
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
fn sleep_until_next_hour(abortable: &AbortableSleep, time: Time) -> WakeReason {
    let now = Span::new()
        .minutes(time.minute())
        .seconds(time.second())
        .milliseconds(time.millisecond())
        .microseconds(time.microsecond())
        .nanoseconds(time.nanosecond());

    let remainder = 1.hour().checked_sub(now).expect("now cannot be > 1 hour");
    let total = remainder.total(Unit::Second).expect("total to succeed") as u64;

    trace!("sleeping for {total} seconds til next hour");

    abortable.sleep(Duration::from_secs(total))
}

type Hour = i8;

#[derive(Default, Debug)]
struct Count {
    grammar: CardCount,
    vocab: CardCount,
}

fn combine_records(
    data: ForecastHourly,
    total_due: Option<TotalDue>,
    config: &Config,
    now: Time,
) -> HashMap<Hour, Count> {
    let mut records: HashMap<_, Count> = HashMap::new();

    for (zone, count) in data.grammar.rest {
        let hour = zone.0.hour();
        records.entry(hour).or_default().grammar = count;
    }

    for (zone, count) in data.vocab.rest {
        let hour = zone.0.hour();
        records.entry(hour).or_default().vocab = count;
    }

    // hour wraparound iterator; stops iteration at now.hour - 1
    let hours = (0..24).map(|x| (x + now.hour()) % 24);

    let mut total_reviews_grammar = total_due.map(|t| t.total_due_grammar).unwrap_or(0);
    let mut total_reviews_vocab = total_due.map(|t| t.total_due_vocab).unwrap_or(0);

    // start at current hour, then wrap around
    for h in hours {
        let entry = records.entry(h).or_default();

        if config.forecast.count == ForecastCount::TotalReviews {
            total_reviews_grammar += entry.grammar;
            entry.grammar = total_reviews_grammar;
            total_reviews_vocab += entry.vocab;
            entry.vocab = total_reviews_vocab;
        }
    }

    records
}
