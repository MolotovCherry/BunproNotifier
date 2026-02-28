use tray_icon::{
    Icon, TrayIconBuilder, TrayIconEvent,
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

use crate::run::ABORT_TOKEN;

static ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tray.bin"));

pub struct AppTray;

impl AppTray {
    pub fn run() {
        let icon = Icon::from_rgba(ICON.to_owned(), 256, 256).expect("rgba creation to succeed");

        let tray_menu = Menu::new();

        let quit = MenuItem::new("Quit", true, None);
        let refresh = MenuItem::new("Refresh Data", true, None);

        let authors = env!("CARGO_PKG_AUTHORS")
            .split(':')
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        let title = "Bunpro Notifier";

        tray_menu
                .append_items(&[
                    &refresh,
                    &PredefinedMenuItem::about(
                        None,
                        Some(AboutMetadata {
                            name: Some(title.to_owned()),
                            copyright: Some(format!("Copyright (c) {}", authors.join(", ")).to_owned()),
                            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
                            authors: Some(authors),
                            license: Some(env!("CARGO_PKG_LICENSE").to_owned()),
                            website: Some(env!("CARGO_PKG_HOMEPAGE").to_owned()),
                            comments: Some(r#"THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE."#.to_owned()),
                            ..Default::default()
                        }),
                    ),
                    &PredefinedMenuItem::separator(),
                    &quit,
                ])
                .unwrap();

        let _tray_icon = TrayIconBuilder::new()
            .with_tooltip(title)
            .with_menu(Box::new(tray_menu))
            .with_icon(icon)
            .build()
            .unwrap();

        let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);

        let proxy = event_loop.create_proxy();
        TrayIconEvent::set_event_handler(Some(move |event| {
            _ = proxy.send_event(UserEvent::TrayIconEvent(event));
        }));

        let proxy = event_loop.create_proxy();
        MenuEvent::set_event_handler(Some(move |event| {
            _ = proxy.send_event(UserEvent::MenuEvent(event));
        }));

        let mut app = TrayHandler { quit, refresh };
        event_loop.run_app(&mut app).unwrap();
    }
}

#[expect(unused)]
enum UserEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
}

struct TrayHandler {
    quit: MenuItem,
    refresh: MenuItem,
}

impl ApplicationHandler<UserEvent> for TrayHandler {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::TrayIconEvent(_) => (),
            UserEvent::MenuEvent(menu_event) => {
                if menu_event.id == self.quit.id() {
                    event_loop.exit();
                }

                if menu_event.id == self.refresh.id()
                    && let Some(token) = ABORT_TOKEN.get()
                {
                    token.abort();
                }
            }
        }
    }
}
