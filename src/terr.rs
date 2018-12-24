use std::fmt;
use std::fmt::Display;

use failure::{Backtrace, Context, Fail};

#[derive(Debug)]
pub struct TodoError {
    inner: Context<TodoErrorKind>,
}

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum TodoErrorKind {
    #[fail(display = "invalid entry: {}", entry)]
    InvalidSubject { entry: String },
    #[fail(display = "invalid todo ID: {}", id)]
    InvalidId { id: usize },
    #[fail(display = "invalid range: {}", range)]
    InvalidRange { range: String },
    #[fail(display = "failed to save todo list")]
    SaveFailed,
    #[fail(display = "invalid priority: {}", pri)]
    InvalidPriority { pri: String },
    #[fail(display = "invalid date: {}", dt)]
    InvalidDate { dt: String },
    #[fail(display = "invalid recurrence: {}", range)]
    InvalidRecRange { range: String },
    #[fail(display = "invalid date range: {}", range)]
    InvalidDateRange { range: String },
    #[fail(display = "invalid recurrence: {}", rec)]
    InvalidRecurrence { rec: String },
    #[fail(display = "invalid project pair: {}", pair)]
    InvalidProjectPair { pair: String },
    #[fail(display = "invalid context pair: {}", pair)]
    InvalidContextPair { pair: String },
    #[fail(display = "I/O Error: {}", err)]
    IOError { err: String },
}

impl Fail for TodoError {
    fn cause(&self) -> Option<&Fail> {
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
