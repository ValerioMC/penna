use std::process::{Command, Stdio};

use crate::model::error::{PennaError, PennaResult};
use crate::model::settings::AgentCommand;

pub trait AgentRunner: Send + Sync {
    fn correct(&self, text: &str) -> PennaResult<String>;
}

pub struct CliAgentRunner {
    command: AgentCommand,
    instruction: String,
}

impl CliAgentRunner {
    pub fn new(command: AgentCommand, instruction: String) -> Self {
        Self {
            command,
            instruction,
        }
    }

    fn build_prompt(&self, text: &str) -> String {
        format!("{}\n\n{}", self.instruction, text)
    }

    #[cfg(unix)]
    fn build_process(&self, prompt: &str) -> Command {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let mut process = Command::new(shell);
        process
            .args(["-l", "-c", r#"exec "$0" "$@""#])
            .arg(&self.command.program)
            .args(&self.command.arguments)
            .arg(prompt);
        isolate_from_environment(&mut process);
        process
    }

    #[cfg(not(unix))]
    fn build_process(&self, prompt: &str) -> Command {
        let mut process = Command::new(&self.command.program);
        process.args(&self.command.arguments).arg(prompt);
        isolate_from_environment(&mut process);
        process
    }
}

impl AgentRunner for CliAgentRunner {
    fn correct(&self, text: &str) -> PennaResult<String> {
        let output = self
            .build_process(&self.build_prompt(text))
            .stdin(Stdio::null())
            .output()
            .map_err(|source| PennaError::AgentSpawn {
                program: self.command.program.clone(),
                source,
            })?;
        interpret_output(
            &self.command.program,
            output.status.success(),
            &output.status.to_string(),
            &output.stdout,
            &output.stderr,
        )
    }
}

/// The agent is a child of Penna, so macOS attributes whatever it touches to Penna.
/// It runs from the temporary directory and without the markers of an enclosing Claude session.
fn isolate_from_environment(process: &mut Command) {
    process.current_dir(std::env::temp_dir());
    process.env_remove("CLAUDECODE");
    process.env_remove("CLAUDE_CODE_ENTRYPOINT");
}

fn interpret_output(
    program: &str,
    succeeded: bool,
    status: &str,
    stdout: &[u8],
    stderr: &[u8],
) -> PennaResult<String> {
    if !succeeded {
        return Err(PennaError::AgentFailed {
            program: program.to_string(),
            status: status.to_string(),
            stderr: String::from_utf8_lossy(stderr).trim().to_string(),
        });
    }
    let rewrite = String::from_utf8_lossy(stdout).trim().to_string();
    if rewrite.is_empty() {
        return Err(PennaError::EmptyRewrite);
    }
    Ok(rewrite)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runner() -> CliAgentRunner {
        CliAgentRunner::new(
            AgentCommand {
                program: "claude".to_string(),
                arguments: vec!["-p".to_string()],
            },
            "Fix this.".to_string(),
        )
    }

    #[test]
    fn prompt_puts_the_instruction_before_the_text() {
        assert_eq!(
            runner().build_prompt("i has a dog"),
            "Fix this.\n\ni has a dog"
        );
    }

    #[test]
    fn the_agent_does_not_inherit_the_claude_session_marker() {
        let process = runner().build_process("prompt");
        let removed = process
            .get_envs()
            .any(|(name, value)| name == "CLAUDECODE" && value.is_none());
        assert!(removed);
    }

    #[test]
    fn the_agent_runs_from_the_temporary_directory() {
        let process = runner().build_process("prompt");
        assert_eq!(
            process.get_current_dir(),
            Some(std::env::temp_dir().as_path())
        );
    }

    #[test]
    fn successful_output_is_trimmed() {
        let rewrite = interpret_output("claude", true, "exit status: 0", b"  I have a dog.\n", b"");
        assert_eq!(rewrite.unwrap(), "I have a dog.");
    }

    #[test]
    fn failed_output_reports_the_agent_stderr() {
        let error = interpret_output("claude", false, "exit status: 1", b"", b"not logged in\n")
            .unwrap_err();
        assert!(matches!(
            error,
            PennaError::AgentFailed { ref stderr, .. } if stderr == "not logged in"
        ));
    }

    #[test]
    fn blank_output_is_an_empty_rewrite() {
        let error = interpret_output("claude", true, "exit status: 0", b" \n", b"").unwrap_err();
        assert!(matches!(error, PennaError::EmptyRewrite));
    }
}
