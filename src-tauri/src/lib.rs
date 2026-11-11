mod controller;
mod gateway;
mod logging;
mod model;
mod service;

use std::sync::Arc;

use controller::commands::{accept_suggestion, dismiss_suggestion, open_suggestion};
use controller::suggestion_session::SuggestionSession;
use gateway::agent_runner::CliAgentRunner;
use gateway::selection_replacer::ClipboardSelectionReplacer;
use model::error::PennaResult;
use model::settings::Settings;
use service::correction_service::CorrectionService;
use tauri::{App, AppHandle, Manager};

pub use model::error::PennaError;

pub fn run() -> PennaResult<()> {
    let settings = Settings::default();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_suggestion,
            accept_suggestion,
            dismiss_suggestion
        ])
        .setup(move |app| {
            logging::init(app.handle())?;
            hide_dock_icon(app);
            app.manage(build_correction_service(app, &settings));
            app.manage(Arc::new(SuggestionSession::default()));
            controller::overlay::create_windows(app.handle())?;
            controller::tray::build(app.handle())?;
            start_selection_watching(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}

fn build_correction_service(app: &App, settings: &Settings) -> Arc<CorrectionService> {
    let agent = Arc::new(CliAgentRunner::new(
        settings.agent.clone(),
        settings.instruction.clone(),
    ));
    let replacer = Arc::new(ClipboardSelectionReplacer::new(app.handle().clone()));
    Arc::new(CorrectionService::new(agent, replacer))
}

#[cfg(target_os = "macos")]
fn start_selection_watching(app: &AppHandle) {
    let trusted = gateway::accessibility_selection_reader::request_permission_prompt();
    tracing::info!(trusted, "accessibility permission checked");
    controller::selection_watcher::spawn(
        app.clone(),
        gateway::accessibility_selection_reader::AccessibilitySelectionReader::default(),
    );
}

#[cfg(not(target_os = "macos"))]
fn start_selection_watching(_app: &AppHandle) {
    tracing::warn!("selection detection is only available on macOS");
}

#[cfg(target_os = "macos")]
fn hide_dock_icon(app: &mut App) {
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
}

#[cfg(not(target_os = "macos"))]
fn hide_dock_icon(_app: &mut App) {}
