use std::{
    fs,
    io::ErrorKind,
    iter, panic,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use env_logger::Env;
use log::{LevelFilter, error};
use snafu::{OptionExt, ResultExt, Whatever};

use crate::{APP_ID, ASSETS, popup::epopup};

pub fn setup() -> Result<(), Whatever> {
    set_hook();

    let env = Env::new().filter("BP_LOG").write_style("BP_STYLE");

    env_logger::builder()
        .format_timestamp(None)
        .filter_level(if cfg!(debug_assertions) {
            LevelFilter::Info
        } else {
            LevelFilter::Debug
        })
        .parse_env(env)
        .init();

    let path = match copy_assets() {
        Ok(v) => v,
        Err(e) => {
            error!("{e}");
            epopup!(e);
            return Err(e);
        }
    };

    #[cfg(windows)]
    if let Err(e) = windows(&path) {
        error!("{e}");
        epopup!(e);
        return Err(e);
    }

    Ok(())
}

fn copy_assets() -> Result<PathBuf, Whatever> {
    let project_dirs = ProjectDirs::from("", "", APP_ID).whatever_context("BaseDirs failed")?;
    let path = project_dirs.cache_dir();

    // noop copy if it path already exists
    if path.exists() {
        return Ok(path.to_path_buf());
    }

    if let Err(e) = fs::create_dir_all(path)
        && e.kind() != ErrorKind::AlreadyExists
    {
        return Err(e).whatever_context("create_dir_all failure");
    }

    for entry in ASSETS.entries() {
        if let Some(dir) = entry.as_dir() {
            let path = path.join(dir.path());

            if let Err(e) = fs::create_dir_all(path)
                && e.kind() != ErrorKind::AlreadyExists
            {
                return Err(e).whatever_context("create_dir_all failure");
            }
        }

        if let Some(file) = entry.as_file() {
            let path = path.join(file.path());

            if let Err(e) = fs::write(path, file.contents()) {
                return Err(e).whatever_context("Tempdir setup failed to copy file");
            }
        }
    }

    Ok(path.to_path_buf())
}

fn windows(tempdir: &Path) -> Result<(), Whatever> {
    use const_format::formatcp;
    use sayuri::macros::tri;
    use snafu::{ResultExt as _, Whatever};
    use windows::{Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID, core::PCWSTR};
    use winreg::{
        RegKey,
        enums::{HKEY_CURRENT_USER, RegDisposition},
    };

    let id = APP_ID
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<_>>();

    if let Err(e) = unsafe { SetCurrentProcessExplicitAppUserModelID(PCWSTR(id.as_ptr())) } {
        return Err(e).whatever_context("Failed to SetCurrentProcessExplicitAppUserModelID");
    }

    // Setup App ID, name, and icon uri for toast notifs
    // https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/send-local-toast-other-apps#step-1-register-your-app-in-the-registry
    const APP_KEY: &str = formatcp!(r"Software\Classes\AppUserModelId\{APP_ID}");

    let icon = tempdir.join("logo.png");

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let res: Result<_, Whatever> = tri!({
        let (key, disposition) = hkcu
            .create_subkey(APP_KEY)
            .with_whatever_context(|e| format!("failed to open subkey: {e}"))?;

        if disposition == RegDisposition::REG_CREATED_NEW_KEY {
            key.set_value("DisplayName", &"BunproNotifier")
                .with_whatever_context(|e| format!("failed to set DisplayName: {e}"))?;
            key.set_value("IconUri", &icon.as_os_str())
                .with_whatever_context(|e| format!("failed to set DisplayName: {e}"))?;
            key.set_value("IconBackgroundColor", &"0")
                .with_whatever_context(|e| format!("failed to set IconBackgroundColor: {e}"))?;
        }

        Ok(())
    });

    if let Err(e) = res {
        return Err(e).whatever_context("Registry setup failed");
    }

    Ok(())
}

fn set_hook() {
    panic::set_hook(Box::new(|info| {
        epopup!(info);

        eprintln!("{info}");
    }));
}
