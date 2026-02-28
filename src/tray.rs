use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

use crate::event_loop::EventLoop;

pub struct AppTray;

impl AppTray {
    pub fn run() {
        let icon = Icon::from_resource(1, None).unwrap();

        let tray_menu = Menu::new();

        let quit_i = MenuItem::new("Quit", true, None);

        let authors = env!("CARGO_PKG_AUTHORS")
            .split(':')
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        let title = "Bunpro Notifier";

        tray_menu
                .append_items(&[
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
                    &quit_i,
                ])
                .unwrap();

        let mut tray_icon = Some(
            TrayIconBuilder::new()
                .with_tooltip(title)
                .with_menu(Box::new(tray_menu))
                .with_icon(icon)
                .build()
                .unwrap(),
        );

        #[cfg(windows)]
        EventLoop::new().run(move |event_loop, _| {
            if let Ok(event) = MenuEvent::receiver().try_recv()
                && event.id == quit_i.id()
            {
                event_loop.exit();

                tray_icon.take();
            }
        });
    }
}
