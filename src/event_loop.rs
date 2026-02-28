#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostQuitMessage, TranslateMessage,
};

#[cfg(not(windows))]
compile_error!("TODO: Support other langs with EventLoop for tray icon");

pub struct EventLoop;

impl EventLoop {
    pub fn new() -> Self {
        Self
    }

    #[cfg(windows)]
    pub fn run(&self, mut cb: impl FnMut(&Self)) {
        let mut msg = MSG::default();

        fn get_msg(msg: &mut MSG) -> bool {
            let res = unsafe { GetMessageW(msg, None, 0, 0) };
            res.as_bool()
        }

        while get_msg(&mut msg) {
            _ = unsafe { TranslateMessage(&msg) };
            unsafe {
                DispatchMessageW(&msg);
            }

            cb(self);
        }
    }

    #[cfg(windows)]
    pub fn exit(&self) {
        unsafe {
            PostQuitMessage(0);
        }
    }
}
