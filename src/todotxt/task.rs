use std::collections::HashMap;

use chrono::{Local, NaiveDate};

use crate::todotxt::utils;

const PRIORITY_TAG: &str = "pri";
const CLEANUP_CLONE_TAGS: [&str; 2] = ["tmr:", "spent:"];

/// Has options to manipulate how task information is handled when
/// transitioning task's state to completed.
pub struct CompletionConfig {
    /// What to do with priority on task completion.
    pub completion_mode: CompletionMode,
    /// How to set completion date on task completion.
    pub completion_date_mode: CompletionDateMode,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            completion_mode: CompletionMode::JustMark,
            completion_date_mode: CompletionDateMode::WhenCreationDateIsPresent,
        }
    }
}

/// What to do with priority on task completion.
/// For case `RemovePriority` it is impossible to restore the original
/// priority when taks is undone
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CompletionMode {
    /// Prepend 'x' when completed and keep priority
    JustMark,
    /// Move priority after completion date, if the task has completion date.
    /// It removed the priority from the output by making it a part of subject
    MovePriority,
    /// Erase priority - do not keep priority for completed tasks
    RemovePriority,
    /// Set priority `(A)` to None, but create a tag `pri:A`
    PriorityToTag,
}

/// How to set completion date on task completion.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CompletionDateMode {
    /// Only add completion date if task has creation date
    WhenCreationDateIsPresent,
    /// Always add completion date, regardless of whether or not creation date is present
    AlwaysSet,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Task {
    pub subject: String,
    pub priority: u8,
    pub finished: bool,
    pub contexts: Vec<String>,
    pub projects: Vec<String>,
    pub tags: HashMap<String, String>,
    pub create_date: Option<NaiveDate>,
    pub finish_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub threshold_date: Option<NaiveDate>,
    pub recurrence: Option<utils::Recurrence>,
    pub hashtags: Vec<String>,
}

impl Default for Task {
    fn default() -> Task {
        Task {
            subject: String::new(),
            priority: utils::NO_PRIORITY,
            finished: false,
            contexts: Vec::new(),
            projects: Vec::new(),
            tags: HashMap::new(),
            create_date: None,
            finish_date: None,
            due_date: None,
            threshold_date: None,
            recurrence: None,
            hashtags: Vec::new(),
        }
    }
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.finished {
            f.write_str("x ")?;
        }
        if self.priority < utils::NO_PRIORITY {
            f.write_str(&utils::format_priority(self.priority))?;
            f.write_str(" ")?;
        }
        if let Some(dt) = self.finish_date {
            f.write_str(&utils::format_date(dt))?;
            f.write_str(" ")?;
        }
        if let Some(dt) = self.create_date {
            f.write_str(&utils::format_date(dt))?;
            f.write_str(" ")?;
        }
        f.write_str(&self.subject)
    }
}

fn next_word(s: &str) -> &str {
    if s.is_empty() {
        return s;
    }
    match s.find(' ') {
        None => s,
        Some(p) => &s[..p],
    }
}

fn try_read_date(s: &str, base: NaiveDate) -> Option<NaiveDate> {
    let c = s.chars().next()?;
    if c.is_ascii_digit() {
        let dt = next_word(s);
        utils::parse_date(dt, base).ok()
    } else {
        None
    }
}

impl Task {
    fn parse_special_tags(&mut self, base: NaiveDate) {
        let mut old_tags: Vec<String> = Vec::new();
        let mut new_tags: Vec<String> = Vec::new();
        for (name, value) in &self.tags {
            if name == "rec"
                && let Ok(rec) = value.parse::<utils::Recurrence>()
            {
                self.recurrence = Some(rec);
            }
            if name == "t"
                && let Ok(dt) = utils::parse_date(value, base)
            {
                self.threshold_date = Some(dt);
                let old_tag = format!("{name}:{value}");
                let new_tag = format!("{name}:{0}", utils::format_date(dt));
                if old_tag != new_tag {
                    old_tags.push(old_tag);
                    new_tags.push(new_tag);
                }
            }
            if name == "due"
                && let Ok(dt) = utils::parse_date(value, base)
            {
                self.due_date = Some(dt);
                let old_tag = format!("{name}:{value}");
                let new_tag = format!("{name}:{0}", utils::format_date(dt));
                if old_tag != new_tag {
                    old_tags.push(old_tag);
                    new_tags.push(new_tag);
                }
            }
            if name == "until"
                && let Ok(dt) = utils::parse_date(value, base)
            {
                let old_tag = format!("{name}:{value}");
                let new_tag = format!("{name}:{0}", utils::format_date(dt));
                if old_tag != new_tag {
                    old_tags.push(old_tag);
                    new_tags.push(new_tag);
                }
            }
        }

        for (old, new) in old_tags.iter().zip(new_tags.iter()) {
            self.replace_tag(old, new);
        }
    }

    /// Coverts a string to a task.
    pub fn parse(s: &str, base: NaiveDate) -> Self {
        let mut task = Task::validate(s, base);
        task.parse_special_tags(base);
        task
    }

    fn validate(s: &str, base: NaiveDate) -> Self {
        let mut task = Task {
            finished: false,
            create_date: None,
            finish_date: None,
            threshold_date: None,
            due_date: None,
            recurrence: None,
            subject: String::new(),
            priority: utils::NO_PRIORITY,
            contexts: utils::extract_contexts(s),
            projects: utils::extract_projects(s),
            tags: utils::extract_tags(s),
            hashtags: utils::extract_hashtags(s),
        };
        let mut s = s;
        if s.starts_with("x ") {
            task.finished = true;
            s = s["x ".len()..].trim();
        }
        if s.starts_with('(') {
            let priority = next_word(s);
            match utils::parse_priority(priority) {
                Err(_) => {
                    task.subject = s.to_string();
                    return task;
                }
                Ok(p) => {
                    task.priority = p;
                    s = s[priority.len()..].trim();
                }
            }
        }
        match try_read_date(s, base) {
            None => {
                task.subject = s.to_string();
                return task;
            }
            Some(dt) => {
                if task.finished {
                    task.finish_date = Some(dt);
                } else {
                    task.create_date = Some(dt);
                }
                match s.find(' ') {
                    None => return task,
                    Some(idx) => s = s[idx + 1..].trim(),
                }
                if !task.finished {
                    task.subject = s.to_string();
                    return task;
                }
            }
        }
        match try_read_date(s, base) {
            None => task.subject = s.to_string(),
            Some(dt) => {
                task.create_date = Some(dt);
                if let Some(idx) = s.find(' ') {
                    task.subject = s[idx + 1..].trim().to_string();
                }
            }
        }
        task
    }

    fn replace_tag(&mut self, old_tag: &str, new_tag: &str) {
        utils::replace_word(&mut self.subject, old_tag, new_tag);
        if let Some((n, v)) = utils::split_tag(new_tag) {
            self.tags.insert(n.to_string(), v.to_string());
        }
    }

    /// Remove certain tags from a clone to avoid spoiling a new task with
    /// old data. Tags to remove see in `CLEANUP_CLONE_TAGS`.
    pub fn cleanup_cloned_task(&mut self) {
        for tag in CLEANUP_CLONE_TAGS {
            let _ = self.update_tag(tag);
        }
    }

    /// Replaces the tag value with a new one. If new value is empty, the tag is removed.
    /// If the tag does not exist, the function adds it to the task.
    /// Tag must be in format "name:value" or "name:"(for removing the tag).
    /// Returns true if the tag was updated.
    pub fn update_tag(&mut self, new_tag: &str) -> bool {
        let (tag, value) = if let Some(pos) = new_tag.find(':') {
            if pos == 0 {
                return false;
            }
            (&new_tag[..pos], &new_tag[pos + 1..])
        } else {
            return false;
        };
        self.update_tag_with_value(tag, value)
    }

    /// Replaces the tag value with a new one. If new value is empty, the tag is removed.
    /// If the tag does not exist, the function adds it to the task.
    /// Returns true if the tag was updated.
    pub fn update_tag_with_value(&mut self, tag: &str, value: &str) -> bool {
        if value.is_empty() {
            let old = self.tags.remove(tag);
            if let Some(v) = old {
                let old_tag = format!("{tag}:{v}");
                self.replace_tag(&old_tag, value);
                self.update_field(tag, value);
                return true;
            }
            return false;
        }
        #[allow(clippy::format_push_string)]
        match self.tags.get(tag) {
            None => {
                self.subject += &format!(" {tag}:{value}");
                self.tags.insert(tag.to_string(), value.to_string());
                self.update_field(tag, value);
                true
            }
            Some(v) => {
                if v != value {
                    let old_tag = format!("{tag}:{v}");
                    let new_tag = format!("{tag}:{value}");
                    self.replace_tag(&old_tag, &new_tag);
                    self.update_field(tag, value);
                    true
                } else {
                    false
                }
            }
        }
    }

    fn update_field(&mut self, tag: &str, value: &str) {
        match tag {
            utils::DUE_TAG => {
                if value.is_empty() {
                    self.due_date = None;
                } else if let Ok(dt) = utils::parse_date(value, Local::now().date_naive()) {
                    self.due_date = Some(dt);
                } else {
                    self.due_date = None;
                }
            }
            utils::THR_TAG => {
                if value.is_empty() {
                    self.threshold_date = None;
                } else if let Ok(dt) = utils::parse_date(value, Local::now().date_naive()) {
                    self.threshold_date = Some(dt);
                } else {
                    self.threshold_date = None;
                }
            }
            utils::REC_TAG => {
                if value.is_empty() {
                    self.recurrence = None;
                } else if let Ok(r) = value.parse::<utils::Recurrence>() {
                    self.recurrence = Some(r);
                } else {
                    self.recurrence = None;
                }
            }
            _ => {}
        }
    }

    /// Mark the task completed.
    /// Returns true if the task was changed(e.g., for a completed task the function return false).
    #[deprecated(note = "Please use `complete_with_config` - it has more stable API")]
    pub fn complete(&mut self, date: NaiveDate, cmpl: CompletionMode) -> bool {
        self.complete_with_config(date, CompletionConfig { completion_mode: cmpl, ..Default::default() })
    }

    /// Mark the task completed.
    /// Returns true if the task was changed(e.g., for a completed task the function return false).
    pub fn complete_with_config(&mut self, date: NaiveDate, cmpl_conf: CompletionConfig) -> bool {
        if self.finished {
            return false;
        }
        self.finished = true;
        if self.create_date.is_some() || cmpl_conf.completion_date_mode == CompletionDateMode::AlwaysSet {
            self.finish_date = Some(date);
        }
        match cmpl_conf.completion_mode {
            CompletionMode::RemovePriority => {
                self.priority = utils::NO_PRIORITY;
            }
            CompletionMode::PriorityToTag if self.priority < utils::NO_PRIORITY => {
                self.tags.insert(PRIORITY_TAG.to_string(), format!("{0}", utils::priority_to_char(self.priority)));
                self.subject =
                    format!("{0} {1}:{2}", self.subject, PRIORITY_TAG, utils::priority_to_char(self.priority));
                self.priority = utils::NO_PRIORITY;
            }
            CompletionMode::MovePriority if self.priority < utils::NO_PRIORITY && self.finish_date.is_some() => {
                let pri = format!("{0} ", &utils::format_priority(self.priority));
                self.subject.insert_str(0, &pri);
                self.priority = utils::NO_PRIORITY;
            }
            _ => {}
        }
        true
    }

    /// If the task has both recurrence and due or threshold date, the recurrence and due dates
    /// change so they point to some day in the future. The new values depends on
    /// recurrence strictness: for strict recurrence, the new date is always due+recurrence;
    /// for regular recurrence, the new due date is current date + recurrence.
    /// If the task has only recurrence, the task is not changed. The function does nothing if the
    /// task is already completed.
    /// Returns true if the task was changed(e.g., for a completed task the function return false).
    pub fn next_dates(&mut self, date: NaiveDate) -> bool {
        if self.finished {
            return false;
        }
        if self.due_date.is_none() && self.threshold_date.is_none() {
            return false;
        }
        let rec = match self.recurrence {
            None => return false,
            Some(r) => r,
        };
        if let Some(due) = self.due_date {
            let mut new_due = if rec.strict { rec.next_date(due) } else { rec.next_date(date) };
            while new_due < date {
                new_due = rec.next_date(new_due);
            }
            let old = format!("due:{}", utils::format_date(due));
            let new = format!("due:{}", utils::format_date(new_due));
            self.due_date = Some(new_due);
            self.replace_tag(&old, &new);
        }
        if let Some(thr) = self.threshold_date {
            let mut new_thr = if rec.strict { rec.next_date(thr) } else { rec.next_date(date) };
            while new_thr < date {
                new_thr = rec.next_date(new_thr);
            }
            let old = format!("t:{}", utils::format_date(thr));
            let new = format!("t:{}", utils::format_date(new_thr));
            self.threshold_date = Some(new_thr);
            self.replace_tag(&old, &new);
        }
        true
    }

    /// Remove completion mark from the task.
    /// Returns true if the task was changed(e.g., for a incomplete task the function return false).
    pub fn uncomplete(&mut self, cmpl: CompletionMode) -> bool {
        if !self.finished {
            return false;
        }
        match cmpl {
            CompletionMode::PriorityToTag => {
                let pri = if let Some(pri_s) = self.tags.get(PRIORITY_TAG) {
                    utils::str_to_priority(pri_s)
                } else {
                    utils::NO_PRIORITY
                };
                if pri != utils::NO_PRIORITY {
                    self.priority = pri;
                    self.tags.remove(PRIORITY_TAG);
                    utils::replace_word(
                        &mut self.subject,
                        &format!("{0}:{1}", PRIORITY_TAG, utils::priority_to_char(pri)),
                        "",
                    );
                }
            }
            CompletionMode::MovePriority => {
                // Check if the subject starts with `(?)`
                let pri_s = if let Some(idx) = self.subject.find(' ') { &self.subject[..idx] } else { &self.subject };
                if let Ok(p) = utils::parse_priority(pri_s) {
                    self.priority = p;
                    self.subject = self.subject[pri_s.len()..].trim_start().to_string();
                }
            }
            _ => {}
        }
        self.finished = false;
        self.finish_date = None;
        true
    }

    /// Replace existing project with a new one. Special cases:
    /// - new is empty: the old project is removed from the task
    /// - old is empty: the new project is appended to the task
    pub fn replace_project(&mut self, old: &str, new: &str) {
        let old = if old.starts_with('+') { &old["+".len()..] } else { old };
        let new = if new.starts_with('+') { &new["+".len()..] } else { new };
        if old.is_empty() {
            if !new.is_empty() && !self.projects.iter().any(|p| p == new) {
                self.projects.push(new.to_string());
                self.subject.push_str(" +");
                self.subject.push_str(new);
            }
            return;
        }
        self.projects.retain(|proj| proj != old);
        if !new.is_empty() {
            self.projects.push(new.to_string());
            utils::replace_word(&mut self.subject, &format!("+{old}"), &format!("+{new}"));
        } else {
            utils::replace_word(&mut self.subject, &format!("+{old}"), "");
        }
    }

    /// Replace existing context with a new one. Special cases:
    /// - new is empty: the old context is removed from the task
    /// - old is empty: the new context is appended to the task
    pub fn replace_context(&mut self, old: &str, new: &str) {
        let old = if old.starts_with('@') { &old["@".len()..] } else { old };
        if old.is_empty() {
            if !new.is_empty() && !self.contexts.iter().any(|p| p == new) {
                self.contexts.push(new.to_string());
                self.subject.push_str(" @");
                self.subject.push_str(new);
            }
            return;
        }
        let new = if new.starts_with('@') { &new["@".len()..] } else { new };
        self.contexts.retain(|proj| proj != old);
        if !new.is_empty() {
            self.contexts.push(new.to_string());
            utils::replace_word(&mut self.subject, &format!("@{old}"), &format!("@{new}"));
        } else {
            utils::replace_word(&mut self.subject, &format!("@{old}"), "");
        }
    }
    pub fn rec_until(&self) -> Option<NaiveDate> {
        if let Some(s) = self.tags.get("until") {
            let now = chrono::Local::now().date_naive();
            if let Ok(dt) = utils::parse_date(s, now) { Some(dt) } else { None }
        } else {
            None
        }
    }
}
