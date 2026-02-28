use std::{error::Error, sync::Arc};

use sayuri::sync::Mutex;

#[expect(unused)]
#[derive(Default, Clone)]
pub struct Notification {
    /// Filled by default with executable name.
    appname: String,

    /// Single line to summarize the content.
    summary: String,

    /// Subtitle for macOS
    subtitle: Option<String>,

    /// Multiple lines possible, may support simple markup,
    /// check out `get_capabilities()` -> `body-markup` and `body-hyperlinks`.
    body: String,

    buttons: Vec<(String, String)>,

    #[allow(clippy::type_complexity)]
    on_activated: Option<
        Arc<Mutex<Box<dyn FnMut(Option<String>) -> Result<(), Box<dyn Error>> + Send + 'static>>>,
    >,
}

impl Notification {
    pub fn new(appname: &str) -> Self {
        Self {
            appname: appname.to_owned(),
            ..Default::default()
        }
    }

    pub fn summary(&mut self, summary: &str) -> &mut Self {
        self.summary = summary.to_owned();
        self
    }

    pub fn body(&mut self, body: &str) -> &mut Self {
        self.body = body.to_owned();
        self
    }

    pub fn add_button(&mut self, content: &str, action: &str) -> &mut Self {
        self.buttons.push((content.to_owned(), action.to_owned()));
        self
    }

    pub fn on_activated<F>(&mut self, f: F) -> &mut Self
    where
        F: FnMut(Option<String>) -> Result<(), Box<dyn Error>> + Send + 'static,
    {
        self.on_activated = Some(Arc::new(Mutex::new(Box::new(f))));
        self
    }

    pub fn show(&mut self) {
        #[cfg(not(windows))]
        {
            let mut n = notify_rust::Notification::new();
            _ = n
                .summary(&self.summary)
                .subtitle(&self.subtitle)
                .body(&self.body)
                .appname(&self.appname)
                .show();
        }

        #[cfg(windows)]
        {
            let mut t = tauri_winrt_notification::Toast::new(crate::APP_ID)
                .title(&self.summary)
                .text1(self.subtitle.as_deref().unwrap_or(""))
                .text2(&self.body);

            for (content, action) in &self.buttons {
                t = t.add_button(content, action);
            }

            if let Some(on_activated) = self.on_activated.clone() {
                t = t.on_activated(move |action| {
                    let mut lock = on_activated.lock();
                    _ = lock(action);

                    Ok(())
                });
            }

            _ = t.show();
        }
    }
}
