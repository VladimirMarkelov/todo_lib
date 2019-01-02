use std::cmp::Ordering;

use crate::todo;
use todo_txt;

/// Sorting rules. First, the list of todos is sorted by the fields defined
/// in `fields` in order of appearance. Then, if `rev` is `true` the list is
/// reversed
#[derive(Debug, Clone)]
pub struct Conf {
    /// comma separated list of field to sort by. Supported field names:
    /// * `pri` or `prioroty` - sort by priority (without priority are the last ones);
    /// * `due` - sor by due date (todos that do not have due date are at the bottom);
    /// * `thr` - sor by threshold date (todos that do not have threshold date are at the bottom);
    /// * `completed` or `finished` - sort by completion date (incomplete ones are at the bottom);
    /// * `created` or `create` - sort by creation date;
    /// * `subject`, `subj` or `text` - sort by todo's subjects;
    /// * `done` - order: incomplete, recurrent, and done todos;
    /// * `project` or `proj` - sort by project names, if todos have more than one project they are compared in order of appearance and shorter list of projects goes first;
    /// * `context` or `ctx` - sort by contexts, if todos have more than one context they are compared in order of appearance and shorter list of contexts goes first;
    pub fields: Option<String>,
    /// reverse the list after sorting
    pub rev: bool,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            fields: None,
            rev: false,
        }
    }
}

pub(crate) fn cmp_opt_dates(d1: Option<todo_txt::Date>, d2: Option<todo_txt::Date>) -> Ordering {
    match (&d1, &d2) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(v1), Some(v2)) => v1.cmp(v2),
    }
}

pub(crate) fn equal_opt_rec(r1: &Option<todo_txt::task::Recurrence>, r2: &Option<todo_txt::task::Recurrence>) -> bool {
    match (&r1, &r2) {
        (None, None) => true,
        (Some(_), None) | (None, Some(_)) => false,
        (Some(v1), Some(v2)) => v1 == v2,
    }
}

fn cmp_opt_arrays(a1: &[String], a2: &[String]) -> Ordering {
    if a1.is_empty() && !a2.is_empty() {
        return Ordering::Greater;
    } else if !a1.is_empty() && a2.is_empty() {
        return Ordering::Less;
    } else if a1.is_empty() && a2.is_empty() {
        return Ordering::Equal;
    }

    let max = if a1.len() > a2.len() { a2.len() } else { a1.len() };

    let mut ord = Ordering::Equal;
    for idx in 0..max {
        let s1_low = a1[idx].to_lowercase();
        let s2_low = a2[idx].to_lowercase();
        ord = s1_low.cmp(&s2_low);
        if ord != Ordering::Equal {
            break;
        }
    }

    if ord == Ordering::Equal {
        ord = a1.len().cmp(&a2.len())
    }

    ord
}

/// The main entry for the todo list sorting.
///
/// The function sorts the provided list of todo IDs `ids` that is generated
/// by filtering function or manually created. To compare todos, the function
/// needs the entire list of them `todos`.
/// The sorting is stable. All non-existing IDs are moved to the end.
///
/// * `ids` - the list of todo IDs to sort
/// * `todos` - the list of all todos
/// * `c` - sorting rules
pub fn sort(ids: &mut todo::IDVec, todos: &todo::TaskSlice, c: &Conf) {
    if c.fields.is_none() && !c.rev {
        return;
    }

    let low: String;
    let fields: Vec<&str> = match &c.fields {
        None => Vec::new(),
        Some(v) => {
            low = v.to_lowercase();
            low.split(|c: char| c == ',' || c == ':').collect()
        }
    };

    if !fields.is_empty() {
        ids.sort_by(|a, b| {
            if *a >= todos.len() && *b >= todos.len() {
                return Ordering::Equal;
            } else if *a >= todos.len() {
                return Ordering::Greater;
            } else if *b >= todos.len() {
                return Ordering::Less;
            }

            let mut res: Ordering = Ordering::Equal;
            for f in &fields {
                res = match *f {
                    "pri" | "priority" => todos[*a].priority.cmp(&todos[*b].priority),
                    "due" => cmp_opt_dates(todos[*a].due_date, todos[*b].due_date),
                    "thr" => cmp_opt_dates(todos[*a].threshold_date, todos[*b].threshold_date),
                    "completed" | "finished" => cmp_opt_dates(todos[*a].finish_date, todos[*b].finish_date),
                    "created" | "create" => cmp_opt_dates(todos[*a].create_date, todos[*b].create_date),
                    "subject" | "text" | "subj" => todos[*a].subject.cmp(&todos[*b].subject),
                    "done" => {
                        let f1 = if todos[*a].recurrence.is_some() {
                            1
                        } else if todos[*a].finished {
                            2
                        } else {
                            0
                        };
                        let f2 = if todos[*b].recurrence.is_some() {
                            1
                        } else if todos[*b].finished {
                            2
                        } else {
                            0
                        };
                        f1.cmp(&f2)
                    }
                    "proj" | "project" => cmp_opt_arrays(&todos[*a].projects, &todos[*b].projects),
                    "ctx" | "context" => cmp_opt_arrays(&todos[*a].contexts, &todos[*b].contexts),
                    // "active" => {
                    //     let a_act = if let Some(state) = todos[*a].tags.get(todo::TIMER_TAG) {
                    //         state != todo::TIMER_OFF
                    //     } else {
                    //         false
                    //     };
                    //     let b_act = if let Some(state) = todos[*b].tags.get(todo::TIMER_TAG) {
                    //         state != todo::TIMER_OFF
                    //     } else {
                    //         false
                    //     };
                    //     b_act.cmp(&a_act)
                    // },
                    _ => Ordering::Equal,
                };

                if res != Ordering::Equal {
                    break;
                }
            }

            res
        });
    }

    if c.rev {
        ids.reverse();
    }
}
