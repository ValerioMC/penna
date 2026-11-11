const PROOFREADER_SYSTEM_PROMPT: &str =
    "You are a proofreader. The user message contains an instruction followed by a text. \
Reply with the corrected text only, without quotes or explanations.";

/// Runs the agent bare: no tools, skills, MCP servers, settings or plugins, so it is fast and
/// touches nothing on disk that macOS would ask Penna about.
fn agent_arguments() -> Vec<String> {
    [
        "-p",
        "--model",
        "sonnet",
        "--effort",
        "low",
        "--tools",
        "",
        "--disable-slash-commands",
        "--no-session-persistence",
        "--strict-mcp-config",
        "--setting-sources",
        "",
        "--system-prompt",
        PROOFREADER_SYSTEM_PROMPT,
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

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
                arguments: agent_arguments(),
            },
            instruction: "Check the following text for spelling, grammar and punctuation mistakes \
                and correct them. Keep the exact meaning, wording, tone and formatting of the \
                author: change only what is wrong and do not rephrase what is already correct. \
                Reply with the corrected text only, without quotes or explanations."
                .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_agent_loads_no_settings_plugins_or_mcp_servers() {
        let arguments = agent_arguments();
        assert!(arguments.contains(&"--strict-mcp-config".to_string()));
        let sources = arguments
            .iter()
            .position(|a| a == "--setting-sources")
            .unwrap();
        assert_eq!(arguments[sources + 1], "");
    }
}
