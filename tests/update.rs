use std::str::FromStr;
use todo_lib::todo;
use todo_txt;

fn init_tasks() -> todo::TaskVec {
    let mut t = Vec::new();

    t.push(todo_txt::task::Extended::from_str("call mother +family @parents").unwrap());
    t.push(
        todo_txt::task::Extended::from_str(
            "x (C) 2018-10-05 2018-10-01 call to car service and schedule repair +car @repair",
        )
        .unwrap(),
    );
    t.push(todo_txt::task::Extended::from_str("(B) 2018-10-15 repair family car +Car @repair due:2018-12-01").unwrap());
    t.push(
        todo_txt::task::Extended::from_str("(A) Kid's art school lesson +Family @Kids due:2018-11-10 rec:1w").unwrap(),
    );
    t.push(todo_txt::task::Extended::from_str("take kid to hockey game +Family @kids due:2018-11-18").unwrap());
    t.push(todo_txt::task::Extended::from_str("xmas vacations +FamilyHoliday due:2018-12-24").unwrap());

    t
}

fn init_task_lists() -> todo::TaskVec {
    let mut t = Vec::new();

    t.push(todo_txt::task::Extended::from_str("call mother +Family @parents").unwrap());
    t.push(todo_txt::task::Extended::from_str("call @Parents @father +family").unwrap());
    t.push(todo_txt::task::Extended::from_str("+car from service @car").unwrap());
    t.push(todo_txt::task::Extended::from_str("@CAR from service +CAR").unwrap());
    t.push(todo_txt::task::Extended::from_str("my +bday @me rec:2y").unwrap());
    t.push(todo_txt::task::Extended::from_str("rec:1m my wife +bday @wife +family").unwrap());

    t
}

#[test]
fn clones() {
    let t = init_tasks();
    let ids: todo::IDVec = vec![2, 4];
    let t2 = todo::clone_tasks(&t, &ids);
    assert_eq!(t2.len(), ids.len());
    assert_eq!(t[2], t2[0]);
    assert_eq!(t[4], t2[1]);

    let ids: todo::IDVec = vec![15, 4];
    let t2 = todo::clone_tasks(&t, &ids);
    assert_eq!(t2.len(), 1);
    assert_eq!(t[4], t2[0]);
}

#[test]
fn add() {
    let mut t = init_tasks();
    let mut c: todo::Conf = todo::Conf::default();

    let orig_len = t.len();
    c.subject = Some("new task".to_owned());
    let n = todo::add(&mut t, &c);
    assert_eq!(t.len(), orig_len + 1);
    assert_eq!(n, orig_len);
}

#[test]
fn done() {
    let mut t = init_tasks();

    let ids: todo::IDVec = vec![0, 1, 3, 4, 10];
    let old_date = t[3].due_date;
    let changed = todo::done(&mut t, Some(&ids));
    assert_eq!(changed, vec![true, false, true, true, false]);
    assert!(!t[3].finished);
    assert!(!t[2].finished && t[3].due_date != old_date);
    assert!(t[0].finished && t[4].finished);
}

#[test]
fn undone() {
    let mut t = init_tasks();

    let ids: todo::IDVec = vec![0, 2, 3];
    let changed = todo::undone(&mut t, Some(&ids));
    assert_eq!(changed, vec![false, false, false]);
    assert!(t[1].finished);
    assert!(!t[0].finished && !t[3].finished && !t[3].finished);

    let ids: todo::IDVec = vec![0, 1, 3, 4, 10];
    let changed = todo::undone(&mut t, Some(&ids));
    assert_eq!(changed, vec![false, true, false, false, false]);
    assert!(!t[1].finished);
    assert!(!t[0].finished && !t[3].finished && !t[4].finished);
}

#[test]
fn remove() {
    let mut t = init_tasks();

    let ids: todo::IDVec = vec![1, 20];
    let old_len = t.len();
    let changed = todo::remove(&mut t, Some(&ids));
    assert_eq!(changed, vec![true, false]);
    assert_eq!(t.len(), old_len - 1);

    let changed = todo::remove(&mut t, None);
    assert_eq!(changed, vec![true, true, true, true, true,]);
    assert_eq!(t.len(), 0);
}

#[test]
fn recs() {
    let mut t = init_task_lists();
    let mut c: todo::Conf = Default::default();

    let ids: todo::IDVec = vec![0, 1, 2, 3, 4, 5];
    c.project_act = todo::Action::Delete;
    c.projects = vec!["noproj".to_string()];
    let changed = todo::edit(&mut t, Some(&ids), &c);
    assert_eq!(changed, vec![false, false, false, false, false, false]);

    c.projects = vec!["CAR".to_string()];
    let changed = todo::edit(&mut t, Some(&ids), &c);
    assert_eq!(changed, vec![false, false, true, true, false, false]);

    c.project_act = todo::Action::Replace;
    c.projects = vec!["Family+People".to_string()];
    let changed = todo::edit(&mut t, Some(&ids), &c);
    assert_eq!(changed, vec![true, true, false, false, false, true]);

    c.recurrence_act = todo::Action::Delete;
    let changed = todo::edit(&mut t, Some(&ids), &c);
    assert_eq!(changed, vec![false, false, false, false, true, true]);
}
