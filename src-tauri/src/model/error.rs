use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum PennaError {
    #[error("the agent returned an empty text")]
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
    #[error("the main thread did not answer in time")]
    MainThreadTimeout,
    #[error("the Accessibility permission is not granted to Penna")]
    AccessibilityNotGranted,
    #[error("could not bring the application {process_id} to the front (status {status})")]
    ApplicationActivation { process_id: i32, status: i32 },
    #[error("the mouse location is not available")]
    MouseLocationUnavailable,
    #[error("file access failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not start logging: {0}")]
    Logging(String),
}

impl Serialize for PennaError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type PennaResult<T> = Result<T, PennaError>;
