use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

#[derive(Debug, Clone)]
pub struct AgentCommand {
    pub program: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub agent: AgentCommand,
    pub instruction: String,
    pub shortcut: Shortcut,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            agent: AgentCommand {
                program: "claude".to_string(),
                arguments: vec!["-p".to_string()],
            },
            instruction: "Rewrite the following text in correct, natural English. \
                Fix grammar, spelling and punctuation, and keep the original meaning and tone. \
                Reply with the rewritten text only, without quotes or explanations."
                .to_string(),
            shortcut: Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyP),
        }
    }
}
