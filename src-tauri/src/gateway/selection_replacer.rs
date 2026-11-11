use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tauri::AppHandle;

use crate::model::error::{PennaError, PennaResult};

const CLIPBOARD_SETTLE_DELAY: Duration = Duration::from_millis(150);
const ACTIVATION_DELAY: Duration = Duration::from_millis(250);
const MAIN_THREAD_TIMEOUT: Duration = Duration::from_secs(5);

pub trait SelectionReplacer: Send + Sync {
    fn replace_selection(&self, target_process_id: i32, text: &str) -> PennaResult<()>;
}

/// Replaces the selection of the focused app by pasting through the clipboard.
/// Keyboard simulation runs on the main thread because macOS requires it.
pub struct ClipboardSelectionReplacer {
    app: AppHandle,
}

impl ClipboardSelectionReplacer {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn run_on_main_thread<T, Job>(&self, job: Job) -> PennaResult<T>
    where
        T: Send + 'static,
        Job: FnOnce() -> PennaResult<T> + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        self.app.run_on_main_thread(move || {
            if sender.send(job()).is_err() {
                tracing::warn!("main thread job finished after its caller stopped waiting");
            }
        })?;
        receiver
            .recv_timeout(MAIN_THREAD_TIMEOUT)
            .map_err(|_| PennaError::MainThreadTimeout)?
    }
}

impl SelectionReplacer for ClipboardSelectionReplacer {
    fn replace_selection(&self, target_process_id: i32, text: &str) -> PennaResult<()> {
        let text = text.to_string();
        self.run_on_main_thread(move || {
            bring_to_front(target_process_id)?;
            thread::sleep(ACTIVATION_DELAY);
            paste_through_clipboard(&text)
        })
    }
}

#[cfg(target_os = "macos")]
fn bring_to_front(process_id: i32) -> PennaResult<()> {
    super::accessibility_selection_reader::bring_application_to_front(process_id)
}

#[cfg(not(target_os = "macos"))]
fn bring_to_front(_process_id: i32) -> PennaResult<()> {
    Ok(())
}

fn paste_through_clipboard(text: &str) -> PennaResult<()> {
    let mut clipboard = Clipboard::new()?;
    let previous_text = read_text_if_present(&mut clipboard)?;
    clipboard.set_text(text)?;
    press_command_with(Key::Unicode('v'))?;
    thread::sleep(CLIPBOARD_SETTLE_DELAY);
    restore_text(&mut clipboard, previous_text)
}

fn read_text_if_present(clipboard: &mut Clipboard) -> PennaResult<Option<String>> {
    match clipboard.get_text() {
        Ok(text) => Ok(Some(text)),
        Err(arboard::Error::ContentNotAvailable) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn restore_text(clipboard: &mut Clipboard, previous_text: Option<String>) -> PennaResult<()> {
    match previous_text {
        Some(text) => Ok(clipboard.set_text(text)?),
        None => Ok(clipboard.clear()?),
    }
}

#[cfg(target_os = "macos")]
fn command_modifier() -> Key {
    Key::Meta
}

#[cfg(not(target_os = "macos"))]
fn command_modifier() -> Key {
    Key::Control
}

fn press_command_with(key: Key) -> PennaResult<()> {
    let mut keyboard = Enigo::new(&Settings::default())?;
    release_shortcut_modifiers(&mut keyboard)?;
    keyboard.key(command_modifier(), Direction::Press)?;
    keyboard.key(key, Direction::Click)?;
    keyboard.key(command_modifier(), Direction::Release)?;
    Ok(())
}

fn release_shortcut_modifiers(keyboard: &mut Enigo) -> PennaResult<()> {
    for modifier in [Key::Control, Key::Alt, Key::Shift] {
        keyboard.key(modifier, Direction::Release)?;
    }
    Ok(())
}
