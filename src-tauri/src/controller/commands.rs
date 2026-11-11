use std::sync::Arc;
use std::thread;

use tauri::{AppHandle, Emitter, State};

use super::overlay::{self, POPUP_LABEL};
use super::suggestion_session::{SuggestionRequest, SuggestionSession};
use crate::model::error::PennaResult;
use crate::model::popup_state::PopupState;
use crate::service::correction_service::CorrectionService;

const POPUP_STATE_EVENT: &str = "popup-state";

#[tauri::command]
pub fn open_suggestion(
    app: AppHandle,
    session: State<Arc<SuggestionSession>>,
    service: State<Arc<CorrectionService>>,
) -> PennaResult<()> {
    let Some(request) = session.begin_suggestion() else {
        tracing::warn!("the icon was clicked without a remembered selection");
        return Ok(());
    };
    overlay::hide_icon(&app)?;
    overlay::show_popup(&app, request.snapshot.anchor)?;
    emit_popup_state(
        &app,
        &PopupState::Loading {
            original: request.snapshot.text.clone(),
        },
    )?;
    let session = session.inner().clone();
    let service = service.inner().clone();
    thread::spawn(move || deliver_correction(&app, &session, &service, &request));
    Ok(())
}

#[tauri::command]
pub fn accept_suggestion(
    app: AppHandle,
    session: State<Arc<SuggestionSession>>,
    service: State<Arc<CorrectionService>>,
) -> PennaResult<()> {
    let Some(replacement) = session.take_replacement() else {
        tracing::warn!("accept was requested without a correction");
        return Ok(());
    };
    overlay::hide_popup(&app)?;
    let service = service.inner().clone();
    thread::spawn(
        move || match service.apply(replacement.process_id, &replacement.text) {
            Ok(()) => tracing::info!("selection replaced with the correction"),
            Err(error) => tracing::error!(%error, "could not replace the selection"),
        },
    );
    Ok(())
}

#[tauri::command]
pub fn dismiss_suggestion(
    app: AppHandle,
    session: State<Arc<SuggestionSession>>,
) -> PennaResult<()> {
    session.discard_suggestion();
    overlay::hide_popup(&app)
}

fn deliver_correction(
    app: &AppHandle,
    session: &SuggestionSession,
    service: &CorrectionService,
    request: &SuggestionRequest,
) {
    let original = &request.snapshot.text;
    let state = match service.correct(original) {
        Ok(corrected) => {
            let state = PopupState::from_correction(original, &corrected);
            if matches!(state, PopupState::Ready { .. })
                && !session.store_correction(request.generation, &corrected)
            {
                return;
            }
            state
        }
        Err(error) => {
            tracing::error!(%error, "the agent could not correct the selection");
            PopupState::Failed {
                message: error.to_string(),
            }
        }
    };
    if !session.is_current(request.generation) {
        return;
    }
    if let Err(error) = emit_popup_state(app, &state) {
        tracing::error!(%error, "could not update the popup");
    }
}

fn emit_popup_state(app: &AppHandle, state: &PopupState) -> PennaResult<()> {
    app.emit_to(POPUP_LABEL, POPUP_STATE_EVENT, state)?;
    Ok(())
}
