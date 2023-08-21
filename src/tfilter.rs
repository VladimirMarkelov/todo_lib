use regex::Regex;

use crate::timer;
use crate::todo;
use crate::todotxt;

/// Setting unused end of Lower/Higher ValueRange makes the filter to include
/// todos that have a given date field undefined
pub const INCLUDE_NONE: i64 = -9_999_998;
const NONE_TITLE: &str = "none";
const ANY_TITLE: &str = "any";

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
    /// List of IDs
    List(Vec<usize>),
}

/// Todo state range
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TodoStatus {
    /// Only todos that are incompleted yet
    Active,
    /// All todos
    All,
    /// Only todos marked `done`
    Done,
    /// Only empty todos
    Empty,
}

/// An arbitrary range of values for todo properties check. The range is inclusive
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValueRange {
    pub low: i64,
    pub high: i64,
}

/// A type of comparison for the property.
///
/// Every property supports only a limited subset:
/// * `project` and `context`: do not use `ValueSpan` because they always search for a given text;
/// * `priority`: `None`, `Any`, `Equal`, `Lower`, and `Higher`;
/// * `recurrence`: `None` and `Any`;
/// * `due`: `None`, `Any`, `Lower`, and `Range`;
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Timer is running
    Active,
}

/// For filtering by date range or value. `days` is inclusive range and
/// is not used when `span` is `Any` or `None`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRange {
    pub days: ValueRange,
    pub span: ValueSpan,
}
impl Default for DateRange {
    fn default() -> DateRange {
        DateRange { span: ValueSpan::None, days: Default::default() }
    }
}

/// For filtering by recurrence. Only `Any` and `None` are supported
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recurrence {
    pub span: ValueSpan,
}
impl Default for Recurrence {
    fn default() -> Recurrence {
        Recurrence { span: ValueSpan::None }
    }
}

/// For filtering by priority
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Priority {
    pub value: u8,
    pub span: ValueSpan,
}
impl Default for Priority {
    fn default() -> Priority {
        Priority { value: todotxt::NO_PRIORITY, span: ValueSpan::None }
    }
}

/// For filtering by timer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timer {
    pub span: ValueSpan,
    pub value: usize,
}
impl Default for Timer {
    fn default() -> Timer {
        Timer { value: 0, span: ValueSpan::None }
    }
}

/// Filter rules for special entities: projects, contexts, tags.
#[derive(Debug, Clone)]
pub struct TagFilter {
    /// List of all project that a todo must include. The search
    /// supports very limited pattern matching:
    /// * `foo*` - finds all todos with projects that starts with `foo`
    /// * `*foo` - finds all todos with projects that ends with `foo`
    /// * `*foo*` - finds all todos with projects that contains `foo`
    /// Special values:
    /// * none - select todos with no contexts
    /// * any - select todos that have at least one context
    pub projects: Vec<String>,
    /// List of all context that a todo must include. The search
    /// supports very limited pattern matching:
    /// * `foo*` - finds all todos with contexts that starts with `foo`
    /// * `*foo` - finds all todos with contexts that ends with `foo`
    /// * `*foo*` - finds all todos with contexts that contains `foo`
    /// Special values:
    /// * none - select todos with no contexts
    /// * any - select todos that have at least one context
    pub contexts: Vec<String>,
    /// List of all tags that a todo must include. The search
    /// supports very limited pattern matching:
    /// * `foo*` - finds all todos with tags that starts with `foo`
    /// * `*foo` - finds all todos with tags that ends with `foo`
    /// * `*foo*` - finds all todos with tags that contains `foo`
    /// Special values:
    /// * none - select todos with no tags
    /// * any - select todos that have at least one tag
    pub tags: Vec<String>,
    /// List of all hashtags that a todo must include. The search
    /// supports very limited pattern matching:
    /// * `foo*` - finds all todos with hashtags that starts with `foo`
    /// * `*foo` - finds all todos with hashtags that ends with `foo`
    /// * `*foo*` - finds all todos with hashtags that contains `foo`
    /// Special values:
    /// * none - select todos with no hashtags
    /// * any - select todos that have at least one tag
    pub hashtags: Vec<String>,
}

/// A rules for todo list filtering. Setting a field to None or empty vector
/// means that the corresponding property is not checked.
/// All text comparisons are case-insensitive.
#[derive(Debug, Clone)]
pub struct Conf {
    /// Range of todo IDs
    pub range: ItemRange,
    /// A text that any of text, project, or context must contain
    pub regex: Option<String>,
    /// If it is `true`, `regex` is treated as regular expression. If `use_regex`
    /// is `false`, the value of `regex` is just a substring to search for
    pub use_regex: bool,

    /// Todos must contain the following values to be included in the list.
    pub include: TagFilter,
    // Todos that include the following values are excluded from the list.
    pub exclude: TagFilter,

    /// All incomplete, completed, or both types of todos
    pub all: TodoStatus,

    /// Search for a due date: any, no due date, or withing range
    pub due: Option<DateRange>,
    /// Search for a threshold date: any, no threshold date, or withing range
    pub thr: Option<DateRange>,
    /// Search for recurrent todos
    pub rec: Option<Recurrence>,
    /// Search for todos with priority or priority range
    pub pri: Option<Priority>,
    /// Search for todos with timer related stuff: active, inactive, any time spent
    pub tmr: Option<Timer>,
    /// Search for a creation date: any, no create date, or withing range
    pub created: Option<DateRange>,
    /// Search for a finished date: any, no finish date, or withing range
    pub finished: Option<DateRange>,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            range: ItemRange::None,
            include: TagFilter { projects: Vec::new(), contexts: Vec::new(), tags: Vec::new(), hashtags: Vec::new() },
            exclude: TagFilter { projects: Vec::new(), contexts: Vec::new(), tags: Vec::new(), hashtags: Vec::new() },
            regex: None,
            use_regex: false,

            all: TodoStatus::Active,
            due: None,
            thr: None,
            rec: None,
            pri: None,
            tmr: None,
            created: None,
            finished: None,
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
        let rx = match Regex::new(&format!("(?i){rx}")) {
            Err(e) => {
                eprintln!("Invalid regex: {}", e);
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
        if low.contains(&rstr) {
            new_v.push(idx);
            continue;
        }
    }
    new_v
}

fn filter_empty(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    if c.all == TodoStatus::All {
        return v;
    }
    let mut new_v: todo::IDVec = Vec::new();
    for i in v.iter() {
        let idx = *i;
        let empty = tasks[idx].subject.is_empty();
        if (empty && c.all == TodoStatus::Empty) || (!empty && c.all != TodoStatus::Empty) {
            new_v.push(idx);
        }
    }
    new_v
}

fn vec_match(task_list: &[String], filter: &[String]) -> bool {
    if filter.is_empty() {
        return true;
    }
    for f in filter.iter() {
        if (f == NONE_TITLE && task_list.is_empty()) || (f == ANY_TITLE && !task_list.is_empty()) {
            return true;
        }
    }
    for ctx in task_list.iter() {
        let low = ctx.to_lowercase();
        for tag in filter.iter() {
            let ltag = tag.to_lowercase();
            if str_matches(&low, &ltag) {
                return true;
            }
        }
    }
    false
}

fn filter_context(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    if c.include.contexts.is_empty() && c.exclude.contexts.is_empty() {
        return v;
    }

    let mut new_v: todo::IDVec = Vec::new();
    for i in v.iter() {
        let idx = *i;
        if !c.exclude.contexts.is_empty() && vec_match(&tasks[idx].contexts, &c.exclude.contexts) {
            continue;
        }
        if c.include.contexts.is_empty() || vec_match(&tasks[idx].contexts, &c.include.contexts) {
            new_v.push(idx);
        }
    }
    new_v
}

fn filter_project(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    if c.include.projects.is_empty() && c.exclude.projects.is_empty() {
        return v;
    }

    let mut new_v: todo::IDVec = Vec::new();
    for i in v.iter() {
        let idx = *i;
        if !c.exclude.projects.is_empty() && vec_match(&tasks[idx].projects, &c.exclude.projects) {
            continue;
        }
        if c.include.projects.is_empty() || vec_match(&tasks[idx].projects, &c.include.projects) {
            new_v.push(idx);
        }
    }
    new_v
}

fn filter_tag(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    if c.include.tags.is_empty() && c.exclude.tags.is_empty() {
        return v;
    }

    let mut new_v: todo::IDVec = Vec::new();
    for i in v.iter() {
        let idx = *i;
        let mut tag_list: Vec<String> = Vec::new();
        for (k, _v) in tasks[idx].tags.iter() {
            tag_list.push(k.to_string());
        }
        if !c.exclude.tags.is_empty() && vec_match(&tag_list, &c.exclude.tags) {
            continue;
        }
        if c.include.tags.is_empty() || vec_match(&tag_list, &c.include.tags) {
            new_v.push(idx);
        }
    }
    new_v
}

fn filter_hashtag(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    if c.include.hashtags.is_empty() && c.exclude.hashtags.is_empty() {
        return v;
    }

    let mut new_v: todo::IDVec = Vec::new();
    for i in v.iter() {
        let idx = *i;
        let mut hashtag_list: Vec<String> = Vec::new();
        for k in tasks[idx].hashtags.iter() {
            hashtag_list.push(k.to_string());
        }
        if !c.exclude.hashtags.is_empty() && vec_match(&hashtag_list, &c.exclude.hashtags) {
            continue;
        }
        if c.include.hashtags.is_empty() || vec_match(&hashtag_list, &c.include.hashtags) {
            new_v.push(idx);
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
                        if tasks[idx].priority == todotxt::NO_PRIORITY {
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
                        if tasks[idx].priority < todotxt::NO_PRIORITY {
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
            let mut new_v: todo::IDVec = Vec::new();
            for i in v.iter() {
                let idx = *i;
                if date_in_range(&tasks[idx].due_date, due) {
                    new_v.push(idx);
                }
            }
            new_v
        }
    }
}

fn filter_created(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    match &c.created {
        None => v,
        Some(created) => {
            let mut new_v: todo::IDVec = Vec::new();
            for i in v.iter() {
                let idx = *i;
                if date_in_range(&tasks[idx].create_date, created) {
                    new_v.push(idx);
                }
            }
            new_v
        }
    }
}

fn filter_finished(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    match &c.finished {
        None => v,
        Some(finished) => {
            let mut new_v: todo::IDVec = Vec::new();
            for i in v.iter() {
                let idx = *i;
                if date_in_range(&tasks[idx].finish_date, finished) {
                    new_v.push(idx);
                }
            }
            new_v
        }
    }
}

fn filter_threshold(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    let flt = if let Some(thr) = &c.thr {
        thr.clone()
    } else {
        DateRange { days: ValueRange { low: INCLUDE_NONE, high: 0 }, span: ValueSpan::Range }
    };
    let mut new_v: todo::IDVec = Vec::new();
    for i in v.iter() {
        let idx = *i;
        if c.all == TodoStatus::All || date_in_range(&tasks[idx].threshold_date, &flt) {
            new_v.push(idx);
        }
    }
    new_v
}

fn date_in_range(date: &Option<chrono::NaiveDate>, range: &DateRange) -> bool {
    let today = chrono::Local::now().date_naive();
    match range.span {
        ValueSpan::None => date.is_none(),
        ValueSpan::Any => date.is_some(),
        ValueSpan::Higher => {
            if let Some(d) = date {
                let diff = *d - today;
                diff.num_days() > range.days.high
            } else {
                range.days.low == INCLUDE_NONE
            }
        }
        ValueSpan::Lower => {
            if let Some(d) = date {
                let diff = *d - today;
                diff.num_days() < range.days.low
            } else {
                range.days.high == INCLUDE_NONE
            }
        }
        ValueSpan::Range => {
            if let Some(d) = date {
                let diff = *d - today;
                match (range.days.low, range.days.high) {
                    (INCLUDE_NONE, INCLUDE_NONE) => false,
                    (INCLUDE_NONE, d) => diff.num_days() <= d,
                    (d, INCLUDE_NONE) => diff.num_days() >= d,
                    (b, e) => diff.num_days() >= b && diff.num_days() <= e,
                }
            } else {
                range.days.low == INCLUDE_NONE || range.days.high == INCLUDE_NONE
            }
        }
        _ => false,
    }
}

fn filter_timer(tasks: &todo::TaskSlice, v: todo::IDVec, c: &Conf) -> todo::IDVec {
    match &c.tmr {
        None => v,
        Some(r) => {
            let mut new_v: todo::IDVec = Vec::new();
            for i in v.iter() {
                let idx = *i;
                match r.span {
                    ValueSpan::None => {
                        if !timer::is_timer_on(&tasks[idx]) {
                            new_v.push(idx);
                        }
                    }
                    ValueSpan::Active => {
                        if timer::is_timer_on(&tasks[idx]) {
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

fn is_status_ok(task: &todotxt::Task, status: &TodoStatus) -> bool {
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
        ItemRange::List(ref lst) => {
            for idx in lst.iter() {
                if *idx >= tasks.len() {
                    continue;
                }
                if is_status_ok(&tasks[*idx], &c.all) {
                    v.push(*idx);
                }
            }
        }
        _ => {
            for (i, item) in tasks.iter().enumerate() {
                if is_status_ok(item, &c.all) {
                    v.push(i);
                }
            }
        }
    }
    v = filter_empty(tasks, v, c);
    v = filter_regex(tasks, v, c);
    v = filter_tag(tasks, v, c);
    v = filter_hashtag(tasks, v, c);
    v = filter_project(tasks, v, c);
    v = filter_context(tasks, v, c);
    v = filter_priority(tasks, v, c);
    v = filter_recurrence(tasks, v, c);
    v = filter_due(tasks, v, c);
    v = filter_created(tasks, v, c);
    v = filter_finished(tasks, v, c);
    v = filter_threshold(tasks, v, c);
    v = filter_timer(tasks, v, c);

    v
}

fn str_matches(orig: &str, patt: &str) -> bool {
    if patt.starts_with('*') && patt.ends_with('*') {
        let p = patt.trim_matches('*');
        orig.contains(p)
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
