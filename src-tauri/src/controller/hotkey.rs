use std::sync::Arc;

use tauri::plugin::TauriPlugin;
use tauri::{Manager, Wry};
use tauri_plugin_global_shortcut::{Builder, Shortcut, ShortcutState};

use super::rewrite_selection_in_background;
use crate::model::error::PennaResult;
use crate::service::rewrite_service::RewriteService;

pub fn plugin(shortcut: Shortcut) -> PennaResult<TauriPlugin<Wry>> {
    let plugin = Builder::new()
        .with_shortcut(shortcut)?
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Released {
                let service = app.state::<Arc<RewriteService>>().inner().clone();
                rewrite_selection_in_background(service);
            }
        })
        .build();
    Ok(plugin)
}
