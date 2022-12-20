use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

use crate::terr;
use crate::timer;
use crate::todotxt;
use crate::tsort;

/// The ID value returned instead of new todo ID if adding a new todo fails
pub const INVALID_ID: usize = 9_999_999_999;
pub const TIMER_TAG: &str = "tmr";
pub const SPENT_TAG: &str = "spent";
pub const TIMER_OFF: &str = "off";

pub type TaskVec = Vec<todotxt::Task>;
pub type TaskSlice = [todotxt::Task];
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
    pub recurrence: Option<todotxt::Recurrence>,
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
            priority: todotxt::NO_PRIORITY,
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

    let file = File::open(filename).map_err(|_| terr::TodoError::LoadFailed)?;
    let now = chrono::Local::now().date_naive();

    let br = BufReader::new(&file);
    for l in br.lines().flatten() {
        let t = todotxt::Task::parse(&l, now);
        tasks.push(t);
    }

    Ok(tasks)
}

/// Saves the list of todos into a local file. Returns an error if saving
/// fails.
pub fn save(tasks: &TaskSlice, filename: &Path) -> Result<(), terr::TodoError> {
    let tmpname = filename.with_extension(OsStr::new("todo.tmp"));

    let mut output = File::create(&tmpname).map_err(|_| terr::TodoError::SaveFailed)?;
    for t in tasks {
        let line = format!("{}\n", t);
        write!(output, "{}", line).map_err(|_| terr::TodoError::FileWriteFailed)?;
    }

    fs::rename(tmpname, filename).map_err(|e| terr::TodoError::IOError(e.to_string()))?;
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
        .open(filename)
        .map_err(|_| terr::TodoError::AppendFailed)?;

    for t in tasks {
        let line = format!("{}\n", t);
        write!(output, "{}", line).map_err(|_| terr::TodoError::FileWriteFailed)?;
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

    let now = chrono::Local::now().date_naive();
    let mut t = todotxt::Task::parse(s, now);
    if c.auto_create_date && t.create_date.is_none() {
        t.create_date = Some(chrono::Local::now().date_naive());
    }
    tasks.push(t);
    tasks.len() - 1
}

fn done_undone(tasks: &mut TaskVec, ids: Option<&IDVec>, c: &Conf) -> ChangedVec {
    if tasks.is_empty() {
        return Vec::new();
    }
    let longvec = make_id_vec(tasks.len());
    let id_iter = if let Some(v) = ids { v } else { &longvec };
    let mut bools = vec![false; id_iter.len()];
    let now = chrono::Local::now().date_naive();

    for (i, idx) in id_iter.iter().enumerate() {
        if *idx >= tasks.len() {
            continue;
        }

        if c.done {
            bools[i] = timer::stop_timer(&mut tasks[*idx]);
            let mut next_task = (tasks[*idx]).clone();
            let completed = tasks[*idx].complete(now);
            if completed
                && next_task.recurrence.is_some()
                && (next_task.due_date.is_some() || next_task.threshold_date.is_some())
            {
                if next_task.create_date.is_some() {
                    next_task.create_date = Some(now);
                }
                next_task.next_dates(now);
                tasks.push(next_task);
            }
            bools[i] = bools[i] || completed;
        } else {
            bools[i] = tasks[*idx].uncomplete();
        }
    }

    bools
}

/// Marks todos completed.
///
/// It works differently for regular and recurrent ones.
/// If a todo is a regular one and is not done yet, the function sets flag
/// `done` and marks the todo as modified.
/// If a todo is a recurrent one and any of due and threshold dates exist,
/// the function marks the current task done and appends a new task with
/// changed due and threshold dates (current values increased by recurrence value).
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
    let c = Conf { done: true, ..Default::default() };
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
    let c = Conf { done: false, ..Default::default() };
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

fn update_priority(task: &mut todotxt::Task, c: &Conf) -> bool {
    match c.priority_act {
        Action::Set => {
            if task.priority != c.priority {
                task.priority = c.priority;
                return true;
            }
        }
        Action::Delete => {
            if task.priority != todotxt::NO_PRIORITY {
                task.priority = todotxt::NO_PRIORITY;
                return true;
            }
        }
        Action::Increase => {
            if task.priority != 0 && task.priority != todotxt::NO_PRIORITY {
                task.priority -= 1u8;
                return true;
            }
        }
        Action::Decrease => {
            if task.priority != todotxt::NO_PRIORITY {
                task.priority += 1u8;
                return true;
            }
        }
        _ => {}
    }

    false
}

fn update_due_date(task: &mut todotxt::Task, c: &Conf) -> bool {
    match c.due_act {
        Action::Set => {
            if tsort::cmp_opt_dates(task.due_date, c.due) != Ordering::Equal {
                match c.due {
                    None => task.update_tag_with_value(todotxt::DUE_TAG, ""),
                    Some(dt) => task.update_tag_with_value(todotxt::DUE_TAG, &todotxt::format_date(dt)),
                };
                return true;
            }
        }
        Action::Delete => {
            if task.due_date.is_some() {
                task.update_tag_with_value(todotxt::DUE_TAG, "");
                return true;
            }
        }
        _ => {}
    }

    false
}

fn update_thr_date(task: &mut todotxt::Task, c: &Conf) -> bool {
    match c.thr_act {
        Action::Set => {
            if tsort::cmp_opt_dates(task.threshold_date, c.thr) != Ordering::Equal {
                match c.thr {
                    None => task.update_tag_with_value(todotxt::THR_TAG, ""),
                    Some(dt) => task.update_tag_with_value(todotxt::THR_TAG, &todotxt::format_date(dt)),
                };
                return true;
            }
        }
        Action::Delete => {
            if task.threshold_date.is_some() {
                task.update_tag_with_value(todotxt::THR_TAG, "");
                return true;
            }
        }
        _ => {}
    }

    false
}

fn update_recurrence(task: &mut todotxt::Task, c: &Conf) -> bool {
    match c.recurrence_act {
        Action::Set => {
            if !tsort::equal_opt_rec(&task.recurrence, &c.recurrence) {
                if let Some(nr) = &c.recurrence {
                    let new_rec = format!("{}", nr);
                    let updated = task.update_tag(&new_rec);
                    if updated && task.finished {
                        task.uncomplete();
                    }
                    return updated;
                }
            }
        }
        Action::Delete => {
            if task.recurrence.is_some() {
                task.update_tag_with_value(todotxt::REC_TAG, "");
                return true;
            }
        }
        _ => {}
    }

    false
}

fn update_projects(task: &mut todotxt::Task, c: &Conf) -> bool {
    let mut changed = false;

    for new_p in &c.projects {
        match c.project_act {
            Action::Set => {
                let old_subj = task.subject.clone();
                task.replace_project("", new_p);
                changed = old_subj != task.subject;
            }
            Action::Delete => {
                let old_subj = task.subject.clone();
                task.replace_project(new_p, "");
                changed = old_subj != task.subject;
            }
            Action::Replace => {
                let pair: Vec<&str> = new_p.split_terminator('+').collect();
                if pair.len() == 2 && pair[0] != pair[1] && !pair[0].is_empty() && !pair[1].is_empty() {
                    let old_subj = task.subject.clone();
                    task.replace_project(pair[0], pair[1]);
                    changed = old_subj != task.subject;
                }
            }
            _ => {}
        }
    }

    changed
}

fn update_contexts(task: &mut todotxt::Task, c: &Conf) -> bool {
    let mut changed = false;

    for new_c in &c.contexts {
        match c.context_act {
            Action::Set => {
                let old_subj = task.subject.clone();
                task.replace_context("", new_c);
                changed = old_subj != task.subject;
            }
            Action::Delete => {
                let old_subj = task.subject.clone();
                task.replace_context(new_c, "");
                changed = old_subj != task.subject;
            }
            Action::Replace => {
                let pair: Vec<&str> = new_c.split_terminator('@').collect();
                if pair.len() == 2 && pair[0] != pair[1] && !pair[0].is_empty() && !pair[1].is_empty() {
                    let old_subj = task.subject.clone();
                    task.replace_context(pair[0], pair[1]);
                    changed = old_subj != task.subject;
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
    let now = chrono::Local::now().date_naive();

    let mut bools = vec![false; idlist.len()];
    for (i, idx) in idlist.iter().enumerate() {
        let id = *idx;
        if id >= tasks.len() {
            continue;
        }

        if let Some(subj) = c.subject.as_ref() {
            let mut t = todotxt::Task::parse(subj, now);
            if t.create_date.is_none() && tasks[id].create_date.is_some() {
                t.create_date = tasks[id].create_date;
            }
            tasks[id] = t;
            bools[i] = true;
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
