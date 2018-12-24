use regex::Regex;

use chrono;
use todo_txt;

use crate::todo;

/// Span of todo record IDs to process. ID is an order number of the todo
/// record in the file starting from 0
#[derive(Debug, Clone)]
pub enum ItemRange {
    None,
    /// one record
    One(usize),
    /// inclusive range of records `(from, to).
    /// `Range(1, 4)` means all records from 1 through 4: 1, 2, 3, and 4.
    /// Values can exceed the number of todos. All IDs greater than the number
    /// of todos are skipped
    Range(usize, usize),
}

/// Todo state range
#[derive(Debug, Clone, PartialEq)]
pub enum TodoStatus {
    /// Only todos that are incompleted yet
    Active,
    /// All todos
    All,
    /// Only todos marked `done`
    Done,
}

/// An arbitrary range of values for todo properties check. The range is inclusive
#[derive(Debug, Clone, PartialEq)]
pub struct ValueRange {
    pub low: i64,
    pub high: i64,
}

impl Default for ValueRange {
    fn default() -> ValueRange {
        ValueRange { low: 0, high: 0 }
    }
}

/// A type of comparison for the property.
///
/// Every property supports only a limited subset:
/// * `project` and `context`: do not use `ValueSpan` because they always search for a given text;
/// * `priority`: `None`, `Any`, `Equal`, `Lower`, and `Higher`;
/// * `recurrence`: `None` and `Any`;
/// * `due`: `None`, `Any`, `Lower`, and `Range`;
#[derive(Debug, Clone, PartialEq)]
pub enum ValueSpan {
    /// Do not check the property value
    None, // without
    /// Property value must equal a given one (projects and contexts provide
    /// more ways to compare, including simple pattern matching)
    Equal, // one value
    /// Property value must be equal to or less than the given one
    Lower, // -
    /// Property value must be equal to or greater than the given one
    Higher, // +
    /// Property must be set to any value except None or empty string. Useful,
    /// e.g, to select all todos with any due date or priority
    Any, // any|(+ without priority)
    /// Property value must be within range (range is inclusive)
    Range, // from - to
}

/// For filtering by due date. `days` is inclusive range and is not used when
/// `span` is `Any` or `None`
#[derive(Debug, Clone, PartialEq)]
pub struct Due {
    pub days: ValueRange,
    pub span: ValueSpan,
}
impl Default for Due {
    fn default() -> Due {
        Due {
            span: ValueSpan::None,
            days: Default::default(),
        }
    }
}

/// For filtering by recurrence. Only `Any` and `None` are supported
#[derive(Debug, Clone, PartialEq)]
pub struct Recurrence {
    pub span: ValueSpan,
}
impl Default for Recurrence {
    fn default() -> Recurrence {
        Recurrence { span: ValueSpan::None }
    }
}

/// For filtering by priority
#[derive(Debug, Clone, PartialEq)]
pub struct Priority {
    pub value: u8,
    pub span: ValueSpan,
}
impl Default for Priority {
    fn default() -> Priority {
        Priority {
            value: todo::NO_PRIORITY,
            span: ValueSpan::None,
        }
    }
}

/// A rules for todo list filtering. Setting a field to None or empty vector
/// means that the corresponding property is not checked.
/// All text comparisons are case-insensitive.
#[derive(Debug, Clone)]
pub struct Conf {
    /// Range of todo IDs
    pub range: ItemRange,
    /// List of all project tags that a todo must include. The search
    /// supports very limited pattern matching:
    /// * `foo*` - finds all todos with projects that starts with `foo`
    /// * `*foo` - finds all todos with projects that ends with `foo`
    /// * `*foo*` - finds all todos with projects that contains `foo`
    pub projects: Vec<String>,
    /// List of all context tags that a todo must include. The search
    /// supports very limited pattern matching:
    /// * `foo*` - finds all todos with contexts that starts with `foo`
    /// * `*foo` - finds all todos with contexts that ends with `foo`
    /// * `*foo*` - finds all todos with contexts that contains `foo`
    pub contexts: Vec<String>,
    /// A text that any of text, project, or context must contain
    pub regex: Option<String>,
    /// If it is `true`, `regex` is treated as regular expression. If `use_regex`
    /// is `false`, the value of `regex` is just a substring to search for
    pub use_regex: bool,

    /// All incomplete, completed, or both types of todos
    pub all: TodoStatus,

    /// Search for a due date
    pub due: Option<Due>,
    /// Search for a recurrent todos
    pub rec: Option<Recurrence>,
    /// Search for a priority
    pub pri: Option<Priority>,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            range: ItemRange::None,
            projects: Vec::new(),
            contexts: Vec::new(),
            regex: None,
            use_regex: false,

            all: TodoStatus::Active,
            due: None,
            rec: None,
            pri: None,
        }
    }
}

fn filter_regex(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    let rx = match &c.regex {
        None => return v,
        Some(s) => s,
    };

    let mut new_v: todo::IDVec = Vec::new();
    if c.use_regex {
        let rx = match Regex::new(&format!("(?i){}", rx)) {
            Err(_) => {
                println!("Invalid regex");
                return v;
            }
            Ok(v) => v,
        };

        for i in v.iter() {
            let idx = *i;
            if idx >= tasks.len() {
                continue;
            }
            if rx.is_match(&tasks[idx].subject) {
                new_v.push(idx);
            }
        }
        return new_v;
    }

    let rstr = rx.to_lowercase();
    for i in v.iter() {
        let idx = *i;
        let low = tasks[idx].subject.to_lowercase();
        if low.find(&rstr).is_some() {
            new_v.push(idx);
            continue;
        }
    }
    new_v
}

fn filter_context(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    if c.contexts.is_empty() {
        return v;
    }

    let mut new_v: todo::IDVec = Vec::new();
    for i in v.iter() {
        let idx = *i;
        'outer: for ctx in tasks[idx].contexts.iter() {
            let low = ctx.to_lowercase();
            for tag in c.contexts.iter() {
                let tag = tag.to_lowercase();
                if str_matches(&low, &tag) {
                    new_v.push(idx);
                    break 'outer;
                }
            }
        }
    }
    new_v
}

fn filter_project(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    if c.projects.is_empty() {
        return v;
    }

    let mut new_v: todo::IDVec = Vec::new();
    for i in v.iter() {
        let idx = *i;
        'outer: for prj in tasks[idx].projects.iter() {
            let low = prj.to_lowercase();
            for tag in c.projects.iter() {
                let tag = tag.to_lowercase();
                if str_matches(&low, &tag) {
                    new_v.push(idx);
                    break 'outer;
                }
            }
        }
    }
    new_v
}

fn filter_priority(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    match &c.pri {
        None => v,
        Some(p) => {
            let mut new_v: todo::IDVec = Vec::new();
            for i in v.iter() {
                let idx = *i;
                match p.span {
                    ValueSpan::None => {
                        if tasks[idx].priority == todo::NO_PRIORITY {
                            new_v.push(idx);
                        }
                    }
                    ValueSpan::Equal => {
                        if p.value == tasks[idx].priority {
                            new_v.push(idx);
                        }
                    }
                    ValueSpan::Lower => {
                        if p.value <= tasks[idx].priority {
                            new_v.push(idx);
                        }
                    }
                    ValueSpan::Higher => {
                        if p.value >= tasks[idx].priority {
                            new_v.push(idx);
                        }
                    }
                    ValueSpan::Any => {
                        if tasks[idx].priority < todo::NO_PRIORITY {
                            new_v.push(idx);
                        }
                    }
                    _ => {}
                }
            }
            new_v
        }
    }
}

fn filter_recurrence(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    match &c.rec {
        None => v,
        Some(r) => {
            let mut new_v: todo::IDVec = Vec::new();
            for i in v.iter() {
                let idx = *i;
                match r.span {
                    ValueSpan::None => {
                        if tasks[idx].recurrence.is_none() {
                            new_v.push(idx);
                        }
                    }
                    ValueSpan::Any => {
                        if tasks[idx].recurrence.is_some() {
                            new_v.push(idx);
                        }
                    }
                    _ => {}
                }
            }
            new_v
        }
    }
}

fn filter_due(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    match &c.due {
        None => v,
        Some(due) => {
            let today = chrono::Local::now().date().naive_local();
            let mut new_v: todo::IDVec = Vec::new();
            for i in v.iter() {
                let idx = *i;
                match due.span {
                    ValueSpan::None => {
                        if tasks[idx].due_date.is_none() {
                            new_v.push(idx);
                        }
                    }
                    ValueSpan::Any => {
                        if tasks[idx].due_date.is_some() {
                            new_v.push(idx);
                        }
                    }
                    ValueSpan::Lower => {
                        if let Some(d) = tasks[idx].due_date {
                            let diff = d - today;
                            if diff.num_days() < due.days.low {
                                new_v.push(idx);
                            }
                        }
                    }
                    ValueSpan::Range => {
                        if let Some(d) = tasks[idx].due_date {
                            let diff = d - today;
                            if diff.num_days() >= due.days.low && diff.num_days() <= due.days.high {
                                new_v.push(idx);
                            }
                        }
                    }
                    _ => {}
                }
            }
            new_v
        }
    }
}

fn is_status_ok(task: &todo_txt::task::Extended, status: &TodoStatus) -> bool {
    !((*status == TodoStatus::Active && task.finished) || (*status == TodoStatus::Done && !task.finished))
}

/// Entry function to filter the list of todo records
///
/// The function does not modify the todo list. It looks through the todo
/// records and fill the vector with ID of todos which meet all the criteria.
///
/// * `tasks` - list of todos to filter
/// * `c` - filtering rules
///
/// Returns:
/// the list of todo IDs which meet filtering criteria
pub fn filter(tasks: &todo::TaskSlice, c: &Conf) -> todo::IDVec {
    let mut v: todo::IDVec = Vec::new();

    match c.range {
        ItemRange::One(i) => {
            if i < tasks.len() && is_status_ok(&tasks[i], &c.all) {
                v.push(i);
            }
        }
        ItemRange::Range(min, max) => {
            let mut start = min;

            while start <= max {
                if start >= tasks.len() {
                    break;
                }
                if is_status_ok(&tasks[start], &c.all) {
                    v.push(start);
                }
                start += 1;
            }
        }
        _ => {
            for (i, ref item) in tasks.iter().enumerate() {
                if is_status_ok(item, &c.all) {
                    v.push(i);
                }
            }
        }
    }
    v = filter_regex(tasks, v, c);
    v = filter_project(tasks, v, c);
    v = filter_context(tasks, v, c);
    v = filter_priority(tasks, v, c);
    v = filter_recurrence(tasks, v, c);
    v = filter_due(tasks, v, c);

    v
}

fn str_matches(orig: &str, patt: &str) -> bool {
    if patt.starts_with('*') && patt.ends_with('*') {
        let p = patt.trim_matches('*');
        orig.find(p).is_some()
    } else if patt.starts_with('*') {
        let p = patt.trim_start_matches('*');
        orig.ends_with(p)
    } else if patt.ends_with('*') {
        let p = patt.trim_end_matches('*');
        orig.starts_with(p)
    } else {
        orig == patt
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn matches() {
        assert!(!str_matches("abcd", "abc"));
        assert!(str_matches("abcd", "abcd"));
        assert!(!str_matches("abcd", "abcde"));
        assert!(str_matches("abcd", "abc*"));
        assert!(str_matches("abcd", "*bcd"));
        assert!(str_matches("abcd", "*b*"));
        assert!(!str_matches("abcd", "bc*"));
        assert!(!str_matches("abcd", "*bc"));
        assert!(!str_matches("abcd", ""));
        assert!(!str_matches("", "abcd"));
        assert!(str_matches("abcd", "*d*"));
        assert!(str_matches("abcd", "*a*"));
    }
}