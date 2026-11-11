use std::sync::Mutex;

use crate::model::selection::SelectionSnapshot;

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
}

/// Holds the selection the icon is shown for and the correction awaiting acceptance.
/// Each new or discarded selection bumps the generation so a late agent answer is ignored.
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
    }

    pub fn forget_selection(&self) {
        let mut data = self.locked();
        data.generation += 1;
        data.snapshot = None;
        data.correction = None;
    }

    pub fn begin_suggestion(&self) -> Option<SuggestionRequest> {
        let mut data = self.locked();
        let snapshot = data.snapshot.clone()?;
        data.generation += 1;
        data.correction = None;
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
        Some(Replacement { process_id, text })
    }

    pub fn discard_suggestion(&self) {
        let mut data = self.locked();
        data.generation += 1;
        data.correction = None;
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
}
