use crate::{CommandError, CommandResult};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "Whisprtypr";

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct HotkeyRegistration {
    hotkey: String,
    label: String,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct HotkeyEventPayload {
    label: String,
}

#[tauri::command]
pub fn get_app_data_dir(app: AppHandle) -> CommandResult<String> {
    let path = app.path().app_data_dir().map_err(|e: tauri::Error| {
        std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string())
    })?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_models_dir(app: AppHandle) -> CommandResult<String> {
    let path = app.path().app_data_dir().map_err(|e: tauri::Error| {
        std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string())
    })?;
    Ok(path.join("models").to_string_lossy().to_string())
}

#[tauri::command]
pub fn register_hotkey(
    app: AppHandle,
    registrations: Vec<HotkeyRegistration>,
) -> CommandResult<()> {
    if registrations.is_empty() {
        return Ok(());
    }

    println!("Registering {} hotkeys", registrations.len());

    if let Err(e) = app.global_shortcut().unregister_all() {
        println!("Warning: Failed to unregister existing shortcuts: {}", e);
    }

    for reg in registrations {
        let shortcut = parse_hotkey(&reg.hotkey)
            .map_err(|e| CommandError::Recording(format!("Invalid hotkey: {}", e)))?;
        let label = reg.label.clone();

        println!(
            "Registering hotkey: {} -> {:?} (label: {})",
            reg.hotkey, shortcut, label
        );

        let result = app
            .global_shortcut()
            .on_shortcut(shortcut, move |app, _shortcut, event| {
                println!("Shortcut event: {:?} for label: {}", event.state(), label);
                match event.state() {
                    ShortcutState::Pressed => {
                        let _ = app.emit(
                            "hotkey-pressed",
                            HotkeyEventPayload {
                                label: label.clone(),
                            },
                        );
                    }
                    ShortcutState::Released => {
                        let _ = app.emit(
                            "hotkey-released",
                            HotkeyEventPayload {
                                label: label.clone(),
                            },
                        );
                    }
                }
            });

        if let Err(e) = result {
            println!("Failed to register hotkey {}: {}", reg.hotkey, e);
            return Err(CommandError::Recording(format!(
                "Failed to register hotkey {}: {}",
                reg.hotkey, e
            )));
        }
    }

    println!("All hotkeys registered successfully");
    Ok(())
}

#[tauri::command]
pub fn unregister_hotkeys(app: AppHandle) -> CommandResult<()> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| CommandError::Recording(format!("Failed to unregister hotkeys: {}", e)))?;
    Ok(())
}

pub(crate) fn parse_hotkey(hotkey: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();

    if parts.is_empty() {
        return Err("Empty hotkey".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "super" | "meta" | "win" | "cmd" => modifiers |= Modifiers::SUPER,
            "space" => key_code = Some(Code::Space),
            "enter" | "return" => key_code = Some(Code::Enter),
            "tab" => key_code = Some(Code::Tab),
            "escape" | "esc" => key_code = Some(Code::Escape),
            "backspace" => key_code = Some(Code::Backspace),
            "delete" => key_code = Some(Code::Delete),
            "f1" => key_code = Some(Code::F1),
            "f2" => key_code = Some(Code::F2),
            "f3" => key_code = Some(Code::F3),
            "f4" => key_code = Some(Code::F4),
            "f5" => key_code = Some(Code::F5),
            "f6" => key_code = Some(Code::F6),
            "f7" => key_code = Some(Code::F7),
            "f8" => key_code = Some(Code::F8),
            "f9" => key_code = Some(Code::F9),
            "f10" => key_code = Some(Code::F10),
            "f11" => key_code = Some(Code::F11),
            "f12" => key_code = Some(Code::F12),
            s if s.len() == 1 => {
                let c = s.chars().next().unwrap().to_ascii_uppercase();
                key_code = match c {
                    'A' => Some(Code::KeyA),
                    'B' => Some(Code::KeyB),
                    'C' => Some(Code::KeyC),
                    'D' => Some(Code::KeyD),
                    'E' => Some(Code::KeyE),
                    'F' => Some(Code::KeyF),
                    'G' => Some(Code::KeyG),
                    'H' => Some(Code::KeyH),
                    'I' => Some(Code::KeyI),
                    'J' => Some(Code::KeyJ),
                    'K' => Some(Code::KeyK),
                    'L' => Some(Code::KeyL),
                    'M' => Some(Code::KeyM),
                    'N' => Some(Code::KeyN),
                    'O' => Some(Code::KeyO),
                    'P' => Some(Code::KeyP),
                    'Q' => Some(Code::KeyQ),
                    'R' => Some(Code::KeyR),
                    'S' => Some(Code::KeyS),
                    'T' => Some(Code::KeyT),
                    'U' => Some(Code::KeyU),
                    'V' => Some(Code::KeyV),
                    'W' => Some(Code::KeyW),
                    'X' => Some(Code::KeyX),
                    'Y' => Some(Code::KeyY),
                    'Z' => Some(Code::KeyZ),
                    '0' => Some(Code::Digit0),
                    '1' => Some(Code::Digit1),
                    '2' => Some(Code::Digit2),
                    '3' => Some(Code::Digit3),
                    '4' => Some(Code::Digit4),
                    '5' => Some(Code::Digit5),
                    '6' => Some(Code::Digit6),
                    '7' => Some(Code::Digit7),
                    '8' => Some(Code::Digit8),
                    '9' => Some(Code::Digit9),
                    _ => return Err(format!("Unknown key: {}", s)),
                };
            }
            _ => return Err(format!("Unknown key or modifier: {}", part)),
        }
    }

    let code = key_code.ok_or("No key specified in hotkey")?;

    Ok(Shortcut::new(Some(modifiers), code))
}

#[tauri::command]
pub fn get_app_version() -> String {
    APP_VERSION.to_string()
}

#[tauri::command]
pub fn get_app_name() -> String {
    APP_NAME.to_string()
}

pub(crate) fn ai_formatting_style_prompt(style: &str) -> &'static str {
    match style {
        "personal" => "You are a helpful dictation assistant. The user has spoken the following text which was transcribed from voice. Your job is to lightly clean up obvious transcription errors, punctuation, and filler words while preserving the speaker's casual, conversational tone and personality. Do not over-edit or change the speaker's voice. Return only the formatted text, with no preamble or explanation.",
        "clean" => "You are a professional dictation editor. The user has spoken the following text which was transcribed from voice. Your job is to produce clean, professional prose: fix grammar, remove filler words (um, uh, like, you know), add proper punctuation, ensure proper sentence structure, and create clean paragraph breaks. Return only the formatted text, with no preamble or explanation.",
        "writing" => "You are an editor helping someone turn spoken dictation into polished written content. Format the following transcribed text as a well-structured article or blog post. Use proper paragraphs, fix grammar and flow, and polish the prose to sound professional and engaging. Return only the formatted text, with no preamble or explanation.",
        "notes" => "You are a note-taking assistant. Extract the key points and important information from the following transcribed dictation. Format as concise bullet points or short phrases, capturing the essential information. Remove filler words and redundant phrasing. Return only the formatted notes, with no preamble or explanation.",
        "email" => "You are a professional email assistant. Format the following transcribed dictation as a polished business email. Add an appropriate greeting, structure the body with clear paragraphs, and include a professional sign-off. Ensure the tone is appropriate and professional. Return only the formatted email text, with no preamble or explanation.",
        "code" => "You are a coding assistant. The following text was transcribed from voice and contains spoken programming terms, code snippets, and technical instructions. Convert spoken descriptions of code into clean, properly formatted code. Apply appropriate casing (camelCase, PascalCase, snake_case), insert code symbols (brackets, braces, operators) that were spoken as words, and organize into logical blocks. Preserve any literal code. Return only the formatted code, with no preamble or explanation.",
        "social" => "You are a social media assistant. The following text was transcribed from casual voice dictation. Format it for social media: keep a conversational and engaging tone, use short paragraphs or sentences, add appropriate line breaks, and make it easy to read. Return only the formatted text, with no preamble or explanation.",
        "academic" => "You are an academic writing assistant. The following text was transcribed from voice. Rewrite it in a formal academic style: use precise language, proper sentence structure, formal tone, and structured paragraphs. Remove colloquialisms and filler words. Return only the formatted text, with no preamble or explanation.",
        "business" => "You are a business communication assistant. The following text was transcribed from voice. Format it as a professional business document: use clear, concise language, structured paragraphs, bullet points where appropriate, and a professional tone. Return only the formatted text, with no preamble or explanation.",
        "transcript" => "You are a transcription editor. The following text was transcribed from voice. Create a clean transcript-style output: preserve the original meaning and content, but fix obvious transcription errors, add proper punctuation, and organize into readable paragraphs. Do not change the tone or style of the original speech. Return only the formatted text, with no preamble or explanation.",
        "legal" => "You are a legal transcription assistant. The following text was transcribed from voice. Rewrite it in a formal legal style: use precise terminology, structured paragraphs, and proper legal phrasing. Maintain the original meaning while ensuring legal accuracy. Return only the formatted text, with no preamble or explanation.",
        "meeting" => "You are a meeting minutes assistant. The following text was transcribed from a meeting recording. Format it as professional meeting minutes: include a brief summary, list key discussion points, highlight decisions made, and extract action items with responsible parties and deadlines. Use clear headings and bullet points. Return only the formatted text, with no preamble or explanation.",
        "journaling" => "You are a journaling assistant. The following text was transcribed from a personal voice journal entry. Format it as a thoughtful, well-structured journal entry: preserve the personal, reflective tone, organize thoughts into coherent paragraphs, add proper punctuation, and maintain the authentic voice of the writer. Return only the formatted text, with no preamble or explanation.",
        _ => "You are a professional dictation editor. Clean up the following transcribed text: fix grammar, add proper punctuation, and create clean paragraph breaks. Return only the formatted text, with no preamble or explanation.",
    }
}
