use crate::model::selection::{SelectionReading, SelectionSnapshot};

const UNREADABLE_READINGS_BEFORE_CLEARING: u32 = 5;

#[derive(Debug, PartialEq)]
pub enum SelectionChange {
    Appeared(SelectionSnapshot),
    Cleared,
    Unchanged,
}

/// Reports a selection only after two consecutive readings agree, so a drag in progress
/// does not flash the icon, and reports each stable selection once. Apps such as Teams
/// intermittently expose no focused element, so only a run of unreadable readings clears it.
#[derive(Default)]
pub struct SelectionTracker {
    candidate_text: Option<String>,
    reported_text: Option<String>,
    consecutive_unreadable_readings: u32,
}

impl SelectionTracker {
    pub fn observe(&mut self, reading: SelectionReading) -> SelectionChange {
        if !matches!(
            reading,
            SelectionReading::Unreadable | SelectionReading::Indeterminate
        ) {
            self.consecutive_unreadable_readings = 0;
        }
        match reading {
            SelectionReading::Indeterminate => SelectionChange::Unchanged,
            SelectionReading::Unreadable => self.observe_unreadable(),
            SelectionReading::NothingSelected => self.clear(),
            SelectionReading::Selected(snapshot) if snapshot.text.trim().is_empty() => self.clear(),
            SelectionReading::Selected(snapshot) => self.observe_selected(snapshot),
        }
    }

    fn observe_unreadable(&mut self) -> SelectionChange {
        self.consecutive_unreadable_readings += 1;
        if self.consecutive_unreadable_readings < UNREADABLE_READINGS_BEFORE_CLEARING {
            return SelectionChange::Unchanged;
        }
        self.clear()
    }

    fn clear(&mut self) -> SelectionChange {
        self.candidate_text = None;
        match self.reported_text.take() {
            Some(_) => SelectionChange::Cleared,
            None => SelectionChange::Unchanged,
        }
    }

    fn observe_selected(&mut self, snapshot: SelectionSnapshot) -> SelectionChange {
        if self.reported_text.as_deref() == Some(snapshot.text.as_str()) {
            return SelectionChange::Unchanged;
        }
        if self.candidate_text.as_deref() == Some(snapshot.text.as_str()) {
            self.reported_text = Some(snapshot.text.clone());
            return SelectionChange::Appeared(snapshot);
        }
        self.candidate_text = Some(snapshot.text);
        SelectionChange::Unchanged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::selection::ScreenPoint;

    fn selected(text: &str) -> SelectionReading {
        SelectionReading::Selected(SelectionSnapshot {
            process_id: 1,
            text: text.to_string(),
            anchor: ScreenPoint { x: 10.0, y: 20.0 },
        })
    }

    #[test]
    fn a_selection_appears_only_after_two_equal_readings() {
        let mut tracker = SelectionTracker::default();
        assert_eq!(
            tracker.observe(selected("hello")),
            SelectionChange::Unchanged
        );
        assert!(matches!(
            tracker.observe(selected("hello")),
            SelectionChange::Appeared(snapshot) if snapshot.text == "hello"
        ));
    }

    #[test]
    fn a_reported_selection_is_not_reported_again() {
        let mut tracker = SelectionTracker::default();
        tracker.observe(selected("hello"));
        tracker.observe(selected("hello"));
        assert_eq!(
            tracker.observe(selected("hello")),
            SelectionChange::Unchanged
        );
    }

    #[test]
    fn a_selection_that_keeps_changing_is_never_reported() {
        let mut tracker = SelectionTracker::default();
        tracker.observe(selected("he"));
        tracker.observe(selected("hel"));
        assert_eq!(
            tracker.observe(selected("hell")),
            SelectionChange::Unchanged
        );
    }

    #[test]
    fn losing_the_selection_clears_what_was_reported() {
        let mut tracker = SelectionTracker::default();
        tracker.observe(selected("hello"));
        tracker.observe(selected("hello"));
        assert_eq!(
            tracker.observe(SelectionReading::NothingSelected),
            SelectionChange::Cleared
        );
        assert_eq!(
            tracker.observe(SelectionReading::NothingSelected),
            SelectionChange::Unchanged
        );
    }

    #[test]
    fn a_short_run_of_unreadable_readings_keeps_the_selection() {
        let mut tracker = SelectionTracker::default();
        tracker.observe(selected("hello"));
        tracker.observe(selected("hello"));
        for _ in 0..UNREADABLE_READINGS_BEFORE_CLEARING - 1 {
            assert_eq!(
                tracker.observe(SelectionReading::Unreadable),
                SelectionChange::Unchanged
            );
        }
        assert_eq!(
            tracker.observe(selected("hello")),
            SelectionChange::Unchanged
        );
        assert_eq!(
            tracker.observe(SelectionReading::Unreadable),
            SelectionChange::Unchanged
        );
    }

    #[test]
    fn a_long_run_of_unreadable_readings_clears_the_selection() {
        let mut tracker = SelectionTracker::default();
        tracker.observe(selected("hello"));
        tracker.observe(selected("hello"));
        for _ in 0..UNREADABLE_READINGS_BEFORE_CLEARING - 1 {
            tracker.observe(SelectionReading::Unreadable);
        }
        assert_eq!(
            tracker.observe(SelectionReading::Unreadable),
            SelectionChange::Cleared
        );
    }

    #[test]
    fn a_blank_selection_counts_as_no_selection() {
        let mut tracker = SelectionTracker::default();
        tracker.observe(selected("hello"));
        tracker.observe(selected("hello"));
        assert_eq!(tracker.observe(selected("  \n")), SelectionChange::Cleared);
    }

    #[test]
    fn readings_that_cannot_tell_change_nothing() {
        let mut tracker = SelectionTracker::default();
        tracker.observe(selected("hello"));
        tracker.observe(selected("hello"));
        assert_eq!(
            tracker.observe(SelectionReading::Indeterminate),
            SelectionChange::Unchanged
        );
    }

    #[test]
    fn a_new_stable_selection_is_reported_after_the_previous_one() {
        let mut tracker = SelectionTracker::default();
        tracker.observe(selected("first"));
        tracker.observe(selected("first"));
        tracker.observe(selected("second"));
        assert!(matches!(
            tracker.observe(selected("second")),
            SelectionChange::Appeared(snapshot) if snapshot.text == "second"
        ));
    }
}
