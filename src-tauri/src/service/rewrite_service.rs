use std::sync::Arc;

use crate::gateway::agent_runner::AgentRunner;
use crate::gateway::selection_gateway::SelectionGateway;
use crate::model::error::{PennaError, PennaResult};

pub struct RewriteService {
    selection: Arc<dyn SelectionGateway>,
    agent: Arc<dyn AgentRunner>,
}

impl RewriteService {
    pub fn new(selection: Arc<dyn SelectionGateway>, agent: Arc<dyn AgentRunner>) -> Self {
        Self { selection, agent }
    }

    pub fn rewrite_selection(&self) -> PennaResult<()> {
        let selected_text = self.selection.read_selection()?;
        if selected_text.trim().is_empty() {
            return Err(PennaError::EmptySelection);
        }
        let rewritten_text = self.agent.rewrite(&selected_text)?;
        self.selection.replace_selection(&rewritten_text)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    struct FakeSelection {
        selected_text: String,
        replacements: Mutex<Vec<String>>,
    }

    impl FakeSelection {
        fn with_selection(selected_text: &str) -> Arc<Self> {
            Arc::new(Self {
                selected_text: selected_text.to_string(),
                replacements: Mutex::new(Vec::new()),
            })
        }

        fn replacements(&self) -> Vec<String> {
            self.replacements.lock().unwrap().clone()
        }
    }

    impl SelectionGateway for FakeSelection {
        fn read_selection(&self) -> PennaResult<String> {
            Ok(self.selected_text.clone())
        }

        fn replace_selection(&self, text: &str) -> PennaResult<()> {
            self.replacements.lock().unwrap().push(text.to_string());
            Ok(())
        }
    }

    struct FakeAgent {
        outcome: fn() -> PennaResult<String>,
    }

    impl AgentRunner for FakeAgent {
        fn rewrite(&self, _text: &str) -> PennaResult<String> {
            (self.outcome)()
        }
    }

    fn service(
        selection: &Arc<FakeSelection>,
        outcome: fn() -> PennaResult<String>,
    ) -> RewriteService {
        RewriteService::new(selection.clone(), Arc::new(FakeAgent { outcome }))
    }

    #[test]
    fn replaces_the_selection_with_the_agent_rewrite() {
        let selection = FakeSelection::with_selection("i has a dog");
        service(&selection, || Ok("I have a dog.".to_string()))
            .rewrite_selection()
            .unwrap();
        assert_eq!(selection.replacements(), vec!["I have a dog."]);
    }

    #[test]
    fn rejects_a_blank_selection_without_calling_the_agent() {
        let selection = FakeSelection::with_selection("  \n");
        let error = service(&selection, || panic!("agent must not run"))
            .rewrite_selection()
            .unwrap_err();
        assert!(matches!(error, PennaError::EmptySelection));
        assert!(selection.replacements().is_empty());
    }

    #[test]
    fn keeps_the_selection_when_the_agent_fails() {
        let selection = FakeSelection::with_selection("i has a dog");
        let error = service(&selection, || Err(PennaError::EmptyRewrite))
            .rewrite_selection()
            .unwrap_err();
        assert!(matches!(error, PennaError::EmptyRewrite));
        assert!(selection.replacements().is_empty());
    }
}
