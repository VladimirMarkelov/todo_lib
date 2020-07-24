use crate::todo;

/// Returns true if a given task is active - its timer is running
pub fn is_timer_on(task: &todo_txt::task::Extended) -> bool {
    if let Some(state) = task.tags.get(todo::TIMER_TAG) {
        return state != todo::TIMER_OFF;
    }
    false
}

/// Returns the time spent on a given task
pub fn spent_time(task: &todo_txt::task::Extended) -> chrono::Duration {
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
pub fn start_timer(task: &mut todo_txt::task::Extended) -> bool {
    if task.finished || is_timer_on(task) {
        return false;
    }

    let utc = chrono::Utc::now();
    let seconds = format!("{}", utc.timestamp());
    task.tags.insert(todo::TIMER_TAG.to_string(), seconds);

    true
}

fn calc_time_spent(task: &todo_txt::task::Extended) -> Option<i64> {
    if let Some(started) = task.tags.get(todo::TIMER_TAG) {
        if let Ok(n) = started.parse::<i64>() {
            let dt_start = chrono::NaiveDateTime::from_timestamp(n, 0);
            let diff = chrono::Utc::now().naive_utc() - dt_start;

            let mut spent: i64 = if let Some(sp) = task.tags.get(todo::SPENT_TAG) {
                match sp.parse::<i64>() {
                    Ok(n) => n,
                    Err(_) => 0,
                }
            } else {
                0
            };

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
pub fn stop_timer(task: &mut todo_txt::task::Extended) -> bool {
    if !is_timer_on(task) {
        return false;
    }

    if let Some(spent) = calc_time_spent(task) {
        let new_spent = format!("{}", spent);
        task.tags.insert(todo::SPENT_TAG.to_string(), new_spent);
        task.tags
            .insert(todo::TIMER_TAG.to_string(), todo::TIMER_OFF.to_string());
        return true;
    }

    false
}
