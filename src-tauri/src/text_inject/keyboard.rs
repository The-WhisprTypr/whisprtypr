use enigo::{Direction, Enigo, Key, Keyboard};

pub fn paste(enigo: &mut Enigo) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        enigo
            .key(Key::Meta, Direction::Press)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Meta, Direction::Release)
            .map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn shortcut(enigo: &mut Enigo, shortcut: &str) -> Result<(), String> {
    match shortcut {
        "undo" => {
            #[cfg(target_os = "macos")]
            {
                enigo.key(Key::Meta, Direction::Press).ok();
                enigo.key(Key::Unicode('z'), Direction::Click).ok();
                enigo.key(Key::Meta, Direction::Release).ok();
            }
            #[cfg(not(target_os = "macos"))]
            {
                enigo.key(Key::Control, Direction::Press).ok();
                enigo.key(Key::Unicode('z'), Direction::Click).ok();
                enigo.key(Key::Control, Direction::Release).ok();
            }
        }
        "redo" => {
            #[cfg(target_os = "macos")]
            {
                enigo.key(Key::Meta, Direction::Press).ok();
                enigo.key(Key::Shift, Direction::Press).ok();
                enigo.key(Key::Unicode('z'), Direction::Click).ok();
                enigo.key(Key::Shift, Direction::Release).ok();
                enigo.key(Key::Meta, Direction::Release).ok();
            }
            #[cfg(not(target_os = "macos"))]
            {
                enigo.key(Key::Control, Direction::Press).ok();
                enigo.key(Key::Unicode('y'), Direction::Click).ok();
                enigo.key(Key::Control, Direction::Release).ok();
            }
        }
        "copy" => {
            #[cfg(target_os = "macos")]
            {
                enigo.key(Key::Meta, Direction::Press).ok();
                enigo.key(Key::Unicode('c'), Direction::Click).ok();
                enigo.key(Key::Meta, Direction::Release).ok();
            }
            #[cfg(not(target_os = "macos"))]
            {
                enigo.key(Key::Control, Direction::Press).ok();
                enigo.key(Key::Unicode('c'), Direction::Click).ok();
                enigo.key(Key::Control, Direction::Release).ok();
            }
        }
        "cut" => {
            #[cfg(target_os = "macos")]
            {
                enigo.key(Key::Meta, Direction::Press).ok();
                enigo.key(Key::Unicode('x'), Direction::Click).ok();
                enigo.key(Key::Meta, Direction::Release).ok();
            }
            #[cfg(not(target_os = "macos"))]
            {
                enigo.key(Key::Control, Direction::Press).ok();
                enigo.key(Key::Unicode('x'), Direction::Click).ok();
                enigo.key(Key::Control, Direction::Release).ok();
            }
        }
        "paste" => {
            return paste(enigo);
        }
        "select_all" => {
            #[cfg(target_os = "macos")]
            {
                enigo.key(Key::Meta, Direction::Press).ok();
                enigo.key(Key::Unicode('a'), Direction::Click).ok();
                enigo.key(Key::Meta, Direction::Release).ok();
            }
            #[cfg(not(target_os = "macos"))]
            {
                enigo.key(Key::Control, Direction::Press).ok();
                enigo.key(Key::Unicode('a'), Direction::Click).ok();
                enigo.key(Key::Control, Direction::Release).ok();
            }
        }
        "backspace_word" | "delete_word" => {
            #[cfg(target_os = "macos")]
            {
                enigo.key(Key::Alt, Direction::Press).ok();
                enigo.key(Key::Backspace, Direction::Click).ok();
                enigo.key(Key::Alt, Direction::Release).ok();
            }
            #[cfg(not(target_os = "macos"))]
            {
                enigo.key(Key::Control, Direction::Press).ok();
                enigo.key(Key::Backspace, Direction::Click).ok();
                enigo.key(Key::Control, Direction::Release).ok();
            }
        }
        "backspace" => {
            enigo.key(Key::Backspace, Direction::Click).ok();
        }
        "delete" => {
            enigo.key(Key::Delete, Direction::Click).ok();
        }
        "delete_line" => {
            enigo.key(Key::Home, Direction::Press).ok();
            #[cfg(target_os = "macos")]
            {
                enigo.key(Key::Shift, Direction::Press).ok();
                enigo.key(Key::End, Direction::Click).ok();
                enigo.key(Key::Shift, Direction::Release).ok();
            }
            #[cfg(not(target_os = "macos"))]
            {
                enigo.key(Key::Shift, Direction::Press).ok();
                enigo.key(Key::End, Direction::Click).ok();
                enigo.key(Key::Shift, Direction::Release).ok();
            }
            enigo.key(Key::Delete, Direction::Click).ok();
        }
        "enter" => {
            enigo.key(Key::Return, Direction::Click).ok();
        }
        "tab" => {
            enigo.key(Key::Tab, Direction::Click).ok();
        }
        "escape" => {
            enigo.key(Key::Escape, Direction::Click).ok();
        }
        "page_up" => {
            enigo.key(Key::PageUp, Direction::Click).ok();
        }
        "page_down" => {
            enigo.key(Key::PageDown, Direction::Click).ok();
        }
        "left" => {
            enigo.key(Key::LeftArrow, Direction::Click).ok();
        }
        "right" => {
            enigo.key(Key::RightArrow, Direction::Click).ok();
        }
        "up" => {
            enigo.key(Key::UpArrow, Direction::Click).ok();
        }
        "down" => {
            enigo.key(Key::DownArrow, Direction::Click).ok();
        }
        "home" => {
            enigo.key(Key::Home, Direction::Click).ok();
        }
        "end" => {
            enigo.key(Key::End, Direction::Click).ok();
        }
        "word_left" | "delete_word_left" => {
            enigo.key(Key::Control, Direction::Press).ok();
            enigo.key(Key::LeftArrow, Direction::Click).ok();
            enigo.key(Key::Control, Direction::Release).ok();
        }
        "word_right" | "delete_word_right" => {
            enigo.key(Key::Control, Direction::Press).ok();
            enigo.key(Key::RightArrow, Direction::Click).ok();
            enigo.key(Key::Control, Direction::Release).ok();
        }
        "select_left" => {
            enigo.key(Key::Shift, Direction::Press).ok();
            enigo.key(Key::LeftArrow, Direction::Click).ok();
            enigo.key(Key::Shift, Direction::Release).ok();
        }
        "select_right" => {
            enigo.key(Key::Shift, Direction::Press).ok();
            enigo.key(Key::RightArrow, Direction::Click).ok();
            enigo.key(Key::Shift, Direction::Release).ok();
        }
        "select_up" => {
            enigo.key(Key::Shift, Direction::Press).ok();
            enigo.key(Key::UpArrow, Direction::Click).ok();
            enigo.key(Key::Shift, Direction::Release).ok();
        }
        "select_down" => {
            enigo.key(Key::Shift, Direction::Press).ok();
            enigo.key(Key::DownArrow, Direction::Click).ok();
            enigo.key(Key::Shift, Direction::Release).ok();
        }
        "select_word_left" => {
            enigo.key(Key::Control, Direction::Press).ok();
            enigo.key(Key::Shift, Direction::Press).ok();
            enigo.key(Key::LeftArrow, Direction::Click).ok();
            enigo.key(Key::Control, Direction::Release).ok();
            enigo.key(Key::Shift, Direction::Release).ok();
        }
        "select_word_right" => {
            enigo.key(Key::Control, Direction::Press).ok();
            enigo.key(Key::Shift, Direction::Press).ok();
            enigo.key(Key::RightArrow, Direction::Click).ok();
            enigo.key(Key::Control, Direction::Release).ok();
            enigo.key(Key::Shift, Direction::Release).ok();
        }
        "select_to_start" => {
            enigo.key(Key::Shift, Direction::Press).ok();
            enigo.key(Key::Home, Direction::Click).ok();
            enigo.key(Key::Shift, Direction::Release).ok();
        }
        "select_to_end" => {
            enigo.key(Key::Shift, Direction::Press).ok();
            enigo.key(Key::End, Direction::Click).ok();
            enigo.key(Key::Shift, Direction::Release).ok();
        }
        _ => {
            return Err(format!("Unknown shortcut: {}", shortcut));
        }
    }

    Ok(())
}
