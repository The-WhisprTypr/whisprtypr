pub mod keyboard;

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
#[cfg(target_os = "windows")]
use std::time::Duration;

pub struct TextInjector {
    enigo: Enigo,
    clipboard: Option<arboard::Clipboard>,
}

#[cfg(target_os = "macos")]
unsafe impl Send for TextInjector {}
#[cfg(target_os = "macos")]
unsafe impl Sync for TextInjector {}

impl TextInjector {
    pub fn new() -> Result<Self, String> {
        let settings = Settings::default();

        let enigo =
            Enigo::new(&settings).map_err(|e| format!("Failed to initialize Enigo: {}", e))?;

        let clipboard = arboard::Clipboard::new().ok();

        Ok(Self { enigo, clipboard })
    }

    pub fn inject_text(&mut self, text: &str) -> Result<(), String> {
        if text.is_empty() {
            return Ok(());
        }

        if let Some(ref mut cb) = self.clipboard {
            let _previous = cb.get_text().ok();

            if cb.set_text(text).is_ok() {
                #[cfg(target_os = "windows")]
                std::thread::sleep(Duration::from_micros(100));

                keyboard::paste(&mut self.enigo)?;

                return Ok(());
            }
        }

        self.enigo
            .text(text)
            .map_err(|e| format!("Failed to inject text: {}", e))?;

        Ok(())
    }

    pub fn execute_shortcut(&mut self, shortcut: &str) -> Result<(), String> {
        keyboard::shortcut(&mut self.enigo, shortcut)
    }
}
