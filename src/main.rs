#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]

mod config;
mod objects;
mod parker;
#[cfg(windows)]
mod popup;
mod req;
mod run;
mod setup;

use include_dir::{Dir, include_dir};
use snafu::Whatever;

use crate::{config::Config, setup::setup};

static ASSETS: Dir = include_dir!("assets/");

#[allow(unused)]
const APP_ID: &str = "com.cherry.BunproNotifier";

#[snafu::report]
fn main() -> Result<(), Whatever> {
    setup()?;

    let config = Config::new("config.ron")?;
    run::run(config);
    Ok(())
}
