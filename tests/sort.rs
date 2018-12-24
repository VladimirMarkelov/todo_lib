use std::str::FromStr;
use todo_lib::{todo, tsort};
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

fn make_id_vec(sz: usize) -> todo::IDVec {
    let mut v: todo::IDVec = Vec::new();
    for i in 0..sz {
        v.push(i);
    }
    v
}

#[test]
fn one_field() {
    let t = init_tasks();
    let mut ids = make_id_vec(t.len());

    // by priority: items without priority must be last items
    let mut c = tsort::Conf::default();
    c.fields = Some("priority".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![3, 2, 1, 0, 4, 5]);

    // by project: items without project must be last items
    // comparison is caseinsensitive
    // if an item has less projects it comes first
    let mut ids = make_id_vec(t.len());
    c.fields = Some("project".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![1, 2, 0, 3, 4, 5]);

    // by context: items without context must be last items
    // comparison is caseinsensitive
    // if an item has less contexts it comes first
    let mut ids = make_id_vec(t.len());
    c.fields = Some("context".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![3, 4, 0, 1, 2, 5]);

    // first - all incompleted and without recurrent tag
    // second - all incompleted recurrent items
    // last - completed items
    let mut ids = make_id_vec(t.len());
    c.fields = Some("done".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![0, 2, 4, 5, 3, 1]);
}

#[test]
fn one_field_reverse() {
    let t = init_tasks();
    let mut ids = make_id_vec(t.len());

    let mut c = tsort::Conf::default();
    c.rev = true;
    c.fields = Some("priority".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![5, 4, 0, 1, 2, 3]);

    let mut ids = make_id_vec(t.len());
    c.fields = Some("project".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![5, 4, 3, 0, 2, 1]);

    let mut ids = make_id_vec(t.len());
    c.fields = Some("context".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![5, 2, 1, 0, 4, 3]);

    let mut ids = make_id_vec(t.len());
    c.fields = Some("done".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![1, 3, 5, 4, 2, 0]);
}

#[test]
fn few_fields() {
    let t = init_tasks();
    let mut c = tsort::Conf::default();

    let mut ids = make_id_vec(t.len());
    c.fields = Some("pri,done".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![3, 2, 1, 0, 4, 5]);

    let mut ids = make_id_vec(t.len());
    c.fields = Some("proj,ctx".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![1, 2, 3, 4, 0, 5]);

    let mut ids = make_id_vec(t.len());
    c.fields = Some("done,proj".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![2, 0, 4, 5, 3, 1]);

    let mut ids = make_id_vec(t.len());
    c.fields = Some("proj,done".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![2, 1, 0, 4, 3, 5]);
}

#[test]
fn invalid_cases() {
    let t = init_tasks();

    // non-existing IDs must be at the end
    let mut ids: todo::IDVec = vec![12, 0, 1, 2, 19, 3, 4, 5, 20];
    let mut c = tsort::Conf::default();
    c.fields = Some("priority".to_owned());
    tsort::sort(&mut ids, &t, &c);
    assert_eq!(ids, vec![3, 2, 1, 0, 4, 5, 12, 19, 20]);
}
