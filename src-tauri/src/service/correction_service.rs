use std::sync::Arc;

use crate::gateway::agent_runner::AgentRunner;
use crate::gateway::selection_replacer::SelectionReplacer;
use crate::model::error::{PennaError, PennaResult};

pub struct CorrectionService {
    agent: Arc<dyn AgentRunner>,
    replacer: Arc<dyn SelectionReplacer>,
}

impl CorrectionService {
    pub fn new(agent: Arc<dyn AgentRunner>, replacer: Arc<dyn SelectionReplacer>) -> Self {
        Self { agent, replacer }
    }

    pub fn correct(&self, original: &str) -> PennaResult<String> {
        self.agent.correct(original)
    }

    pub fn apply(&self, target_process_id: i32, corrected: &str) -> PennaResult<()> {
        if corrected.trim().is_empty() {
            return Err(PennaError::EmptyRewrite);
        }
        self.replacer
            .replace_selection(target_process_id, corrected)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    struct FakeAgent;

    impl AgentRunner for FakeAgent {
        fn correct(&self, _text: &str) -> PennaResult<String> {
            Ok("I have a dog.".to_string())
        }
    }

    #[derive(Default)]
    struct FakeReplacer {
        replacements: Mutex<Vec<(i32, String)>>,
    }

    impl SelectionReplacer for FakeReplacer {
        fn replace_selection(&self, target_process_id: i32, text: &str) -> PennaResult<()> {
            self.replacements
                .lock()
                .unwrap()
                .push((target_process_id, text.to_string()));
            Ok(())
        }
    }

    fn service(replacer: &Arc<FakeReplacer>) -> CorrectionService {
        CorrectionService::new(Arc::new(FakeAgent), replacer.clone())
    }

    #[test]
    fn correct_returns_the_agent_text() {
        let replacer = Arc::new(FakeReplacer::default());
        assert_eq!(
            service(&replacer).correct("i has a dog").unwrap(),
            "I have a dog."
        );
    }

    #[test]
    fn apply_replaces_the_selection_of_the_target_application() {
        let replacer = Arc::new(FakeReplacer::default());
        service(&replacer).apply(42, "I have a dog.").unwrap();
        assert_eq!(
            *replacer.replacements.lock().unwrap(),
            vec![(42, "I have a dog.".to_string())]
        );
    }

    #[test]
    fn apply_refuses_blank_text_so_the_selection_is_never_erased() {
        let replacer = Arc::new(FakeReplacer::default());
        let error = service(&replacer).apply(42, "  \n").unwrap_err();
        assert!(matches!(error, PennaError::EmptyRewrite));
        assert!(replacer.replacements.lock().unwrap().is_empty());
    }
}
