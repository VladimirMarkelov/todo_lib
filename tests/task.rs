use chrono::NaiveDate;
use todo_lib::todo::{Action, Conf, DateTagChange, NewDateValue, done, edit};
use todo_lib::todotxt::{CompletionConfig, CompletionDateMode, CompletionMode, Task, business_days_between};

#[test]
fn parse_tasks_simple() {
    struct Test {
        i: &'static str,
        t: Task,
    }
    let data: Vec<Test> = vec![
        Test { i: "just text", t: Task { subject: "just text".to_string(), ..Default::default() } },
        Test { i: "x just text", t: Task { subject: "just text".to_string(), finished: true, ..Default::default() } },
        Test { i: "X just text", t: Task { subject: "X just text".to_string(), ..Default::default() } },
        Test { i: "xjust text", t: Task { subject: "xjust text".to_string(), ..Default::default() } },
        Test { i: "(a) just text", t: Task { subject: "(a) just text".to_string(), ..Default::default() } },
        Test { i: "(B) just text", t: Task { subject: "just text".to_string(), priority: 1, ..Default::default() } },
        Test { i: "(B)just text", t: Task { subject: "(B)just text".to_string(), ..Default::default() } },
        Test {
            i: "2020-01-03 just text",
            t: Task {
                subject: "just text".to_string(),
                create_date: Some(NaiveDate::from_ymd_opt(2020, 01, 03).unwrap()),
                ..Default::default()
            },
        },
        Test {
            i: "2020-02-03 2020-01-03 just text",
            t: Task {
                subject: "2020-01-03 just text".to_string(),
                create_date: Some(NaiveDate::from_ymd_opt(2020, 02, 03).unwrap()),
                ..Default::default()
            },
        },
        Test {
            i: "x 2020-02-03 2020-01-03 just text",
            t: Task {
                subject: "just text".to_string(),
                create_date: Some(NaiveDate::from_ymd_opt(2020, 01, 03).unwrap()),
                finish_date: Some(NaiveDate::from_ymd_opt(2020, 02, 03).unwrap()),
                finished: true,
                ..Default::default()
            },
        },
        Test {
            i: "x (E) 2020-02-03 2020-01-03 just text",
            t: Task {
                subject: "just text".to_string(),
                priority: 4,
                create_date: Some(NaiveDate::from_ymd_opt(2020, 01, 03).unwrap()),
                finish_date: Some(NaiveDate::from_ymd_opt(2020, 02, 03).unwrap()),
                finished: true,
                ..Default::default()
            },
        },
        Test {
            i: "x (j) 2020-02-03 2020-01-03 just text",
            t: Task {
                subject: "(j) 2020-02-03 2020-01-03 just text".to_string(),
                finished: true,
                ..Default::default()
            },
        },
        Test {
            i: "2020-31-03 just text",
            t: Task { subject: "2020-31-03 just text".to_string(), ..Default::default() },
        },
        Test {
            i: "2020-01-43 just text",
            t: Task { subject: "2020-01-43 just text".to_string(), ..Default::default() },
        },
        Test {
            i: "2020-01-03a just text",
            t: Task { subject: "2020-01-03a just text".to_string(), ..Default::default() },
        },
    ];
    let base = NaiveDate::from_ymd_opt(2020, 3, 15).unwrap();
    for d in data.iter() {
        let t = Task::parse(d.i, base);
        assert_eq!(d.t, t, "{}", d.i);
        let back = format!("{}", t);
        assert_eq!(d.i, &back, "{}", d.i);
    }
}

#[test]
fn parse_tasks_tags() {
    struct Test {
        i: &'static str,
        o: &'static str,
        hk: Vec<&'static str>,
        hv: Vec<&'static str>,
    }
    let data: Vec<Test> = vec![
        Test { i: "task rec: due:2d", o: "task rec: due:2020-03-17", hk: vec!["due"], hv: vec!["2020-03-17"] },
        Test {
            i: "task rec:2w due:2d",
            o: "task rec:2w due:2020-03-17",
            hk: vec!["rec", "due"],
            hv: vec!["2w", "2020-03-17"],
        },
        Test {
            i: "due:2d task rec:2w due:2d",
            o: "due:2020-03-17 task rec:2w due:2020-03-17",
            hk: vec!["rec", "due"],
            hv: vec!["2w", "2020-03-17"],
        },
        Test {
            i: "task rec:2w due:2d t:1m",
            o: "task rec:2w due:2020-03-17 t:2020-04-15",
            hk: vec!["rec", "due", "t"],
            hv: vec!["2w", "2020-03-17", "2020-04-15"],
        },
        Test {
            i: "task rec:2w due:230 14:20 end",
            o: "task rec:2w due:230 14:20 end",
            hk: vec!["rec", "due", "14"],
            hv: vec!["2w", "230", "20"],
        },
    ];
    let base = NaiveDate::from_ymd_opt(2020, 3, 15).unwrap();
    for d in data.iter() {
        let t = Task::parse(d.i, base);
        if t.tags.is_empty() {
            assert!(d.hk.is_empty(), "{} has no tags", d.i);
        } else {
            assert!(d.hk.len() == t.tags.len(), "{}", d.i);
        }
        for (k, v) in d.hk.iter().zip(d.hv.iter()) {
            let val = t.tags.get(*k);
            assert!(val.is_some(), "{} - {}", d.i, k);
            assert_eq!(v, val.unwrap(), "{} - {}", d.i, k);
        }

        let back = format!("{}", t);
        if d.o == "=" {
            assert_eq!(d.i, &back, "{}", d.i);
        } else if !d.o.is_empty() {
            assert_eq!(d.o, &back, "{}", d.i);
        }
    }
}

#[test]
#[allow(deprecated)]
fn complete_old_signature() {
    struct Test {
        i: &'static str,
        d: &'static str,
        u: &'static str,
        m: CompletionMode,
    }
    let data: Vec<Test> = vec![
        Test { i: "test", d: "x test", u: "test", m: CompletionMode::JustMark },
        Test {
            i: "2020-01-01 test",
            d: "x 2020-02-02 2020-01-01 test",
            u: "2020-01-01 test",
            m: CompletionMode::JustMark,
        },
        Test {
            i: "test rec:+1m due:2020-03-01",
            d: "x test rec:+1m due:2020-03-01",
            u: "test rec:+1m due:2020-03-01",
            m: CompletionMode::JustMark,
        },
        Test {
            i: "test rec:1m due:2020-03-01",
            d: "x test rec:1m due:2020-03-01",
            u: "test rec:1m due:2020-03-01",
            m: CompletionMode::JustMark,
        },
        Test {
            i: "2020-01-01 test rec:7d",
            d: "x 2020-02-02 2020-01-01 test rec:7d",
            u: "2020-01-01 test rec:7d",
            m: CompletionMode::JustMark,
        },
        Test { i: "(B) testb", d: "x (B) testb", u: "(B) testb", m: CompletionMode::JustMark },
        Test { i: "(B) testb", d: "x (B) testb", u: "(B) testb", m: CompletionMode::MovePriority },
        Test { i: "(B) testb", d: "x testb pri:B", u: "(B) testb", m: CompletionMode::PriorityToTag },
        Test { i: "(B) testb", d: "x testb", u: "testb", m: CompletionMode::RemovePriority },
        Test {
            i: "(B) 2020-01-01 testc",
            d: "x 2020-02-02 2020-01-01 (B) testc",
            u: "(B) 2020-01-01 testc",
            m: CompletionMode::MovePriority,
        },
        Test {
            i: "(B) 2020-01-01 testc",
            d: "x 2020-02-02 2020-01-01 testc pri:B",
            u: "(B) 2020-01-01 testc",
            m: CompletionMode::PriorityToTag,
        },
        Test {
            i: "(B) 2020-01-01 testc",
            d: "x 2020-02-02 2020-01-01 testc",
            u: "2020-01-01 testc",
            m: CompletionMode::RemovePriority,
        },
    ];
    let base = NaiveDate::from_ymd_opt(2020, 2, 2).unwrap();
    for d in data.iter() {
        let mut t = Task::parse(d.i, base);
        t.complete(base, d.m);
        assert_eq!(d.d, &format!("{}", t), "done '{}', mode: {:?}", d.i, d.m);
        if t.create_date.is_some() && t.recurrence.is_none() {
            assert_eq!(t.finish_date, Some(base));
        }
        if d.m != CompletionMode::RemovePriority {
            t.uncomplete(d.m);
            assert_eq!(d.u, &format!("{}", t), "undone '{}', mode: {:?}", d.i, d.m);
        }
    }
}

#[test]
fn complete_uncomplete() {
    struct Test {
        i: &'static str,
        d: &'static str,
        u: &'static str,
        m: CompletionMode,
        cdm: CompletionDateMode,
    }
    let data: Vec<Test> = vec![
        Test {
            i: "test",
            d: "x test",
            u: "test",
            m: CompletionMode::JustMark,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "test",
            d: "x 2020-02-02 test",
            u: "test",
            m: CompletionMode::JustMark,
            cdm: CompletionDateMode::AlwaysSet,
        },
        Test {
            i: "2020-01-01 test",
            d: "x 2020-02-02 2020-01-01 test",
            u: "2020-01-01 test",
            m: CompletionMode::JustMark,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "test rec:+1m due:2020-03-01",
            d: "x test rec:+1m due:2020-03-01",
            u: "test rec:+1m due:2020-03-01",
            m: CompletionMode::JustMark,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "test rec:1m due:2020-03-01",
            d: "x test rec:1m due:2020-03-01",
            u: "test rec:1m due:2020-03-01",
            m: CompletionMode::JustMark,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "test rec:1m due:2020-03-01",
            d: "x test rec:1m due:2020-03-01",
            u: "test rec:1m due:2020-03-01",
            m: CompletionMode::JustMark,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "2020-01-01 test rec:7d",
            d: "x 2020-02-02 2020-01-01 test rec:7d",
            u: "2020-01-01 test rec:7d",
            m: CompletionMode::JustMark,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "(B) testb",
            d: "x (B) testb",
            u: "(B) testb",
            m: CompletionMode::JustMark,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "(B) testb",
            d: "x (B) 2020-02-02 testb",
            u: "(B) testb",
            m: CompletionMode::JustMark,
            cdm: CompletionDateMode::AlwaysSet,
        },
        Test {
            i: "(B) testb",
            d: "x (B) testb",
            u: "(B) testb",
            m: CompletionMode::MovePriority,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "(B) testb",
            d: "x 2020-02-02 (B) testb",
            u: "(B) testb",
            m: CompletionMode::MovePriority,
            cdm: CompletionDateMode::AlwaysSet,
        },
        Test {
            i: "(B) testb",
            d: "x testb pri:B",
            u: "(B) testb",
            m: CompletionMode::PriorityToTag,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "(B) testb",
            d: "x testb",
            u: "testb",
            m: CompletionMode::RemovePriority,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "(B) testb",
            d: "x 2020-02-02 testb",
            u: "testb",
            m: CompletionMode::RemovePriority,
            cdm: CompletionDateMode::AlwaysSet,
        },
        Test {
            i: "(B) 2020-01-01 testc",
            d: "x 2020-02-02 2020-01-01 (B) testc",
            u: "(B) 2020-01-01 testc",
            m: CompletionMode::MovePriority,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "(B) 2020-01-01 testc",
            d: "x 2020-02-02 2020-01-01 testc pri:B",
            u: "(B) 2020-01-01 testc",
            m: CompletionMode::PriorityToTag,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
        Test {
            i: "(B) 2020-01-01 testc",
            d: "x 2020-02-02 2020-01-01 testc",
            u: "2020-01-01 testc",
            m: CompletionMode::RemovePriority,
            cdm: CompletionDateMode::WhenCreationDateIsPresent,
        },
    ];
    let base = NaiveDate::from_ymd_opt(2020, 2, 2).unwrap();
    for d in data.iter() {
        let mut t = Task::parse(d.i, base);
        t.complete_with_config(base, CompletionConfig { completion_mode: d.m, completion_date_mode: d.cdm });
        assert_eq!(d.d, &format!("{}", t), "done '{}', mode: {:?}", d.i, d.m);
        if t.create_date.is_some() && t.recurrence.is_none() {
            assert_eq!(t.finish_date, Some(base));
        }
        if d.m != CompletionMode::RemovePriority {
            t.uncomplete(d.m);
            assert_eq!(d.u, &format!("{}", t), "undone '{}', mode: {:?}", d.i, d.m);
        }
    }
}

#[test]
fn complete_cleanup_recurrent_test() {
    struct Test {
        i: &'static str,
        n: &'static str,
        m: CompletionMode,
    }
    let data: Vec<Test> = vec![
        Test { i: "test rec:1d due:2020-02-01 tmr:off one", n: "test rec:1d one", m: CompletionMode::JustMark },
        Test { i: "test rec:1d due:2020-02-01 two spent:23", n: "test rec:1d two", m: CompletionMode::JustMark },
        Test {
            i: "test rec:1d due:2020-02-01 spent:23 three tmr:on four",
            n: "test rec:1d three four",
            m: CompletionMode::JustMark,
        },
    ];

    let base = NaiveDate::from_ymd_opt(2020, 2, 2).unwrap();
    for d in data.iter() {
        let t = Task::parse(d.i, base);
        let mut tasks: Vec<Task> = vec![t];
        let completion_config =
            CompletionConfig { completion_mode: d.m, completion_date_mode: CompletionDateMode::AlwaysSet };
        let changed = done(&mut tasks, None, completion_config);

        assert_eq!(changed.len(), 1, "Expected 1 changed tasks, got {0}", changed.len());
        println!("{:?}", tasks);
        assert_eq!(tasks.len(), 2, "Expected new task is created");
        let mut new_cleaned = tasks[1].clone();
        new_cleaned.update_tag("due:");
        assert_eq!(
            d.n,
            &format!("{}", new_cleaned),
            "Invalid new task [{0}], expected [{1}]",
            &format!("{}", new_cleaned),
            d.n
        );
    }
}

#[test]
fn business_days_between_test() {
    struct Test {
        s: NaiveDate,
        e: NaiveDate,
        d: i64,
    }
    let data: Vec<Test> = vec![
        Test {
            s: NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            e: NaiveDate::from_ymd_opt(2024, 2, 16).unwrap(),
            d: 0,
        },
        Test {
            s: NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            e: NaiveDate::from_ymd_opt(2024, 2, 21).unwrap(),
            d: 2,
        },
        Test {
            s: NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            e: NaiveDate::from_ymd_opt(2024, 2, 24).unwrap(),
            d: 4,
        },
        Test {
            s: NaiveDate::from_ymd_opt(2024, 2, 24).unwrap(),
            e: NaiveDate::from_ymd_opt(2024, 2, 24).unwrap(),
            d: 2,
        },
    ];
    for d in data.iter() {
        let r = business_days_between(d.s, d.e);
        assert_eq!(d.d, r, "done {} --> {}", d.s, d.e);
    }
}

#[test]
fn next_date() {
    struct Test {
        i: &'static str,
        d: &'static str,
    }
    let data: Vec<Test> = vec![
        Test { i: "2020-01-01 test", d: "2020-01-01 test" },
        Test { i: "test t:2020-03-02 rec:+1m due:2020-03-01", d: "test t:2020-04-02 rec:+1m due:2020-04-01" },
        Test { i: "test t:2020-02-29 rec:1m due:2020-03-01", d: "test t:2020-03-02 rec:1m due:2020-03-02" },
        Test { i: "2020-01-01 test rec:7d", d: "2020-01-01 test rec:7d" },
        Test { i: "2020-01-01 test due:2020-01-01", d: "2020-01-01 test due:2020-01-01" },
        Test { i: "test rec:7d due:2020-02-01", d: "test rec:7d due:2020-02-09" },
        Test { i: "test rec:1b due:2020-02-01", d: "test rec:1b due:2020-02-03" },
        Test { i: "test rec:7b due:2020-02-01", d: "test rec:7b due:2020-02-11" },
        Test { i: "test rec:14b due:2020-02-01", d: "test rec:14b due:2020-02-20" },
        Test { i: "test rec:+14b due:2020-02-01", d: "test rec:+14b due:2020-02-20" },
    ];
    let base = NaiveDate::from_ymd_opt(2020, 2, 2).unwrap();
    for d in data.iter() {
        let mut t = Task::parse(d.i, base);
        let orig_due = t.due_date.clone();
        let orig_thr = t.threshold_date.clone();
        t.next_dates(base);
        assert_eq!(d.d, &format!("{}", t), "done {}", d.i);
        if orig_due.is_some() && t.recurrence.is_some() && t.due_date.is_some() {
            let orig = orig_due.unwrap();
            assert!(orig < t.due_date.unwrap(), "due must change: {}", d.i);
        }
        if orig_thr.is_some() && t.recurrence.is_some() && t.threshold_date.is_some() {
            let orig = orig_thr.unwrap();
            assert!(orig < t.threshold_date.unwrap(), "threshold must change: {}", d.i);
        }
    }
}

#[test]
fn replace_projects() {
    struct Test {
        i: &'static str,
        o: &'static str,
        r: &'static str,
        w: &'static str,
    }
    let data: Vec<Test> = vec![
        Test { i: "str abc", o: "str abc", r: "+tag", w: "+tg" },
        Test { i: "str +tag1 abc", o: "str +tag1 abc", r: "+tag", w: "+tg" },
        Test { i: "str some+tag abc", o: "str some+tag abc", r: "+tag", w: "+tg" },
        Test { i: "+tag str abc +tag1", o: "+newtag str abc +tag1", r: "+tag", w: "+newtag" },
        Test { i: "+tag str abc +tag1", o: "str abc +tag1", r: "+tag", w: "" },
        Test { i: "+tag str abc +tag1", o: "+tag str abc", r: "+tag1", w: "" },
        Test { i: "+tag str +abc +tag1", o: "+tag str +tag1", r: "abc", w: "" },
        Test { i: "efg ++tag str abc +tag", o: "efg ++tag str abc +newstr", r: "+tag", w: "+newstr" },
    ];

    let dt = NaiveDate::from_ymd_opt(2021, 01, 05).unwrap();
    for d in data.iter() {
        let mut t = Task::parse(d.i, dt);
        t.replace_project(d.r, d.w);
        assert_eq!(d.o, &t.subject, "{}: {} -> {}", d.i, d.r, d.w);
    }
}

#[test]
fn replace_contexts() {
    struct Test {
        i: &'static str,
        o: &'static str,
        r: &'static str,
        w: &'static str,
    }
    let data: Vec<Test> = vec![
        Test { i: "str abc", o: "str abc", r: "@tag", w: "@tg" },
        Test { i: "str @tag1 abc", o: "str @tag1 abc", r: "@tag", w: "@tg" },
        Test { i: "str some@tag abc", o: "str some@tag abc", r: "@tag", w: "@tg" },
        Test { i: "@tag str abc @tag1", o: "@newtag str abc @tag1", r: "@tag", w: "@newtag" },
        Test { i: "@tag str abc @tag1", o: "str abc @tag1", r: "@tag", w: "" },
        Test { i: "@tag str abc @tag1", o: "@tag str abc", r: "@tag1", w: "" },
        Test { i: "@tag str @abc @tag1", o: "@tag str @tag1", r: "abc", w: "" },
        Test { i: "efg @@tag str abc @tag", o: "efg @@tag str abc @newstr", r: "@tag", w: "@newstr" },
    ];

    let dt = NaiveDate::from_ymd_opt(2021, 01, 05).unwrap();
    for d in data.iter() {
        let mut t = Task::parse(d.i, dt);
        t.replace_context(d.r, d.w);
        assert_eq!(d.o, &t.subject, "{}: {} -> {}", d.i, d.r, d.w);
    }
}

#[test]
fn replace_recurrences() {
    struct Test {
        i: &'static str,
        o: &'static str,
        w: &'static str,
    }
    let data: Vec<Test> = vec![
        Test { i: "str abc", o: "str abc rec:12", w: "12" },
        Test { i: "str somrec:45 abc", o: "str somrec:45 abc rec:12", w: "12" },
        Test { i: "rec:", o: "rec: rec:345", w: "345" },
        Test { i: "abc rec: def", o: "abc rec: def rec:345", w: "345" },
        Test { i: "rec:11", o: "rec:345", w: "345" },
        Test { i: "str rec:22", o: "str rec:345", w: "345" },
        Test { i: "str rec:222 end", o: "str rec:345 end", w: "345" },
        Test { i: "rec:11 text rec:22", o: "rec:11 text rec:345", w: "345" },
        Test { i: "rec:22 text rec:22", o: "rec:345 text rec:345", w: "345" },
        Test { i: "rrec:11 text rec:22", o: "rrec:11 text rec:345", w: "345" },
    ];

    let dt = NaiveDate::from_ymd_opt(2021, 01, 05).unwrap();
    for d in data.iter() {
        let mut t = Task::parse(d.i, dt);
        t.update_tag_with_value("rec", d.w);
        assert_eq!(d.o, &t.subject, "{}-> {}", d.i, d.w);
    }
}

#[test]
fn finish_date_with_create_date() {
    let task = Task {
        finished: true,
        create_date: Some(NaiveDate::default()),
        finish_date: NaiveDate::default().succ_opt(),
        subject: "Feed cat".to_owned(),
        ..Default::default()
    };
    assert_eq!(task.to_string(), "x 1970-01-02 1970-01-01 Feed cat")
}

#[test]
fn finish_date_without_create_date() {
    let task = Task {
        finished: true,
        finish_date: NaiveDate::default().succ_opt(),
        subject: "Feed cat".to_owned(),
        ..Default::default()
    };
    assert_eq!(task.to_string(), "x 1970-01-02 Feed cat")
}

#[test]
fn due_expr_test() {
    struct Test {
        i: &'static str,
        e: &'static str,
        d: &'static str,
    }
    let data: Vec<Test> = vec![
        Test { i: "feed cat thr:2020-10-10", e: "thr+1d", d: "feed cat thr:2020-10-10 due:2020-10-11" },
        Test { i: "feed cat due:2020-10-09 thr:2020-10-10", e: "thr+1d", d: "feed cat due:2020-10-11 thr:2020-10-10" },
        Test { i: "feed cat due:2020-10-09 thr:2020-10-10", e: "due+1d", d: "feed cat due:2020-10-10 thr:2020-10-10" },
        Test {
            i: "feed cat due:2020-10-09 thr:2020-10-10",
            e: "due+1m-1d",
            d: "feed cat due:2020-11-08 thr:2020-10-10",
        },
    ];
    let base = NaiveDate::from_ymd_opt(2020, 10, 12).unwrap();
    for d in data.iter() {
        let t = Task::parse(d.i, base);
        let mut tasks = vec![t];
        let mut c = Conf::default();
        c.due = DateTagChange { action: Action::Set, value: NewDateValue::Expr(d.e.to_string()) };
        let changed = edit(&mut tasks, None, &c);
        assert_eq!(changed.len(), 1, "The subject must change");
        assert!(changed[0], "The subject of the first task must change");
        assert_eq!(tasks[0].subject, d.d, "New value must be {0}, got {1}", d.d, tasks[0].subject);
    }
}
