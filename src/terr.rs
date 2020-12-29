use std::fmt;
use std::fmt::Display;

use failure::{Backtrace, Context, Fail};

#[derive(Debug)]
pub struct TodoError {
    inner: Context<TodoErrorKind>,
}

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum TodoErrorKind {
    #[fail(display = "invalid value {} for {}", value, name)]
    InvalidValue { value: String, name: String },
    #[fail(display = "failed to save todo list")]
    SaveFailed,
    #[fail(display = "failed to load todo list")]
    LoadFailed,
    #[fail(display = "failed to append to file")]
    AppendFailed,
    #[fail(display = "failed to write todo list")]
    FileWriteFailed,
    #[fail(display = "first argument must be a command")]
    NotCommand,
    #[fail(display = "I/O Error: {}", err)]
    IOError { err: String },
}

impl Fail for TodoError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl TodoError {
    pub fn kind(&self) -> TodoErrorKind {
        self.inner.get_context().clone()
    }
}

impl From<TodoErrorKind> for TodoError {
    fn from(kind: TodoErrorKind) -> TodoError {
        TodoError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<TodoErrorKind>> for TodoError {
    fn from(inner: Context<TodoErrorKind>) -> TodoError {
        TodoError { inner }
    }
}
