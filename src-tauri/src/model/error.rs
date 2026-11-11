#[derive(Debug, thiserror::Error)]
pub enum PennaError {
    #[error("no text is selected")]
    EmptySelection,
    #[error("the agent returned an empty rewrite")]
    EmptyRewrite,
    #[error("could not start the agent '{program}': {source}")]
    AgentSpawn {
        program: String,
        source: std::io::Error,
    },
    #[error("the agent '{program}' failed with {status}: {stderr}")]
    AgentFailed {
        program: String,
        status: String,
        stderr: String,
    },
    #[error("clipboard failure: {0}")]
    Clipboard(#[from] arboard::Error),
    #[error("could not create the keyboard driver: {0}")]
    KeyboardDriver(#[from] enigo::NewConError),
    #[error("keyboard simulation failure: {0}")]
    KeyboardInput(#[from] enigo::InputError),
    #[error("desktop runtime failure: {0}")]
    Runtime(#[from] tauri::Error),
    #[error("global shortcut failure: {0}")]
    Shortcut(#[from] tauri_plugin_global_shortcut::Error),
    #[error("the main thread did not answer in time")]
    MainThreadTimeout,
}

pub type PennaResult<T> = Result<T, PennaError>;
