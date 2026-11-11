pub mod hotkey;
pub mod tray;

use std::sync::Arc;
use std::thread;

use crate::service::rewrite_service::RewriteService;

pub fn rewrite_selection_in_background(service: Arc<RewriteService>) {
    thread::spawn(move || match service.rewrite_selection() {
        Ok(()) => tracing::info!("selection rewritten"),
        Err(error) => tracing::error!(%error, "could not rewrite the selection"),
    });
}
