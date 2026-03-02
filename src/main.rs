#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]

mod config;
mod notification;
mod objects;
mod parker;
#[cfg(windows)]
mod popup;
mod req;
mod run;
mod setup;
mod tray;

use std::{
    env,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{TryRecvError, channel, sync_channel},
    },
    thread,
    time::Duration,
};

use include_dir::{Dir, include_dir};
use log::error;
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, new_debouncer,
    notify::{Event, EventKind, RecursiveMode},
};
use snafu::{OptionExt, ResultExt, Whatever};

use crate::{
    config::{Config, ConfigError},
    popup::epopup,
    run::Run,
    setup::setup,
    tray::{AppTray, EXIT},
};

static ASSETS: Dir = include_dir!("assets/");

#[allow(unused)]
const APP_ID: &str = "com.cherry.BunproNotifier";

#[snafu::report]
fn main() -> Result<(), Whatever> {
    setup()?;

    let mut runner = Run::new();
    let abort_token = runner.abort_token();
    let stop_run = runner.stop_guard();

    let path = env::current_exe().whatever_context("failed to get current_exe")?;
    let parent = path.parent().whatever_context("failed to get exe parent")?;
    let config_path = parent.join("config.ron");

    let (tx, rx) = sync_channel(1);

    let handle = thread::spawn(move || {
        let stop = runner.stop_guard();
        let (config_tx, config_rx) = channel();

        let initial = Arc::new(AtomicBool::default());
        let get_config = {
            let config_path = config_path.clone();
            let popup = initial.clone();

            move || match Config::new(config_path.clone()) {
                Ok(c) => Some(c),
                Err(e) => match e {
                    ConfigError::PathNotFound { source } | ConfigError::Io { source } => {
                        let msg = format!(
                            "failed to get config. was it moved, renamed, or missing?\n{source}"
                        );
                        epopup!(msg);
                        error!("{msg}");
                        None
                    }

                    ConfigError::String { source } => {
                        let msg = format!("config file is not in a readable format.\n{source}");
                        epopup!(msg);
                        error!("{msg}");
                        None
                    }

                    ConfigError::Serialize { source } => {
                        let msg = format!("failed to serialize config struct? what?\n{source}");
                        if popup.load(Ordering::Relaxed) {
                            epopup!(msg);
                        }
                        error!("{msg}");
                        None
                    }

                    ConfigError::Deserialize { source } => {
                        let msg = format!("Config file has syntax error:\n{source}");
                        epopup!(msg);
                        error!("{msg}");
                        None
                    }
                },
            }
        };

        // provide initial config value for startup
        if let Some(config) = get_config() {
            _ = config_tx.send(config);
        }

        initial.swap(false, Ordering::Relaxed);

        let mut debouncer = new_debouncer(Duration::from_secs(1), None, {
            move |result: DebounceEventResult| {
                if let Ok(events) = result
                    && events.into_iter().any(|i| {
                        matches!(
                            i,
                            DebouncedEvent {
                                event: Event {
                                    kind: EventKind::Modify(_),
                                    ..
                                },
                                ..
                            }
                        )
                    })
                    && let Some(config) = get_config()
                {
                    _ = config_tx.send(config);
                    stop.stop();
                }
            }
        })
        .expect("debouncer to open");

        debouncer
            .watch(&config_path, RecursiveMode::NonRecursive)
            .expect("failed to watch config file. was it moved, renamed, or missing?");

        while let Ok(config) = config_rx.recv() {
            if !runner.run(config) {
                // panicked
                break;
            }

            // event loop quit and is signalling us to also quit
            match rx.try_recv() {
                // got signal
                Ok(_) | Err(TryRecvError::Disconnected) => break,
                // no signal
                Err(TryRecvError::Empty) => (),
            }
        }

        EXIT.signal();
    });

    AppTray::run(abort_token);

    // graceful stop
    _ = tx.try_send(());
    stop_run.stop();
    _ = handle.join();

    Ok(())
}
