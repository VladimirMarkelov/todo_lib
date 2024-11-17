//! # Todo.txt utilities
//!
//! `todo_lib` is a collection of utilities for processing todo lists in todo.txt
//! format. Please read details about the format at http://todotxt.org/
//!
//! All functions works with a list of todo entries. Supported operations:
//! * loading and saving todo lists;
//! * add/delete/edit todo records;
//! * mark todo records done/undone or archive completed ones;
//! * set/remove/replace todo properties, like priority or project, of one or more records;
//! * basic support of recurring todos;
//! * rich filtering and sorting capabilities.
//!
//! Almost all the functions support group operations. Exceptions are adding a
//! new todo record and replacing a todo record text.

pub mod date_expr;
pub mod human_date;
pub mod terr;
pub mod tfilter;
pub mod timer;
pub mod todo;
pub mod todotxt;
pub mod tsort;
