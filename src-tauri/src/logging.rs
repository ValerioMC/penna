use std::fs::{self, OpenOptions};
use std::sync::Mutex;

use tauri::{AppHandle, Manager};

use crate::model::error::{PennaError, PennaResult};

pub fn init(app: &AppHandle) -> PennaResult<()> {
    let directory = app.path().app_log_dir()?;
    fs::create_dir_all(&directory)?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("penna.log"))?;
    tracing_subscriber::fmt()
        .with_writer(Mutex::new(file))
        .with_ansi(false)
        .try_init()
        .map_err(|error| PennaError::Logging(error.to_string()))
}
