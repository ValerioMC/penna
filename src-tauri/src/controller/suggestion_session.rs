use std::sync::Mutex;

use crate::model::selection::SelectionSnapshot;
use crate::service::selection_tracker::SelectionChange;

pub struct Replacement {
    pub process_id: i32,
    pub text: String,
}

pub struct SuggestionRequest {
    pub generation: u64,
    pub snapshot: SelectionSnapshot,
}

#[derive(Default)]
struct SessionData {
    generation: u64,
    snapshot: Option<SelectionSnapshot>,
    correction: Option<String>,
    open_suggestion_text: Option<String>,
}

/// Holds the selection the icon is shown for and the correction awaiting acceptance.
/// Each new or discarded selection bumps the generation so a late agent answer is ignored.
/// A suggestion stays open until dismissed, accepted or replaced by a different selection.
#[derive(Default)]
pub struct SuggestionSession {
    data: Mutex<SessionData>,
}

impl SuggestionSession {
    fn locked(&self) -> std::sync::MutexGuard<'_, SessionData> {
        self.data
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn remember_selection(&self, snapshot: SelectionSnapshot) {
        let mut data = self.locked();
        data.generation += 1;
        data.snapshot = Some(snapshot);
        data.correction = None;
        data.open_suggestion_text = None;
    }

    pub fn forget_selection(&self) {
        let mut data = self.locked();
        data.generation += 1;
        data.snapshot = None;
        data.correction = None;
        data.open_suggestion_text = None;
    }

    pub fn keeps_suggestion_open(&self, change: &SelectionChange) -> bool {
        let data = self.locked();
        let Some(open_text) = data.open_suggestion_text.as_deref() else {
            return false;
        };
        match change {
            SelectionChange::Cleared => true,
            SelectionChange::Appeared(snapshot) => snapshot.text == open_text,
            SelectionChange::Unchanged => false,
        }
    }

    pub fn begin_suggestion(&self) -> Option<SuggestionRequest> {
        let mut data = self.locked();
        let snapshot = data.snapshot.clone()?;
        data.generation += 1;
        data.correction = None;
        data.open_suggestion_text = Some(snapshot.text.clone());
        Some(SuggestionRequest {
            generation: data.generation,
            snapshot,
        })
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.locked().generation == generation
    }

    pub fn store_correction(&self, generation: u64, corrected: &str) -> bool {
        let mut data = self.locked();
        if data.generation != generation {
            return false;
        }
        data.correction = Some(corrected.to_string());
        true
    }

    pub fn take_replacement(&self) -> Option<Replacement> {
        let mut data = self.locked();
        let process_id = data.snapshot.as_ref()?.process_id;
        let text = data.correction.take()?;
        data.open_suggestion_text = None;
        Some(Replacement { process_id, text })
    }

    pub fn discard_suggestion(&self) {
        let mut data = self.locked();
        data.generation += 1;
        data.correction = None;
        data.open_suggestion_text = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::selection::ScreenPoint;

    fn snapshot() -> SelectionSnapshot {
        SelectionSnapshot {
            process_id: 7,
            text: "i has a dog".to_string(),
            anchor: ScreenPoint { x: 1.0, y: 2.0 },
        }
    }

    #[test]
    fn there_is_nothing_to_suggest_without_a_selection() {
        assert!(SuggestionSession::default().begin_suggestion().is_none());
    }

    #[test]
    fn a_stored_correction_is_taken_once() {
        let session = SuggestionSession::default();
        session.remember_selection(snapshot());
        let request = session.begin_suggestion().unwrap();
        assert!(session.store_correction(request.generation, "I have a dog"));
        let replacement = session.take_replacement().unwrap();
        assert_eq!(replacement.text, "I have a dog");
        assert_eq!(replacement.process_id, 7);
        assert!(session.take_replacement().is_none());
    }

    #[test]
    fn a_late_answer_is_dropped_after_the_selection_changes() {
        let session = SuggestionSession::default();
        session.remember_selection(snapshot());
        let request = session.begin_suggestion().unwrap();
        session.forget_selection();
        assert!(!session.store_correction(request.generation, "I have a dog"));
        assert!(session.take_replacement().is_none());
    }

    #[test]
    fn a_dismissed_suggestion_drops_its_late_answer() {
        let session = SuggestionSession::default();
        session.remember_selection(snapshot());
        let request = session.begin_suggestion().unwrap();
        session.discard_suggestion();
        assert!(!session.is_current(request.generation));
    }

    #[test]
    fn an_open_suggestion_survives_the_selection_being_lost_and_found_again() {
        let session = SuggestionSession::default();
        session.remember_selection(snapshot());
        session.begin_suggestion().unwrap();
        assert!(session.keeps_suggestion_open(&SelectionChange::Cleared));
        assert!(session.keeps_suggestion_open(&SelectionChange::Appeared(snapshot())));
    }

    #[test]
    fn a_different_selection_replaces_an_open_suggestion() {
        let session = SuggestionSession::default();
        session.remember_selection(snapshot());
        session.begin_suggestion().unwrap();
        let other = SelectionSnapshot {
            text: "another text".to_string(),
            ..snapshot()
        };
        assert!(!session.keeps_suggestion_open(&SelectionChange::Appeared(other)));
    }

    #[test]
    fn a_closed_suggestion_follows_the_selection() {
        let session = SuggestionSession::default();
        session.remember_selection(snapshot());
        assert!(!session.keeps_suggestion_open(&SelectionChange::Cleared));
        session.begin_suggestion().unwrap();
        session.discard_suggestion();
        assert!(!session.keeps_suggestion_open(&SelectionChange::Cleared));
    }
}
