#[derive(Debug, Clone)]
pub struct AgentCommand {
    pub program: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub agent: AgentCommand,
    pub instruction: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            agent: AgentCommand {
                program: "claude".to_string(),
                arguments: vec!["-p".to_string()],
            },
            instruction: "Check the following text for spelling, grammar and punctuation mistakes \
                and correct them. Keep the exact meaning, wording, tone and formatting of the \
                author: change only what is wrong and do not rephrase what is already correct. \
                Reply with the corrected text only, without quotes or explanations."
                .to_string(),
        }
    }
}
