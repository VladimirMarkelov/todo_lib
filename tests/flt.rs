use std::str::FromStr;
use todo_lib::{tfilter, todo, tsort};
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
    t.push(todo_txt::task::Extended::from_str("(B) 2018-10-15 repair family car +Car @repair due:2018-12-01 t:2019-01-02").unwrap());
    t.push(
        todo_txt::task::Extended::from_str("(A) Kid's art school lesson +Family @Kids due:2018-11-10 rec:1w").unwrap(),
    );
    t.push(todo_txt::task::Extended::from_str("take kid to hockey game +Family @kids due:2018-11-18").unwrap());
    t.push(todo_txt::task::Extended::from_str("xmas vacations +FamilyHoliday due:2018-12-24").unwrap());

    t
}

#[test]
fn one_item() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // invalid ranges
    cflt.range = tfilter::ItemRange::One(t.len());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids.len(), 0);

    cflt.range = tfilter::ItemRange::One(t.len() + 1);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids.len(), 0);
}

#[test]
fn item_range() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // both ends are out of range
    cflt.range = tfilter::ItemRange::Range(t.len(), t.len() + 5);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids.len(), 0);

    // one item that is completed
    cflt.range = tfilter::ItemRange::One(1);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids.len(), 0);

    // short range only active
    cflt.range = tfilter::ItemRange::Range(1, 3);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2, 3]);
    cflt.range = tfilter::ItemRange::None;
}

#[test]
fn item_status() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // one incomplete
    cflt.range = tfilter::ItemRange::One(0);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0]);

    // full range all
    cflt.all = tfilter::TodoStatus::All;
    cflt.range = tfilter::ItemRange::Range(0, 10);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 1, 2, 3, 4, 5]);

    // full range only completed
    cflt.all = tfilter::TodoStatus::Done;
    cflt.range = tfilter::ItemRange::Range(0, 10);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![1]);

    // full range only active
    cflt.all = tfilter::TodoStatus::Active;
    cflt.range = tfilter::ItemRange::Range(0, 10);
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 2, 3, 4, 5]);
}

#[test]
fn item_regex() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // all with 'car' anywhere
    cflt.all = tfilter::TodoStatus::All;
    cflt.regex = Some("car".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![1, 2]);
    cflt.all = tfilter::TodoStatus::Active;

    // active with <regex> anywhere
    cflt.use_regex = true;
    cflt.regex = Some("CA[rl]".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 2]);
}

#[test]
fn item_projects() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // active with 'car' project
    cflt.projects.push("car".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2]);
    cflt.projects.clear();

    // active with 'family' project
    cflt.projects.push("FAMILY".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 3, 4]);
    cflt.projects.clear();

    // active with 'family' project
    cflt.projects.push("FAMILY*".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 3, 4, 5]);
    cflt.projects.clear();

    // active with 'holiday' project
    cflt.projects.push("*holiday".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![5]);
    cflt.projects.clear();

    // active with 'family' related projects
    cflt.projects.push("*family*".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 3, 4, 5]);
}

#[test]
fn item_contexts() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();

    // active with 'kids' context
    cflt.contexts.push("kids".to_owned());
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![3, 4]);
    cflt.contexts.clear();
}

#[test]
fn item_priority() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();
    cflt.all = tfilter::TodoStatus::All;

    // only B priority
    cflt.pri = Some(tfilter::Priority {
        value: 'b' as u8 - 'a' as u8,
        span: tfilter::ValueSpan::Equal,
    });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2]);

    // B priority and higher
    cflt.pri = Some(tfilter::Priority {
        value: 'b' as u8 - 'a' as u8,
        span: tfilter::ValueSpan::Higher,
    });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2, 3]);

    //  B priority and lower
    cflt.pri = Some(tfilter::Priority {
        value: 'b' as u8 - 'a' as u8,
        span: tfilter::ValueSpan::Lower,
    });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 1, 2, 4, 5]);

    // any priority except no priority
    cflt.pri = Some(tfilter::Priority {
        value: todo::NO_PRIORITY,
        span: tfilter::ValueSpan::Any,
    });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![1, 2, 3]);

    // no priority
    cflt.pri = Some(tfilter::Priority {
        value: todo::NO_PRIORITY,
        span: tfilter::ValueSpan::None,
    });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 4, 5]);
}

#[test]
fn item_recurrence() {
    let t = init_tasks();
    let mut cflt = tfilter::Conf::default();
    cflt.all = tfilter::TodoStatus::All;

    // with recurrence
    cflt.rec = Some(tfilter::Recurrence {
        span: tfilter::ValueSpan::Any,
    });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![3]);

    // without recurrence
    cflt.rec = Some(tfilter::Recurrence {
        span: tfilter::ValueSpan::None,
    });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 1, 2, 4, 5]);
}

#[test]
fn item_due() {
    let t = init_tasks();

    let mut cflt = tfilter::Conf::default();
    cflt.all = tfilter::TodoStatus::All;

    // with due
    cflt.due = Some(tfilter::Due {
        span: tfilter::ValueSpan::Any,
        days: Default::default(),
    });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2, 3, 4, 5]);

    // without due
    cflt.due = Some(tfilter::Due {
        span: tfilter::ValueSpan::None,
        days: Default::default(),
    });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 1]);

    let sconf = tsort::Conf {
        fields: Some("due".to_string()),
        rev: true,
    };
    let mut ids: todo::IDVec = vec![0, 1, 2, 3, 4, 5];
    tsort::sort(&mut ids, &t, &sconf);
    assert_eq!(ids, vec![1, 0, 5, 2, 4, 3]);
}

#[test]
fn item_threshold() {
    let t = init_tasks();

    let mut cflt = tfilter::Conf::default();
    cflt.all = tfilter::TodoStatus::All;

    // with due
    cflt.thr = Some(tfilter::Due {
        span: tfilter::ValueSpan::Any,
        days: Default::default(),
    });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![2]);

    // without due
    cflt.thr = Some(tfilter::Due {
        span: tfilter::ValueSpan::None,
        days: Default::default(),
    });
    let ids = tfilter::filter(&t, &cflt);
    assert_eq!(ids, vec![0, 1, 3, 4, 5]);

    let sconf = tsort::Conf {
        fields: Some("thr".to_string()),
        rev: false,
    };
    let mut ids: todo::IDVec = vec![0, 1, 2, 3, 4, 5];
    tsort::sort(&mut ids, &t, &sconf);
    assert_eq!(ids, vec![2, 0, 1, 3, 4, 5]);
}
