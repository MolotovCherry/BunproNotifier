use std::{sync::Arc, thread, time::Duration};

use notify_rust::Notification;

use crate::{
    config::Config,
    objects::ForecastDaily,
    parker::{AbortToken, AbortableSleep},
};

pub struct Daily;

impl Daily {
    pub fn run(
        data: ForecastDaily,
        config: Arc<Config>,
        initial_notify: bool,
        token: AbortToken,
        notification: Notification,
    ) -> AbortToken {
        let (abortable, abort_token) = AbortableSleep::new();

        thread::spawn(move || {
            let reason = abortable.sleep(Duration::from_secs(5));
            println!("Thread stopped with reason: {reason:?}");
        });

        abort_token
    }
}
