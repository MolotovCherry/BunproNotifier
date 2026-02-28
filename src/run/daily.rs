use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use jiff::{
    Zoned,
    civil::{Date, DateTime},
};
use log::trace;
use notify_rust::Notification;

use crate::{
    config::{Config, ForecastCount},
    objects::{CardCount, ForecastDaily, TotalDue},
    parker::{AbortToken, AbortableSleep, WakeReason},
};

pub struct Daily;

impl Daily {
    pub fn run(
        data: ForecastDaily,
        total_due: Option<TotalDue>,
        config: Arc<Config>,
        initial_notify: bool,
        token: AbortToken,
        mut notification: Notification,
    ) -> AbortToken {
        let (abortable, abort_token) = AbortableSleep::new();

        thread::spawn(move || {
            let mut today = Zoned::now().datetime();

            let mut data = combine_records(data, total_due, &config, today.date());

            // do initial notify, and remove current hour from data
            if initial_notify && let Some(data) = data.remove(&today.date()) {
                notify(&data, &mut notification, &config);
            }

            while let WakeReason::Timeout = sleep_until(&config, &abortable, &today) {
                today = Zoned::now().datetime();

                // there should always be a next hour. If there isn't, we've hit the end of our available data
                // and need to repoll
                let Some(count) = data.remove(&today.date()) else {
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

/// Sleep until the next day at hour in config (if out of range, defaults to 6am)
/// Returns the wake reason
fn sleep_until(config: &Config, abortable: &AbortableSleep, now: &DateTime) -> WakeReason {
    let hour = match config.forecast.daily_time {
        t @ 0..24 => t,
        _ => 6,
    };

    let date = now.date().tomorrow().expect("not 9999").at(hour, 0, 0, 0);

    let next = now.duration_until(date);

    trace!("sleeping for {} seconds til next day", next.as_secs());

    abortable.sleep(Duration::from_secs(next.as_secs() as _))
}

#[derive(Default, Debug)]
struct Count {
    grammar: CardCount,
    vocab: CardCount,
}

fn combine_records(
    data: ForecastDaily,
    total_due: Option<TotalDue>,
    config: &Config,
    today: Date,
) -> HashMap<Date, Count> {
    let mut records = HashMap::new();

    records.insert(
        today,
        Count {
            grammar: data.grammar.later,
            vocab: data.vocab.later,
        },
    );

    records.insert(
        today.tomorrow().expect("date to not be 9999"),
        Count {
            grammar: data.grammar.tomorrow,
            vocab: data.vocab.tomorrow,
        },
    );

    for (date, count) in data.grammar.rest {
        records.insert(
            date,
            Count {
                grammar: count,
                vocab: 0,
            },
        );
    }

    for (date, count) in data.vocab.rest {
        records
            .entry(date)
            .and_modify(|c| c.vocab = count)
            .or_insert(Count {
                grammar: 0,
                vocab: count,
            });
    }

    let mut total_grammar = total_due.map(|t| t.total_due_grammar).unwrap_or(0);
    let mut total_vocab = total_due.map(|t| t.total_due_vocab).unwrap_or(0);
    let mut date = today;

    if config.forecast.count == ForecastCount::TotalReviews {
        loop {
            let Some(count) = records.get_mut(&date) else {
                break;
            };

            total_grammar += count.grammar;
            total_vocab += count.vocab;

            count.grammar = total_grammar;
            count.vocab = total_vocab;

            date = date.tomorrow().expect("date to not be 9999");
        }
    }

    records
}
