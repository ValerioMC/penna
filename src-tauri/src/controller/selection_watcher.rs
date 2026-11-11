use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use super::overlay;
use super::suggestion_session::SuggestionSession;
use crate::gateway::selection_reader::SelectionReader;
use crate::model::error::{PennaError, PennaResult};
use crate::service::selection_tracker::{SelectionChange, SelectionTracker};

const POLL_INTERVAL: Duration = Duration::from_millis(200);

pub fn spawn(app: AppHandle, reader: impl SelectionReader + Send + 'static) {
    thread::spawn(move || watch(&app, reader));
}

fn watch(app: &AppHandle, mut reader: impl SelectionReader) {
    let session = app.state::<Arc<SuggestionSession>>().inner().clone();
    let mut tracker = SelectionTracker::default();
    let mut last_failure: Option<String> = None;
    loop {
        thread::sleep(POLL_INTERVAL);
        let outcome = reader
            .read_selection()
            .and_then(|reading| apply_change(app, &session, tracker.observe(reading)));
        match outcome {
            Ok(()) => report_recovery(&mut last_failure),
            Err(error) => report_new_failure(&mut last_failure, &error),
        }
    }
}

fn apply_change(
    app: &AppHandle,
    session: &SuggestionSession,
    change: SelectionChange,
) -> PennaResult<()> {
    match change {
        SelectionChange::Appeared(snapshot) => {
            tracing::info!(
                characters = snapshot.text.chars().count(),
                "selection detected"
            );
            let anchor = snapshot.anchor;
            session.remember_selection(snapshot);
            overlay::hide_popup(app)?;
            overlay::show_icon(app, anchor)
        }
        SelectionChange::Cleared => {
            session.forget_selection();
            overlay::hide_icon(app)?;
            overlay::hide_popup(app)
        }
        SelectionChange::Unchanged => Ok(()),
    }
}

fn report_new_failure(last_failure: &mut Option<String>, error: &PennaError) {
    let message = error.to_string();
    if last_failure.as_deref() != Some(message.as_str()) {
        tracing::warn!(%error, "selection watching is failing");
        *last_failure = Some(message);
    }
}

fn report_recovery(last_failure: &mut Option<String>) {
    if last_failure.take().is_some() {
        tracing::info!("selection watching works again");
    }
}
