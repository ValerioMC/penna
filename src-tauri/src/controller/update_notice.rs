use std::sync::Arc;
use std::thread;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::model::error::PennaResult;
use crate::model::version::Version;
use crate::service::update_service::UpdateService;

pub const UPDATE_LABEL: &str = "update";
const UPDATE_SIZE: (f64, f64) = (380.0, 170.0);

pub fn spawn_startup_check(app: AppHandle) {
    thread::spawn(move || check_for_update(&app));
}

pub fn close(app: &AppHandle) -> PennaResult<()> {
    if let Some(window) = app.get_webview_window(UPDATE_LABEL) {
        window.close()?;
    }
    Ok(())
}

fn check_for_update(app: &AppHandle) {
    let service = Arc::clone(app.state::<Arc<UpdateService>>().inner());
    let outcome = Version::parse(&app.package_info().version.to_string())
        .and_then(|installed| service.newer_version(installed));
    match outcome {
        Ok(Some(newer)) => announce(app, newer),
        Ok(None) => tracing::info!("Penna is up to date"),
        Err(error) => tracing::error!(%error, "could not check for a newer version"),
    }
}

fn announce(app: &AppHandle, newer: Version) {
    tracing::info!(%newer, "a newer version of Penna is available");
    let window_owner = app.clone();
    let scheduled = app.run_on_main_thread(move || {
        if let Err(error) = show_notice(&window_owner, newer) {
            tracing::error!(%error, "could not show the update notice");
        }
    });
    if let Err(error) = scheduled {
        tracing::error!(%error, "could not schedule the update notice");
    }
}

fn show_notice(app: &AppHandle, newer: Version) -> PennaResult<()> {
    let page = format!("update.html?version={newer}");
    WebviewWindowBuilder::new(app, UPDATE_LABEL, WebviewUrl::App(page.into()))
        .inner_size(UPDATE_SIZE.0, UPDATE_SIZE.1)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        .accept_first_mouse(true)
        .center()
        .build()?;
    Ok(())
}
