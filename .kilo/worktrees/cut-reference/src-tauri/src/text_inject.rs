use enigo::{Direction, Enigo, Key, Keyboard, Settings};
#[cfg(target_os = "windows")]
use std::time::Duration;

pub struct TextInjector {
    enigo: Enigo,
    clipboard: Option<arboard::Clipboard>,
}

// Safety: TextInjector must be Send + Sync for Tauri state management.
// On macOS, Enigo uses thread-local unsafe pointers (CGEventSource), but since we:
// 1. Only create one instance per app lifetime
// 2. Always access it through a Mutex (serialized)
// 3. Never share raw Enigo pointers across threads
// It's safe to mark as Send + Sync.
#[cfg(target_os = "macos")]
unsafe impl Send for TextInjector {}
#[cfg(target_os = "macos")]
unsafe impl Sync for TextInjector {}

impl TextInjector {
    pub fn new() -> Result<Self, String> {
        let settings = Settings::default();

        let enigo =
            Enigo::new(&settings).map_err(|e| format!("Failed to initialize Enigo: {}", e))?;

        // Initialize clipboard - critical for fast text injection
        let clipboard = arboard::Clipboard::new().ok();

        Ok(Self { enigo, clipboard })
    }

    pub fn inject_text(&mut self, text: &str) -> Result<(), String> {
        if text.is_empty() {
            return Ok(());
        }

        // ALWAYS use clipboard paste - it's significantly faster than keystroke simulation
        // Clipboard paste is instant regardless of text length
        // Direct typing can be 10-100x slower for longer text
        if let Some(ref mut cb) = self.clipboard {
            // Save current clipboard content to restore later (optional, for user convenience)
            let _previous = cb.get_text().ok();

            if cb.set_text(text).is_ok() {
                // Small delay to ensure clipboard is ready.
                #[cfg(target_os = "windows")]
                std::thread::sleep(Duration::from_micros(100));

                // Execute Paste shortcut
                self.execute_paste()?;

                // Optional: Restore previous clipboard after a brief delay
                // This is commented out as it may interfere with user workflow
                // if let Some(prev) = _previous {
                //     std::thread::sleep(Duration::from_millis(50));
                //     let _ = cb.set_text(&prev);
                // }

                return Ok(());
            }
        }

        // Fallback to direct typing only if clipboard fails
        self.enigo
            .text(text)
            .map_err(|e| format!("Failed to inject text: {}", e))?;

        Ok(())
    }

    /// Optimized paste operation for each platform
    fn execute_paste(&mut self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            // macOS: Cmd+V
            self.enigo
                .key(Key::Meta, Direction::Press)
                .map_err(|e| e.to_string())?;
            self.enigo
                .key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| e.to_string())?;
            self.enigo
                .key(Key::Meta, Direction::Release)
                .map_err(|e| e.to_string())?;
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: Ctrl+V with minimal delay
            self.enigo
                .key(Key::Control, Direction::Press)
                .map_err(|e| e.to_string())?;
            self.enigo
                .key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| e.to_string())?;
            self.enigo
                .key(Key::Control, Direction::Release)
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Execute a keyboard shortcut
    pub fn execute_shortcut(&mut self, shortcut: &str) -> Result<(), String> {
        match shortcut {
            "undo" => {
                // Ctrl+Z (or Cmd+Z on macOS)
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Meta, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('z'), Direction::Click).ok();
                    self.enigo.key(Key::Meta, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.enigo.key(Key::Control, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('z'), Direction::Click).ok();
                    self.enigo.key(Key::Control, Direction::Release).ok();
                }
            }
            "redo" => {
                // Ctrl+Y (Windows) or Cmd+Shift+Z (macOS)
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Meta, Direction::Press).ok();
                    self.enigo.key(Key::Shift, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('z'), Direction::Click).ok();
                    self.enigo.key(Key::Shift, Direction::Release).ok();
                    self.enigo.key(Key::Meta, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.enigo.key(Key::Control, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('y'), Direction::Click).ok();
                    self.enigo.key(Key::Control, Direction::Release).ok();
                }
            }
            "copy" => {
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Meta, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('c'), Direction::Click).ok();
                    self.enigo.key(Key::Meta, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.enigo.key(Key::Control, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('c'), Direction::Click).ok();
                    self.enigo.key(Key::Control, Direction::Release).ok();
                }
            }
            "cut" => {
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Meta, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('x'), Direction::Click).ok();
                    self.enigo.key(Key::Meta, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.enigo.key(Key::Control, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('x'), Direction::Click).ok();
                    self.enigo.key(Key::Control, Direction::Release).ok();
                }
            }
            "paste" => {
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Meta, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('v'), Direction::Click).ok();
                    self.enigo.key(Key::Meta, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.enigo.key(Key::Control, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('v'), Direction::Click).ok();
                    self.enigo.key(Key::Control, Direction::Release).ok();
                }
            }
            "select_all" => {
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Meta, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('a'), Direction::Click).ok();
                    self.enigo.key(Key::Meta, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.enigo.key(Key::Control, Direction::Press).ok();
                    self.enigo.key(Key::Unicode('a'), Direction::Click).ok();
                    self.enigo.key(Key::Control, Direction::Release).ok();
                }
            }
            "backspace_word" | "delete_word" => {
                // Ctrl+Backspace (delete word) or just multiple backspaces
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Alt, Direction::Press).ok();
                    self.enigo.key(Key::Backspace, Direction::Click).ok();
                    self.enigo.key(Key::Alt, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.enigo.key(Key::Control, Direction::Press).ok();
                    self.enigo.key(Key::Backspace, Direction::Click).ok();
                    self.enigo.key(Key::Control, Direction::Release).ok();
                }
            }
            "backspace" => {
                self.enigo.key(Key::Backspace, Direction::Click).ok();
            }
            "delete" => {
                self.enigo.key(Key::Delete, Direction::Click).ok();
            }
            "delete_line" => {
                // Select entire line then delete: Home, Shift+End, Delete
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Meta, Direction::Press).ok();
                    self.enigo.key(Key::Backspace, Direction::Click).ok();
                    self.enigo.key(Key::Meta, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    // Optimized for Windows: minimal delay between operations
                    // Go to start of line
                    self.enigo.key(Key::Home, Direction::Click).ok();
                    // Select to end (no delay - operations are queued)
                    self.enigo.key(Key::Shift, Direction::Press).ok();
                    self.enigo.key(Key::End, Direction::Click).ok();
                    self.enigo.key(Key::Shift, Direction::Release).ok();
                    // Delete (no delay - keyboard buffer handles sequencing)
                    self.enigo.key(Key::Backspace, Direction::Click).ok();
                }
            }
            "enter" => {
                self.enigo.key(Key::Return, Direction::Click).ok();
            }
            "tab" => {
                self.enigo.key(Key::Tab, Direction::Click).ok();
            }
            "escape" => {
                self.enigo.key(Key::Escape, Direction::Click).ok();
            }
            "page_up" => {
                self.enigo.key(Key::PageUp, Direction::Click).ok();
            }
            "page_down" => {
                self.enigo.key(Key::PageDown, Direction::Click).ok();
            }
            "left" => {
                self.enigo.key(Key::LeftArrow, Direction::Click).ok();
            }
            "right" => {
                self.enigo.key(Key::RightArrow, Direction::Click).ok();
            }
            "up" => {
                self.enigo.key(Key::UpArrow, Direction::Click).ok();
            }
            "down" => {
                self.enigo.key(Key::DownArrow, Direction::Click).ok();
            }
            "home" => {
                self.enigo.key(Key::Home, Direction::Click).ok();
            }
            "end" => {
                self.enigo.key(Key::End, Direction::Click).ok();
            }
            "word_left" => {
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Alt, Direction::Press).ok();
                    self.enigo.key(Key::LeftArrow, Direction::Click).ok();
                    self.enigo.key(Key::Alt, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.enigo.key(Key::Control, Direction::Press).ok();
                    self.enigo.key(Key::LeftArrow, Direction::Click).ok();
                    self.enigo.key(Key::Control, Direction::Release).ok();
                }
            }
            "word_right" => {
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Alt, Direction::Press).ok();
                    self.enigo.key(Key::RightArrow, Direction::Click).ok();
                    self.enigo.key(Key::Alt, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.enigo.key(Key::Control, Direction::Press).ok();
                    self.enigo.key(Key::RightArrow, Direction::Click).ok();
                    self.enigo.key(Key::Control, Direction::Release).ok();
                }
            }
            "select_left" => {
                self.enigo.key(Key::Shift, Direction::Press).ok();
                self.enigo.key(Key::LeftArrow, Direction::Click).ok();
                self.enigo.key(Key::Shift, Direction::Release).ok();
            }
            "select_right" => {
                self.enigo.key(Key::Shift, Direction::Press).ok();
                self.enigo.key(Key::RightArrow, Direction::Click).ok();
                self.enigo.key(Key::Shift, Direction::Release).ok();
            }
            "select_up" => {
                self.enigo.key(Key::Shift, Direction::Press).ok();
                self.enigo.key(Key::UpArrow, Direction::Click).ok();
                self.enigo.key(Key::Shift, Direction::Release).ok();
            }
            "select_down" => {
                self.enigo.key(Key::Shift, Direction::Press).ok();
                self.enigo.key(Key::DownArrow, Direction::Click).ok();
                self.enigo.key(Key::Shift, Direction::Release).ok();
            }
            "select_word_left" => {
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Alt, Direction::Press).ok();
                    self.enigo.key(Key::Shift, Direction::Press).ok();
                    self.enigo.key(Key::LeftArrow, Direction::Click).ok();
                    self.enigo.key(Key::Shift, Direction::Release).ok();
                    self.enigo.key(Key::Alt, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.enigo.key(Key::Control, Direction::Press).ok();
                    self.enigo.key(Key::Shift, Direction::Press).ok();
                    self.enigo.key(Key::LeftArrow, Direction::Click).ok();
                    self.enigo.key(Key::Shift, Direction::Release).ok();
                    self.enigo.key(Key::Control, Direction::Release).ok();
                }
            }
            "select_word_right" => {
                #[cfg(target_os = "macos")]
                {
                    self.enigo.key(Key::Alt, Direction::Press).ok();
                    self.enigo.key(Key::Shift, Direction::Press).ok();
                    self.enigo.key(Key::RightArrow, Direction::Click).ok();
                    self.enigo.key(Key::Shift, Direction::Release).ok();
                    self.enigo.key(Key::Alt, Direction::Release).ok();
                }
                #[cfg(not(target_os = "macos"))]
                {
                    self.enigo.key(Key::Control, Direction::Press).ok();
                    self.enigo.key(Key::Shift, Direction::Press).ok();
                    self.enigo.key(Key::RightArrow, Direction::Click).ok();
                    self.enigo.key(Key::Shift, Direction::Release).ok();
                    self.enigo.key(Key::Control, Direction::Release).ok();
                }
            }
            "select_to_start" => {
                self.enigo.key(Key::Shift, Direction::Press).ok();
                self.enigo.key(Key::Home, Direction::Click).ok();
                self.enigo.key(Key::Shift, Direction::Release).ok();
            }
            "select_to_end" => {
                self.enigo.key(Key::Shift, Direction::Press).ok();
                self.enigo.key(Key::End, Direction::Click).ok();
                self.enigo.key(Key::Shift, Direction::Release).ok();
            }
            _ => {
                return Err(format!("Unknown shortcut: {}", shortcut));
            }
        }

        // No delay needed after shortcut - execution is immediate

        Ok(())
    }
}

// These functions are deprecated - use the state-managed TextInjector instead
// (kept for backwards compatibility but not used in the new optimized path)
#[allow(dead_code)]
pub fn inject_text_once(text: &str) -> Result<(), String> {
    let mut injector = TextInjector::new()?;
    injector.inject_text(text)
}

#[allow(dead_code)]
pub fn execute_shortcut(shortcut: &str) -> Result<(), String> {
    let mut injector = TextInjector::new()?;
    injector.execute_shortcut(shortcut)
}
