use crate::todo;
use crate::todotxt;

/// Returns true if a given task is active - its timer is running
pub fn is_timer_on(task: &todotxt::Task) -> bool {
    if let Some(state) = task.tags.get(todo::TIMER_TAG) {
        return state != todo::TIMER_OFF;
    }
    false
}

/// Returns the time spent on a given task
pub fn spent_time(task: &todotxt::Task) -> chrono::Duration {
    if is_timer_on(task) {
        return match calc_time_spent(task) {
            Some(n) => chrono::Duration::seconds(n),
            None => chrono::Duration::seconds(0),
        };
    }

    if let Some(sp) = task.tags.get(todo::SPENT_TAG) {
        if let Ok(n) = sp.parse::<i64>() {
            chrono::Duration::seconds(n)
        } else {
            chrono::Duration::seconds(0)
        }
    } else {
        chrono::Duration::seconds(0)
    }
}

/// Make the todo active - start its timer. Attribute `tmr` is set to the
/// current time in seconds
pub fn start_timer(task: &mut todotxt::Task) -> bool {
    if task.finished || is_timer_on(task) {
        return false;
    }

    let utc = chrono::Utc::now();
    let seconds = format!("{}", utc.timestamp());
    task.update_tag_with_value(todo::TIMER_TAG, &seconds);

    true
}

fn calc_time_spent(task: &todotxt::Task) -> Option<i64> {
    if let Some(started) = task.tags.get(todo::TIMER_TAG) {
        if let Ok(n) = started.parse::<i64>() {
            let dt_start = chrono::DateTime::from_timestamp(n, 0)?;
            let diff = chrono::Utc::now() - dt_start;

            let mut spent: i64 =
                if let Some(sp) = task.tags.get(todo::SPENT_TAG) { sp.parse::<i64>().unwrap_or(0) } else { 0 };

            if diff.num_seconds() > 0 {
                spent += diff.num_seconds();
            }

            return Some(spent);
        }
    }

    None
}

/// Stops the todo's timer and updates the spent time. Attribute `tmr` gets
/// value 'off'
pub fn stop_timer(task: &mut todotxt::Task) -> bool {
    if !is_timer_on(task) {
        return false;
    }

    if let Some(spent) = calc_time_spent(task) {
        let new_spent = format!("{spent}");
        task.update_tag_with_value(todo::SPENT_TAG, &new_spent);
        task.update_tag_with_value(todo::TIMER_TAG, todo::TIMER_OFF);
        return true;
    }

    false
}
