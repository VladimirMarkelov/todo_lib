use std::collections::HashMap;
use todo_lib::{
    todo,
    todotxt::{self, CompletionConfig},
};

fn init_tasks() -> todo::TaskVec {
    let mut t = Vec::new();
    let now = chrono::Local::now().date_naive();

    t.push(todotxt::Task::parse("call mother +family @parents", now));
    t.push(todotxt::Task::parse(
        "x (C) 2018-10-05 2018-10-01 call to car service and schedule repair +car @repair",
        now,
    ));
    t.push(todotxt::Task::parse("(B) 2018-10-15 repair family car +Car @repair due:2018-12-01", now));
    t.push(todotxt::Task::parse("(A) Kid's art school lesson +Family @Kids due:2018-11-10 rec:1w", now));
    t.push(todotxt::Task::parse("take kid to hockey game +Family @kids due:2018-11-18", now));
    t.push(todotxt::Task::parse("xmas vacations +FamilyHoliday due:2018-12-24", now));

    t
}

fn init_task_lists() -> todo::TaskVec {
    let mut t = Vec::new();
    let now = chrono::Local::now().date_naive();

    t.push(todotxt::Task::parse("call mother +Family @parents", now));
    t.push(todotxt::Task::parse("call @Parents @father +family", now));
    t.push(todotxt::Task::parse("+car from service @car", now));
    t.push(todotxt::Task::parse("@CAR from service +CAR", now));
    t.push(todotxt::Task::parse("my +bday @me rec:2y", now));
    t.push(todotxt::Task::parse("rec:1m my wife +bday @wife +family", now));

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
#[allow(deprecated)] // tests functions that were left for back compatibility
fn done() {
    let mut t = init_tasks();
    let orig_len = t.len();
    let ids: todo::IDVec = vec![0, 1, 3, 4, 10];
    let mut must_change = 0;
    for i in &ids {
        if t.len() < *i {
            continue;
        }
        if t[*i].finished {
            continue;
        }
        if t[*i].recurrence.is_some() && (t[*i].due_date.is_some() || t[*i].threshold_date.is_some()) {
            must_change += 1;
        }
    }

    let old_date = t[3].due_date;
    let changed = todo::done(&mut t, Some(&ids), todotxt::CompletionMode::JustMark);
    assert_eq!(changed, vec![true, false, true, true, false]);
    assert!(!t[2].finished);
    assert!(t[3].due_date == old_date);
    for i in 0..5 {
        assert!(i == 2 || t[i].finished);
    }
    assert_eq!(t.len(), orig_len + must_change);
    for idx in orig_len..orig_len + must_change {
        assert!(!t[idx].finished);
    }
}

#[test]
fn done_with_config() {
    let mut t: Vec<todotxt::Task> = init_tasks();
    let orig_len = t.len();
    let ids: todo::IDVec = vec![0, 1, 3, 4, 10];
    let mut must_change = 0;
    for i in &ids {
        if t.len() < *i {
            continue;
        }
        if t[*i].finished {
            continue;
        }
        if t[*i].recurrence.is_some() && (t[*i].due_date.is_some() || t[*i].threshold_date.is_some()) {
            must_change += 1;
        }
    }

    let old_date = t[3].due_date;
    let completion_config = CompletionConfig {
        completion_mode: todotxt::CompletionMode::JustMark,
        completion_date_mode: todotxt::CompletionDateMode::AlwaysSet,
    };
    let changed = todo::done_with_config(&mut t, Some(&ids), completion_config);
    assert_eq!(changed, vec![true, false, true, true, false]);
    assert!(!t[2].finished);
    assert!(t[3].due_date == old_date);
    for i in 0..5 {
        assert!(i == 2 || t[i].finished);
        assert!(i == 2 || t[i].finish_date.is_some())
    }
    assert_eq!(t.len(), orig_len + must_change);
    for idx in orig_len..orig_len + must_change {
        assert!(!t[idx].finished);
    }
}

#[test]
fn undone() {
    let mut t = init_tasks();

    let ids: todo::IDVec = vec![0, 2, 3];
    let changed = todo::undone(&mut t, Some(&ids), todotxt::CompletionMode::JustMark);
    assert_eq!(changed, vec![false, false, false]);
    assert!(t[1].finished);
    assert!(!t[0].finished && !t[3].finished && !t[3].finished);

    let ids: todo::IDVec = vec![0, 1, 3, 4, 10];
    let changed = todo::undone(&mut t, Some(&ids), todotxt::CompletionMode::JustMark);
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
    c.projects = todo::ListTagChange { action: todo::Action::Delete, value: vec!["noproj".to_string()] };
    let changed = todo::edit(&mut t, Some(&ids), &c);
    assert_eq!(changed, vec![false, false, false, false, false, false]);

    c.projects = todo::ListTagChange { action: todo::Action::Delete, value: vec!["CAR".to_string()] };
    let changed = todo::edit(&mut t, Some(&ids), &c);
    assert_eq!(changed, vec![false, false, false, true, false, false]);

    c.projects = todo::ListTagChange { action: todo::Action::Replace, value: vec!["Family+People".to_string()] };
    let changed = todo::edit(&mut t, Some(&ids), &c);
    assert_eq!(changed, vec![true, false, false, false, false, false]);

    c.recurrence_act = todo::Action::Delete;
    let changed = todo::edit(&mut t, Some(&ids), &c);
    assert_eq!(changed, vec![false, false, false, false, true, true]);
}

#[test]
fn tag_update_test() {
    struct Test {
        subj: &'static str,
        tags: Vec<&'static str>,
        res: &'static str,
        delete: bool,
        changes: bool,
    }
    let data: Vec<Test> = vec![
        Test {
            subj: "item:ball take to who:me game game:there",
            tags: vec!["game:here", "item:puck"],
            res: "item:puck take to who:me game game:here",
            delete: false,
            changes: true,
        },
        Test {
            subj: "item:ball take to who:me game game:there",
            tags: vec!["gam", "item"],
            res: "take to who:me game game:there",
            delete: true,
            changes: true,
        },
        Test {
            subj: "item:ball take to who:me why:because game game:there",
            tags: vec!["game:new", "item:some", "who:that", "wh:some"],
            res: "take to why:because game",
            delete: true,
            changes: true,
        },
        Test {
            subj: "item:ball take to who:me game game:there",
            tags: vec!["game", "item"],
            res: "take to who:me game",
            delete: false,
            changes: true,
        },
        Test {
            subj: "item:ball take to who:me game game:there",
            tags: vec!["who:they", "ite"],
            res: "item:ball take to who:they game game:there",
            delete: false,
            changes: true,
        },
        Test {
            subj: "item:ball take to who:me game game:there",
            tags: vec!["who:they", "item:puck", "date:tomorrow", "game:somewhere"],
            res: "item:puck take to who:they game game:somewhere date:tomorrow",
            delete: false,
            changes: true,
        },
        Test {
            subj: "item:ball take to who:me game game:there",
            tags: vec!["who:me", "item:ball"],
            res: "item:ball take to who:me game game:there",
            delete: false,
            changes: false,
        },
        Test {
            subj: "item:ball take to who:me game game:there",
            tags: vec!["wh:me", "ite"],
            res: "item:ball take to who:me game game:there",
            delete: true,
            changes: false,
        },
    ];

    let now = chrono::Local::now().date_naive();
    for (idx, test) in data.iter().enumerate() {
        let mut t = Vec::new();
        t.push(todotxt::Task::parse(test.subj, now));

        let mut c: todo::Conf = todo::Conf::default();
        c.tags_act = if test.delete { todo::Action::Delete } else { todo::Action::Set };
        let mut hm = HashMap::<String, String>::new();
        for tag in &test.tags {
            if let Some(pos) = tag.find(':') {
                hm.insert(tag[..pos].to_string(), tag[pos + 1..].to_string());
            } else {
                hm.insert(tag.to_string(), String::new());
            }
        }
        c.tags = Some(hm);
        let changed = todo::edit(&mut t, None, &c);
        if test.changes {
            assert!(changed.len() > 0 && changed[0]);
        } else {
            assert!(changed.len() == 0 || !changed[0]);
        }
        assert_eq!(test.res, &t[0].subject, "\n{}. {} != {}", idx, t[0].subject, test.res);
    }
}

#[test]
fn hashtags_test() {
    struct Test {
        subj: &'static str,
        hashtags: Vec<&'static str>,
        res: &'static str,
        act: todo::Action,
        changed: bool,
    }
    let data: Vec<Test> = vec![
        Test {
            subj: "test #about some #hashtags",
            hashtags: vec!["about", "hashtags"],
            res: "",
            act: todo::Action::None,
            changed: false,
        },
        Test {
            subj: "test #about some #hashtags",
            hashtags: vec!["about", "tags"],
            res: "test some #hashtags",
            act: todo::Action::Delete,
            changed: true,
        },
        Test {
            subj: "test #about some #hashtags",
            hashtags: vec!["about", "tags"],
            res: "test #about some #hashtags #tags",
            act: todo::Action::Set,
            changed: true,
        },
        Test {
            subj: "test #about some #hashtags and #some",
            hashtags: vec!["about:this", "hashtags:tags", "no:yes"],
            res: "test #this some #tags and #some",
            act: todo::Action::Replace,
            changed: true,
        },
        Test {
            subj: "test #about some #hashtags and #some",
            hashtags: vec!["about:about"],
            res: "test #about some #hashtags and #some",
            act: todo::Action::Replace,
            changed: false,
        },
    ];
    let now = chrono::Local::now().date_naive();
    for (idx, test) in data.iter().enumerate() {
        let mut t = Vec::new();
        t.push(todotxt::Task::parse(test.subj, now));

        if let todo::Action::None = test.act {
            for (i, h) in test.hashtags.iter().enumerate() {
                assert_eq!(t[0].hashtags[i], h.to_string(), "{}. {:?} != {:?}", idx, test.hashtags, t[0].hashtags);
            }
            continue;
        }

        let mut c: todo::Conf = todo::Conf::default();
        let mut hvec = Vec::new();
        for h in test.hashtags.iter() {
            hvec.push(h.to_string());
        }
        c.hashtags = todo::ListTagChange { value: hvec, action: test.act };
        let changed = todo::edit(&mut t, None, &c);
        assert!(changed.len() > 0, "{}. {}", idx, t[0].subject);
        assert_eq!(changed[0], test.changed, "{}. {}", idx, t[0].subject);
        assert_eq!(test.res, &t[0].subject, "\n{}. {} != {}", idx, t[0].subject, test.res);
    }
}
