use thiserror::Error;

#[derive(Error, Debug)]
pub enum TodoError {
    #[error("invalid value {0} for {1}")]
    InvalidValue(String, String), // value, name
    #[error("failed to save todo list")]
    SaveFailed,
    #[error("failed to load todo list")]
    LoadFailed,
    #[error("failed to append to file")]
    AppendFailed,
    #[error("failed to write todo list")]
    FileWriteFailed,
    #[error("first argument must be a command")]
    NotCommand,
    #[error("I/O Error: {0}")]
    IOError(String),
}
