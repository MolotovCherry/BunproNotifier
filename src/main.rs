#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]

mod config;
#[cfg(windows)]
mod event_loop;
mod objects;
mod parker;
#[cfg(windows)]
mod popup;
mod req;
mod run;
mod setup;

use std::env;

use include_dir::{Dir, include_dir};
use snafu::{OptionExt, ResultExt, Whatever};

use crate::{config::Config, setup::setup};

static ASSETS: Dir = include_dir!("assets/");

#[allow(unused)]
const APP_ID: &str = "com.cherry.BunproNotifier";

#[snafu::report]
fn main() -> Result<(), Whatever> {
    setup()?;

    let path = env::current_exe().whatever_context("failed to get current_exe")?;
    let parent = path.parent().whatever_context("failed to get exe parent")?;
    let config = Config::new(parent.join("config.ron"))?;
    run::run(config);
    Ok(())
}
