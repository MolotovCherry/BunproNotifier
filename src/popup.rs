use std::iter;

use windows::{
    Win32::UI::WindowsAndMessaging::{
        MB_ICONERROR, MB_ICONINFORMATION, MB_ICONWARNING, MESSAGEBOX_STYLE, MessageBoxW,
    },
    core::PCWSTR,
};

pub enum MessageBoxIcon {
    Info,
    Warn,
    Error,
}

impl From<MessageBoxIcon> for MESSAGEBOX_STYLE {
    fn from(value: MessageBoxIcon) -> Self {
        match value {
            MessageBoxIcon::Info => MB_ICONINFORMATION,
            MessageBoxIcon::Warn => MB_ICONWARNING,
            MessageBoxIcon::Error => MB_ICONERROR,
        }
    }
}

pub fn popup<T: AsRef<str>, M: AsRef<str>>(title: T, message: M, icon: MessageBoxIcon) {
    let title = title.as_ref();
    let message = message.as_ref();

    // these must be explicitly assigned, otherwise they will be temporary and drop
    // and create an invalid pointer, causing corruption and UB
    let title = title
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<_>>();
    let msg = message
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<_>>();

    unsafe {
        MessageBoxW(
            None,
            PCWSTR(msg.as_ptr()),
            PCWSTR(title.as_ptr()),
            icon.into(),
        );
    }
}

macro_rules! epopup {
    ($error:ident) => {
        #[cfg(windows)]
        {
            use crate::popup::{MessageBoxIcon, popup};
            popup(
                "Bunpro Notifier Error",
                $error.to_string(),
                MessageBoxIcon::Error,
            );
        }
    };
}

pub(crate) use epopup;
