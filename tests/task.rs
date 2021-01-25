use chrono::NaiveDate;
use todo_lib::todotxt::Task;

#[test]
fn parse_tasks_simple() {
    struct Test {
        i: &'static str,
        t: Task,
    };
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
                create_date: Some(NaiveDate::from_ymd(2020, 01, 03)),
                ..Default::default()
            },
        },
        Test {
            i: "2020-02-03 2020-01-03 just text",
            t: Task {
                subject: "2020-01-03 just text".to_string(),
                create_date: Some(NaiveDate::from_ymd(2020, 02, 03)),
                ..Default::default()
            },
        },
        Test {
            i: "x 2020-02-03 2020-01-03 just text",
            t: Task {
                subject: "just text".to_string(),
                create_date: Some(NaiveDate::from_ymd(2020, 01, 03)),
                finish_date: Some(NaiveDate::from_ymd(2020, 02, 03)),
                finished: true,
                ..Default::default()
            },
        },
        Test {
            i: "x (E) 2020-02-03 2020-01-03 just text",
            t: Task {
                subject: "just text".to_string(),
                priority: 4,
                create_date: Some(NaiveDate::from_ymd(2020, 01, 03)),
                finish_date: Some(NaiveDate::from_ymd(2020, 02, 03)),
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
    let base = NaiveDate::from_ymd(2020, 3, 15);
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
    };
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
    let base = NaiveDate::from_ymd(2020, 3, 15);
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
fn complete_uncomplete() {
    struct Test {
        i: &'static str,
        d: &'static str,
        u: &'static str,
    };
    let data: Vec<Test> = vec![
        Test { i: "test", d: "x test", u: "test" },
        Test { i: "2020-01-01 test", d: "x 2020-02-02 2020-01-01 test", u: "2020-01-01 test" },
        Test { i: "test rec:+1m due:2020-03-01", d: "test rec:+1m due:2020-04-01", u: "test rec:+1m due:2020-04-01" },
        Test { i: "test rec:1m due:2020-03-01", d: "test rec:1m due:2020-03-02", u: "test rec:1m due:2020-03-02" },
        Test { i: "2020-01-01 test rec:7d", d: "x 2020-02-02 2020-01-01 test rec:7d", u: "2020-01-01 test rec:7d" },
    ];
    let base = NaiveDate::from_ymd(2020, 2, 2);
    for d in data.iter() {
        let mut t = Task::parse(d.i, base);
        let orig_due = t.due_date.clone();
        t.complete(base);
        assert_eq!(d.d, &format!("{}", t), "done {}", d.i);
        if t.create_date.is_some() && t.recurrence.is_none() {
            assert_eq!(t.finish_date, Some(base));
        }
        if orig_due.is_some() && t.recurrence.is_some() && t.due_date.is_some() {
            let orig = orig_due.unwrap();
            assert!(orig < t.due_date.unwrap(), "due must change: {}", d.i);
        }
        t.uncomplete();
        assert_eq!(d.u, &format!("{}", t), "undone {}", d.i);
    }
}

#[test]
fn replace_projects() {
    struct Test {
        i: &'static str,
        o: &'static str,
        r: &'static str,
        w: &'static str,
    };
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

    let dt = NaiveDate::from_ymd(2021, 01, 05);
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
    };
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

    let dt = NaiveDate::from_ymd(2021, 01, 05);
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
    };
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

    let dt = NaiveDate::from_ymd(2021, 01, 05);
    for d in data.iter() {
        let mut t = Task::parse(d.i, dt);
        t.update_tag_with_value("rec", d.w);
        assert_eq!(d.o, &t.subject, "{}-> {}", d.i, d.w);
    }
}
