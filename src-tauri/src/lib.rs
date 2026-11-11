mod controller;
mod gateway;
mod model;
pub use model::error::PennaError;
mod service;

use std::sync::Arc;

use gateway::agent_runner::CliAgentRunner;
use gateway::selection_gateway::ClipboardSelectionGateway;
use model::error::PennaResult;
use model::settings::Settings;
use service::rewrite_service::RewriteService;
use tauri::{App, Manager};

pub fn run() -> PennaResult<()> {
    tracing_subscriber::fmt().init();
    let settings = Settings::default();
    tauri::Builder::default()
        .plugin(controller::hotkey::plugin(settings.shortcut)?)
        .setup(move |app| {
            hide_dock_icon(app);
            app.manage(build_rewrite_service(app, &settings));
            controller::tray::build(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}

fn build_rewrite_service(app: &App, settings: &Settings) -> Arc<RewriteService> {
    let selection = Arc::new(ClipboardSelectionGateway::new(app.handle().clone()));
    let agent = Arc::new(CliAgentRunner::new(
        settings.agent.clone(),
        settings.instruction.clone(),
    ));
    Arc::new(RewriteService::new(selection, agent))
}

#[cfg(target_os = "macos")]
fn hide_dock_icon(app: &mut App) {
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
}

#[cfg(not(target_os = "macos"))]
fn hide_dock_icon(_app: &mut App) {}
