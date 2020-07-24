use failure::ResultExt;
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use crate::terr;
use crate::timer;
use crate::tsort;
use caseless::default_caseless_match_str;

/// The ID value returned instead of new todo ID if adding a new todo fails
pub const INVALID_ID: usize = 9_999_999_999;
/// Empty priority - means a todo do not have any priority set
pub const NO_PRIORITY: u8 = 26u8;
pub const TIMER_TAG: &str = "tmr";
pub const SPENT_TAG: &str = "spent";
pub const TIMER_OFF: &str = "off";

pub type TaskVec = Vec<todo_txt::task::Extended>;
pub type TaskSlice = [todo_txt::task::Extended];
pub type IDVec = Vec<usize>;
pub type IDSlice = [usize];
pub type ChangedVec = Vec<bool>;
pub type ChangedSlice = [bool];

/// Type of operation applied to todo properties. Every field supports
/// its own set of operations (except `None` that can be used for all of them):
/// * priority: `Set`, `Delete`, `Increase`, `Decrease`;
/// * due date: `Set`, `Delete`;
/// * recurrence: `Set`, `Delete`;
/// * projects: `Set`, `Delete`, `Replace`;
/// * contexts: `Set`, `Delete`, `Replace`;
#[derive(Debug, Clone)]
pub enum Action {
    /// do not touch the property
    None,
    /// Priority, due date, and recurrence: set the new value;
    /// Projects and contexts: add a new value to the list;
    Set,
    /// Remove the value
    Delete,
    /// Replace old value with a new one. The format for the new value:
    /// projects: `old_value+new_value`
    /// contexts: `old_value@new_value`
    Replace,
    /// Only for priority: increases the priority by one level. If a todo
    /// has A priority the todo is not changed. If a todo does not have a
    /// priority it gets the lowest one `Z`
    Increase,
    /// Only for priority: decreases the priority by one level. If a todo
    /// has no priority the todo is not changed. If a todo has the lowest
    /// priority `Z` the priority is removed
    Decrease,
}

/// The list of changes to apply to all records in a list. All operations with
/// text are case insensitive, so if you, e.g., try to replace a project
/// `projectone` to `ProjectOne` no todo is updated
#[derive(Debug, Clone)]
pub struct Conf {
    done: bool,
    /// New subject replaces the old one. Trying to change the subject for more
    /// than one todo results in that only the first todo in list is changed,
    /// all the rest todos are skipped.
    ///
    /// NOTE: the new subject must contain all required attributes: projects,
    /// contexts, recurrence, due date etc because after assigning a new
    /// subject it is parsed and all todo properties are filled with parsed data
    pub subject: Option<String>,
    /// New priority (is not used if action is `Delete, `Increase` or `Decrease`)
    pub priority: u8,
    /// Type of operation applied to old priority
    pub priority_act: Action,
    /// New due date
    pub due: Option<chrono::NaiveDate>,
    /// Type of operation applied to old due date
    pub due_act: Action,
    /// New threshold date
    pub thr: Option<chrono::NaiveDate>,
    /// Type of operation applied to old threshold date
    pub thr_act: Action,
    /// New recurrence
    pub recurrence: Option<todo_txt::task::Recurrence>,
    /// Type of operation applied to old recurrence
    pub recurrence_act: Action,
    /// List of projects.
    /// For `Set` and `Delete` is a list of strings;
    /// For `Replace` it is a list of strings containing pairs in format:
    /// `old_project+new_project`
    pub projects: Vec<String>,
    /// Type of operation applied to projects
    pub project_act: Action,
    /// List of contexts.
    /// For `Set` and `Delete` is a list of strings;
    /// For `Replace` it is a list of strings containing pairs in format:
    /// `old_context@new_context`
    pub contexts: Vec<String>,
    /// Type of operation applied to contexts
    pub context_act: Action,
    /// Automatically set creation date to today if it is not defined in subject
    /// when adding a new todo
    pub auto_create_date: bool,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            subject: None,
            done: true,
            priority: NO_PRIORITY,
            priority_act: Action::None,
            due: None,
            due_act: Action::None,
            thr: None,
            thr_act: Action::None,
            recurrence: None,
            recurrence_act: Action::None,
            projects: Vec::new(),
            project_act: Action::None,
            contexts: Vec::new(),
            context_act: Action::None,
            auto_create_date: false,
        }
    }
}

pub(crate) fn make_id_vec(sz: usize) -> IDVec {
    let mut v: IDVec = Vec::new();
    for i in 0..sz {
        v.push(i);
    }
    v
}

/// Load a list of todo from a file in todo.txt format. If the file does not
/// exist or cannot be opened the function returns empty list
pub fn load(filename: &Path) -> Result<TaskVec, terr::TodoError> {
    let mut tasks = Vec::new();
    if !filename.exists() {
        return Ok(tasks);
    }

    let file = File::open(filename).context(terr::TodoErrorKind::LoadFailed)?;

    let br = BufReader::new(&file);
    for l in br.lines() {
        if let Ok(line) = l {
            if let Ok(t) = todo_txt::task::Extended::from_str(&line) {
                tasks.push(t);
            }
        }
    }

    Ok(tasks)
}

/// Saves the list of todos into a local file. Returns an error if saving
/// fails.
pub fn save(tasks: &TaskSlice, filename: &Path) -> Result<(), terr::TodoError> {
    let tmpname = filename.with_extension(OsStr::new("todo.tmp"));

    let mut output = File::create(&tmpname).context(terr::TodoErrorKind::SaveFailed)?;
    for t in tasks {
        let line = format!("{}\n", t);
        if write!(output, "{}", line).is_err() {
            return Err(terr::TodoError::from(terr::TodoErrorKind::SaveFailed));
        }
    }

    if fs::rename(tmpname, filename).is_err() {
        return Err(terr::TodoError::from(terr::TodoErrorKind::SaveFailed));
    }

    Ok(())
}

/// Appends todos to a file. If file does not exist it is created.
///
/// * `tasks` - todo list to append to the file
/// * `filename` - the name of the file to save the data (usually it is `done.txt`)
///
/// Returns true if all todos are saved successfully
pub fn archive(tasks: &TaskSlice, filename: &Path) -> Result<(), terr::TodoError> {
    let mut output = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&filename)
        .context(terr::TodoErrorKind::AppendFailed)?;

    for t in tasks {
        let line = format!("{}\n", t);
        write!(output, "{}", line).context(terr::TodoErrorKind::FileWriteFailed)?;
    }

    Ok(())
}

/// Makes a clones of selected todos
///
/// * `tasks` - the full list of todos
/// * `ids` - the list of todo IDs that must be clones. Invalid IDs (e.g, ID
///     greater than the number of items in `tasks`) are skipped
///
/// Returns the list of cloned todos. Size of the list is equal to or less than
/// size of `ids` vector
pub fn clone_tasks(tasks: &TaskSlice, ids: &IDSlice) -> TaskVec {
    // TODO: clone all if ids is empty?
    let mut v: TaskVec = Vec::new();
    for id in ids.iter() {
        if *id < tasks.len() {
            let t = tasks[*id].clone();
            v.push(t);
        }
    }
    v
}

/// Appends a new todo to todo list
///
/// * `tasks` - a list of todos for adding a new item
/// * `c` - information about new todo. At this moment only `subject` field
///     is used - it should contains all info including due date, priority etc.
///     The `subject` field should be in todo.txt format
///
/// Returns:
/// * INVALID_ID if the subject is empty or cannot be parsed as todo.txt entry
/// * id of the new todo
pub fn add(tasks: &mut TaskVec, c: &Conf) -> usize {
    let s = match &c.subject {
        None => return INVALID_ID,
        Some(subj) => subj,
    };

    match todo_txt::task::Extended::from_str(s) {
        Ok(mut t) => {
            if c.auto_create_date && t.create_date.is_none() {
                t.create_date = Some(chrono::Local::now().date().naive_local());
            }
            tasks.push(t);
            tasks.len() - 1
        }
        Err(_) => INVALID_ID,
    }
}

fn done_undone(tasks: &mut TaskVec, ids: Option<&IDVec>, c: &Conf) -> ChangedVec {
    if tasks.is_empty() {
        return Vec::new();
    }
    let longvec = make_id_vec(tasks.len());
    let id_iter = if let Some(v) = ids { v } else { &longvec };
    let mut bools = vec![false; id_iter.len()];
    let now = chrono::Local::now().date().naive_local();

    for (i, idx) in id_iter.iter().enumerate() {
        if *idx >= tasks.len() {
            continue;
        }

        if c.done {
            bools[i] = timer::stop_timer(&mut tasks[*idx]);
            if let (Some(rr), Some(dd)) = (&tasks[*idx].recurrence, &tasks[*idx].due_date) {
                let td = tasks[*idx].threshold_date;
                let mut cnt = 0;
                let rd = rr.clone();
                let mut new_due = *dd;
                while cnt == 0 || new_due <= now {
                    new_due = rr.clone() + new_due;
                    cnt += 1;
                }
                tasks[*idx].due_date = Some(new_due);
                bools[i] = true;
                if let Some(dh) = &td {
                    let mut new_thr = *dh;
                    for _i in 0..cnt {
                        new_thr = rd.clone() + new_thr;
                    }
                    tasks[*idx].threshold_date = Some(new_thr);
                }
            } else if let (Some(rr), Some(dh)) = (&tasks[*idx].recurrence, &tasks[*idx].threshold_date) {
                tasks[*idx].threshold_date = Some(rr.clone() + *dh);
                bools[i] = true;
            } else if !tasks[*idx].finished {
                tasks[*idx].complete();
                bools[i] = true;
            }
        } else if tasks[*idx].finished {
            tasks[*idx].uncomplete();
            bools[i] = true;
        }
    }

    bools
}

/// Marks todos completed.
///
/// It works differently for regular and recurrent ones.
/// If a todo is a regular one and is not done yet, the function sets flag
/// `done` and marks the todo as modified.
/// If a todo is a recurrent one, the function pushes its due date to the next
/// date (current due increased by recurrence value). Flag `done` is not set
/// but the todo is marked modified.
///
/// * `tasks` - the task list
/// * `ids` - the list of todo IDs which should be completed. If it is `None`
///     the entire task list is marked completed
///
/// Returns a list of boolean values: a value per each ID in `ids` or `tasks`.
/// The length of the result list equals either length of `ids`(if `ids` is
/// `Some`) or  length of `tasks`(if `ids` is `None`). Value `true` in this
/// array means that corresponding item from `ids` or `tasks` was modified.
pub fn done(tasks: &mut TaskVec, ids: Option<&IDVec>) -> ChangedVec {
    let c = Conf {
        done: true,
        ..Default::default()
    };
    done_undone(tasks, ids, &c)
}

/// Removes flag `done` from todos.
///
/// * `tasks` - the task list
/// * `ids` - the list of todo IDs which should be undone. If it is `None`
///     the entire task list is marked undone.
///
/// Returns a list of boolean values: a value per each ID in `ids` or `tasks`.
/// The length of the result list equals either length of `ids`(if `ids` is
/// `Some`) or  length of `tasks`(if `ids` is `None`). Value `true` in this
/// array means that corresponding item from `ids` or `tasks` was modified.
pub fn undone(tasks: &mut TaskVec, ids: Option<&IDVec>) -> ChangedVec {
    let c = Conf {
        done: false,
        ..Default::default()
    };
    done_undone(tasks, ids, &c)
}

/// Removes todos from the list
///
/// * `tasks` - the task list
/// * `ids` - the list of todo IDs which should be removed. If it is `None`
///     the task list is cleared.
///
/// Returns a list of boolean values: a value per each ID in `ids` or `tasks`.
/// The length of the result list equals either length of `ids`(if `ids` is
/// `Some`) or  length of `tasks`(if `ids` is `None`). Value `true` in this
/// array means that corresponding item from `ids` or `tasks` was modified.
/// Note: all items in `ChangedVec` are always `true`.
pub fn remove(tasks: &mut TaskVec, ids: Option<&IDVec>) -> ChangedVec {
    if tasks.is_empty() {
        return vec![];
    }
    let longvec = make_id_vec(tasks.len());
    let idlist = if let Some(v) = ids { v } else { &longvec };
    let mut bools = vec![false; idlist.len()];

    for (i, id) in idlist.iter().enumerate() {
        if *id < tasks.len() {
            bools[i] = true;
        }
    }

    let mut remained: TaskVec = Vec::new();
    for (i, t) in tasks.iter().enumerate() {
        let mut found: bool = false;
        for id in idlist.iter() {
            if *id == i {
                found = true;
                break;
            }
        }
        if !found {
            remained.push(t.clone());
        }
    }

    std::mem::swap(tasks, &mut remained);
    bools
}

fn update_priority(task: &mut todo_txt::task::Extended, c: &Conf) -> bool {
    match c.priority_act {
        Action::Set => {
            if task.priority != c.priority {
                task.priority = c.priority;
                return true;
            }
        }
        Action::Delete => {
            if task.priority != NO_PRIORITY {
                task.priority = NO_PRIORITY;
                return true;
            }
        }
        Action::Increase => {
            if task.priority != 0 {
                task.priority -= 1u8;
                return true;
            }
        }
        Action::Decrease => {
            if task.priority != NO_PRIORITY {
                task.priority += 1u8;
                return true;
            }
        }
        _ => {}
    }

    false
}

fn update_due_date(task: &mut todo_txt::task::Extended, c: &Conf) -> bool {
    match c.due_act {
        Action::Set => {
            if tsort::cmp_opt_dates(task.due_date, c.due) != Ordering::Equal {
                task.due_date = c.due;
                return true;
            }
        }
        Action::Delete => {
            if task.due_date.is_some() {
                task.due_date = None;
                return true;
            }
        }
        _ => {}
    }

    false
}

fn update_thr_date(task: &mut todo_txt::task::Extended, c: &Conf) -> bool {
    match c.thr_act {
        Action::Set => {
            if tsort::cmp_opt_dates(task.threshold_date, c.thr) != Ordering::Equal {
                task.threshold_date = c.thr;
                return true;
            }
        }
        Action::Delete => {
            if task.threshold_date.is_some() {
                task.threshold_date = None;
                return true;
            }
        }
        _ => {}
    }

    false
}

fn update_recurrence(task: &mut todo_txt::task::Extended, c: &Conf) -> bool {
    match c.recurrence_act {
        Action::Set => {
            if !tsort::equal_opt_rec(&task.recurrence, &c.recurrence) {
                task.recurrence = c.recurrence.clone();
                if let Some(nr) = &c.recurrence {
                    let new_rec = format!("{}", nr);
                    if let Some(subj) = replace_tag(&task.subject, "rec:", &new_rec) {
                        task.subject = subj;
                        if task.finished {
                            task.uncomplete();
                        }
                    }
                    return true;
                }
            }
        }
        Action::Delete => {
            if task.recurrence.is_some() {
                task.recurrence = None;
                if let (Some(_), Some(subj)) = remove_tag(&task.subject, "rec:") {
                    task.subject = subj;
                }
                return true;
            }
        }
        _ => {}
    }

    false
}

fn item_in_list(list: &[String], val: &str) -> (Option<String>, Option<usize>) {
    for (idx, ll) in list.iter().enumerate() {
        if default_caseless_match_str(ll, val) {
            return (Some(ll.to_string()), Some(idx));
        }
    }

    (None, None)
}

fn update_projects(task: &mut todo_txt::task::Extended, c: &Conf) -> bool {
    let mut changed = false;

    for new_p in &c.projects {
        match c.project_act {
            Action::Set => {
                let (s, _) = item_in_list(&task.projects, &new_p);
                if s.is_none() {
                    task.subject.push_str(" +");
                    task.subject.push_str(new_p);
                    task.projects.push(new_p.to_string());
                    changed = true;
                }
            }
            Action::Delete => {
                if let (Some(prj), Some(pos)) = item_in_list(&task.projects, &new_p) {
                    let prj = "+".to_string() + &prj;
                    if let Some(new_subj) = remove_proj_ctx(&task.subject, &prj) {
                        task.subject = new_subj;
                        task.projects.remove(pos);
                        changed = true;
                    }
                }
            }
            Action::Replace => {
                let pair: Vec<&str> = new_p.split_terminator('+').collect();
                if pair.len() == 2 && pair[0] != pair[1] && !pair[0].is_empty() && !pair[1].is_empty() {
                    if let (Some(prj), Some(pos)) = item_in_list(&task.projects, &pair[0]) {
                        let prj = "+".to_string() + &prj;
                        let new_prj = "+".to_string() + pair[1];
                        if let Some(new_subj) = replace_proj_ctx(&task.subject, &prj, &new_prj) {
                            task.subject = new_subj;
                            task.projects[pos] = new_prj;
                            changed = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    changed
}

fn update_contexts(task: &mut todo_txt::task::Extended, c: &Conf) -> bool {
    let mut changed = false;

    for new_c in &c.contexts {
        match c.context_act {
            Action::Set => {
                let (s, _) = item_in_list(&task.contexts, &new_c);
                if s.is_none() {
                    task.subject.push_str(" @");
                    task.subject.push_str(new_c);
                    task.contexts.push(new_c.to_string());
                    changed = true;
                }
            }
            Action::Delete => {
                if let (Some(ctx), Some(pos)) = item_in_list(&task.contexts, &new_c) {
                    let ctx = "@".to_string() + &ctx;
                    if let Some(new_subj) = remove_proj_ctx(&task.subject, &ctx) {
                        task.subject = new_subj;
                        task.contexts.remove(pos);
                        changed = true;
                    }
                }
            }
            Action::Replace => {
                let pair: Vec<&str> = new_c.split_terminator('@').collect();
                if pair.len() == 2 && pair[0] != pair[1] && !pair[0].is_empty() && !pair[1].is_empty() {
                    if let (Some(ctx), Some(pos)) = item_in_list(&task.contexts, &pair[0]) {
                        let ctx = "@".to_string() + &ctx;
                        let new_ctx = "@".to_string() + pair[1];
                        if let Some(new_subj) = replace_proj_ctx(&task.subject, &ctx, &new_ctx) {
                            task.subject = new_subj;
                            task.contexts[pos] = new_ctx;
                            changed = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    changed
}

/// Modifies existing todos.
///
/// A powerful function to transform todo list. List of operations:
/// - for priority: change, remove, increase or decrease by one;
/// - for subject: set a new subject;
/// - for due date: remove or set a new one;
/// - for recurrence: remove or set a new one (setting recurrence clears
///     `done` flag;
/// - for project: add a new, remove old, replace old with new one;
/// - for context: add a new, remove old, replace old with new one;
///
/// * `tasks` - the task list
/// * `ids` - the list of todo IDs which should be undone. If it is `None`
///     the entire task list is marked undone.
/// * `c` - what to modify and how
///
/// if `c` contains a new subject then only the first todo from `ids` is
/// modified. For all other cases the function processes all todos from `ids`.
///
/// Returns a list of boolean values: a value per each ID in `ids` or `tasks`.
/// The length of the result list equals either length of `ids`(if `ids` is
/// `Some`) or  length of `tasks`(if `ids` is `None`). Value `true` in this
/// array means that corresponding item from `ids` or `tasks` was modified.
pub fn edit(tasks: &mut TaskVec, ids: Option<&IDVec>, c: &Conf) -> ChangedVec {
    if tasks.is_empty() {
        return vec![];
    }

    let longvec = make_id_vec(tasks.len());
    let idlist = if let Some(v) = ids { v } else { &longvec };

    let mut bools = vec![false; idlist.len()];
    for (i, idx) in idlist.iter().enumerate() {
        let id = *idx;
        if id >= tasks.len() {
            continue;
        }

        if let Some(subj) = c.subject.as_ref() {
            if let Ok(mut t) = todo_txt::task::Extended::from_str(subj) {
                if t.create_date.is_none() && tasks[id].create_date.is_some() {
                    t.create_date = tasks[id].create_date;
                }
                tasks[id] = t;
                bools[i] = true;
            }
            // it does not make sense to replace more than 1 todo's subject
            // with the same text. So, replace for the first one and stop
            break;
        }

        bools[i] = update_priority(&mut tasks[id], c);
        bools[i] |= update_due_date(&mut tasks[id], c);
        bools[i] |= update_thr_date(&mut tasks[id], c);
        bools[i] |= update_recurrence(&mut tasks[id], c);
        bools[i] |= update_projects(&mut tasks[id], c);
        bools[i] |= update_contexts(&mut tasks[id], c);
    }

    bools
}

fn replace_proj_ctx(orig: &str, old: &str, new: &str) -> Option<String> {
    if old == new {
        return None;
    }

    if !old.starts_with('@') && !old.starts_with('+') {
        return None;
    }

    let mut new_s = String::new();

    let slices = orig.split_whitespace();
    let mut changed = false;
    for s in slices {
        if default_caseless_match_str(s, old) {
            changed = true;
            new_s.push_str(new);
            new_s.push(' ');
            continue;
        }

        new_s.push_str(s);
        new_s.push(' ');
    }

    if changed {
        new_s = new_s.trim_matches(' ').to_string();
        return Some(new_s);
    }

    None
}

fn replace_tag(orig: &str, old: &str, new: &str) -> Option<String> {
    if old == new {
        return None;
    }

    let (opt_pos, opt_s) = remove_tag(orig, old);
    let p = match opt_pos {
        None => return None,
        Some(pos) => pos,
    };
    let mut s = match opt_s {
        None => return None,
        Some(st) => st,
    };

    let mut new_s = new.to_string();
    if p != 0 {
        new_s = " ".to_string() + &new_s;
    } else if !s.is_empty() {
        new_s.push(' ');
    }
    s.insert_str(p, &new_s);
    Some(s)
}

fn remove_proj_ctx(orig: &str, patt: &str) -> Option<String> {
    if orig == patt {
        return None;
    }

    if !patt.starts_with('@') && !patt.starts_with('+') {
        return None;
    }

    // must be full match: project or context
    let mut new_s = String::new();

    let slices = orig.split_whitespace();
    let mut changed = false;
    for s in slices {
        if default_caseless_match_str(s, patt) {
            changed = true;
            continue;
        }

        new_s.push_str(s);
        new_s.push(' ');
    }

    if changed {
        new_s = new_s.trim_matches(' ').to_string();
        return Some(new_s);
    }

    None
}

fn remove_tag(orig: &str, patt: &str) -> (Option<usize>, Option<String>) {
    if orig == patt {
        return (Some(0), Some(String::new()));
    }

    // partial match: custom tag with value
    if orig.starts_with(patt) {
        match orig.find(' ') {
            None => return (Some(0), Some(String::new())),
            Some(pos) => return (Some(0), Some(orig[pos + 1..].to_string())),
        }
    }

    let patts = " ".to_string() + patt;
    let start = if let Some(p) = orig.find(&patts) {
        p
    } else {
        return (None, None);
    };
    let new_orig = &orig[start + 1..];
    let new_str = match new_orig.find(' ') {
        None => orig[..start].to_string(),
        Some(end) => orig[..start].to_string() + &new_orig[end..],
    };
    (Some(start), Some(new_str))
}

/// Starts timers of all toods that are not done
pub fn start(tasks: &mut TaskVec, ids: Option<&IDVec>) -> ChangedVec {
    if tasks.is_empty() {
        return vec![];
    }

    let longvec = make_id_vec(tasks.len());
    let idlist = if let Some(v) = ids { v } else { &longvec };

    let mut bools = vec![false; idlist.len()];
    for (i, idx) in idlist.iter().enumerate() {
        let id = *idx;
        if id >= tasks.len() {
            continue;
        }

        bools[i] = timer::start_timer(&mut tasks[id]);
    }

    bools
}

/// Stops timers of all toods that are running
pub fn stop(tasks: &mut TaskVec, ids: Option<&IDVec>) -> ChangedVec {
    if tasks.is_empty() {
        return vec![];
    }

    let longvec = make_id_vec(tasks.len());
    let idlist = if let Some(v) = ids { v } else { &longvec };

    let mut bools = vec![false; idlist.len()];
    for (i, idx) in idlist.iter().enumerate() {
        let id = *idx;
        if id >= tasks.len() {
            continue;
        }

        bools[i] = timer::stop_timer(&mut tasks[id]);
    }

    bools
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn remove_tags() {
        let new_str = remove_proj_ctx("", "@tag");
        assert!(new_str.is_none());
        let new_str = remove_proj_ctx("str abc", "@tag");
        assert!(new_str.is_none());
        let new_str = remove_proj_ctx("str @tag1 abc", "@tag");
        assert!(new_str.is_none());
        let new_str = remove_proj_ctx("str abc some@tag example", "@tag");
        assert!(new_str.is_none());
        let new_str = remove_proj_ctx("str abc", "rec:");
        assert!(new_str.is_none());
        let new_str = remove_proj_ctx("str abc somerec:11 example", "rec:");
        assert!(new_str.is_none());

        let new_str = remove_proj_ctx("@tag str abc @tag1", "@tag");
        assert_eq!(new_str, Some("str abc @tag1".to_string()));
        let new_str = remove_proj_ctx("str @tag abc", "@tag");
        assert_eq!(new_str, Some("str abc".to_string()));
        let new_str = remove_proj_ctx("str abc @tag", "@tag");
        assert_eq!(new_str, Some("str abc".to_string()));
        let new_str = remove_proj_ctx("efg @@tag str abc @tag", "@tag");
        assert_eq!(new_str, Some("efg @@tag str abc".to_string()));

        let (pos, new_str) = remove_tag("rec:", "rec:");
        assert_eq!(new_str, Some("".to_string()));
        assert_eq!(pos, Some(0));
        let (pos, new_str) = remove_tag("rec:11", "rec:");
        assert_eq!(new_str, Some("".to_string()));
        assert_eq!(pos, Some(0));
        let (pos, new_str) = remove_tag("str rec:22 abc", "rec:");
        assert_eq!(new_str, Some("str abc".to_string()));
        assert_eq!(pos, Some(3));
        let (pos, new_str) = remove_tag("str abc rec:22", "rec:");
        assert_eq!(new_str, Some("str abc".to_string()));
        assert_eq!(pos, Some(7));
        let (pos, new_str) = remove_tag("rec:44 str abc rec:55", "rec:");
        assert_eq!(new_str, Some("str abc rec:55".to_string()));
        assert_eq!(pos, Some(0));
        let (pos, new_str) = remove_tag("efg rrec:44 str abc rec:55", "rec:");
        assert_eq!(new_str, Some("efg rrec:44 str abc".to_string()));
        assert_eq!(pos, Some(19));
    }

    #[test]
    fn replace_tags() {
        let new_str = replace_proj_ctx("", "@tag", "@tg");
        assert!(new_str.is_none());
        let new_str = replace_proj_ctx("str abc", "@tag", "@tg");
        assert!(new_str.is_none());
        let new_str = replace_proj_ctx("str @tag1 abc", "@tag", "@tg");
        assert!(new_str.is_none());
        let new_str = replace_proj_ctx("str some@tag abc", "@tag", "@tg");
        assert!(new_str.is_none());
        let new_str = replace_proj_ctx("str abc", "rec:", "rec:12");
        assert!(new_str.is_none());
        let new_str = replace_proj_ctx("str somrec:45 abc", "rec:", "rec:12");
        assert!(new_str.is_none());

        let new_str = replace_proj_ctx("@tag str abc @tag1", "@tag", "@newstr");
        assert_eq!(new_str, Some("@newstr str abc @tag1".to_string()));
        let new_str = replace_proj_ctx("str @tag abc", "@tag", "@newstr");
        assert_eq!(new_str, Some("str @newstr abc".to_string()));
        let new_str = replace_proj_ctx("str abc @tag", "@tag", "@newstr");
        assert_eq!(new_str, Some("str abc @newstr".to_string()));
        let new_str = replace_proj_ctx("efg @@tag str abc @tag", "@tag", "@newstr");
        assert_eq!(new_str, Some("efg @@tag str abc @newstr".to_string()));

        let new_str = replace_tag("rec:", "rec:", "rec:345");
        assert_eq!(new_str, Some("rec:345".to_string()));
        let new_str = replace_tag("rec:11", "rec:", "rec:345");
        assert_eq!(new_str, Some("rec:345".to_string()));
        let new_str = replace_tag("str rec:22 abc", "rec:", "rec:345");
        assert_eq!(new_str, Some("str rec:345 abc".to_string()));
        let new_str = replace_tag("str abc rec:22", "rec:", "rec:345");
        assert_eq!(new_str, Some("str abc rec:345".to_string()));
        let new_str = replace_tag("rec:44 str abc rec:55", "rec:", "rec:345");
        assert_eq!(new_str, Some("rec:345 str abc rec:55".to_string()));
        let new_str = replace_tag("efg rrec:44 str abc rec:55", "rec:", "rec:345");
        assert_eq!(new_str, Some("efg rrec:44 str abc rec:345".to_string()));
    }
}
