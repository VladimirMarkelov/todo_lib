use chrono::NaiveDate;
use todo_lib::todotxt::*;

#[test]
fn priorities() {
    struct Test {
        i: &'static str,
        o: u8,
        err: bool,
    }
    let data: Vec<Test> = vec![
        Test { i: "", o: 0, err: true },
        Test { i: "(A", o: 0, err: true },
        Test { i: "(a)", o: 0, err: true },
        Test { i: "(ñ)", o: 0, err: true },
        Test { i: "a", o: 0, err: true },
        Test { i: "(A)", o: 0, err: false },
        Test { i: "(D)", o: 3, err: false },
        Test { i: "(Z)", o: NO_PRIORITY - 1, err: false },
    ];
    for d in data.iter() {
        let p = parse_priority(d.i);
        if d.err {
            assert!(p.is_err(), "{}", d.i);
        } else {
            let p = p.unwrap();
            assert_eq!(p, d.o, "{}", d.i);
            let back = format_priority(p);
            assert_eq!(&back, d.i, "{}", d.i);
        }
    }
}

#[test]
fn abs_dates() {
    struct Test {
        i: &'static str,
        o: NaiveDate,
        err: bool,
        eq: bool,
    }
    let data: Vec<Test> = vec![
        Test { i: "", o: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(), err: true, eq: true },
        Test { i: "0-1-1", o: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(), err: true, eq: true },
        Test { i: "1-0-1", o: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(), err: true, eq: true },
        Test { i: "1-1-0", o: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(), err: true, eq: true },
        Test { i: "abcde", o: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(), err: true, eq: true },
        Test { i: "1900-15-15", o: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(), err: true, eq: true },
        Test { i: "1900-11-32", o: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(), err: true, eq: true },
        Test { i: "2020-11--23", o: NaiveDate::from_ymd_opt(2020, 11, 23).unwrap(), err: true, eq: true },
        Test { i: "2020-11-23", o: NaiveDate::from_ymd_opt(2020, 11, 23).unwrap(), err: false, eq: true },
        Test { i: "2020-11-31", o: NaiveDate::from_ymd_opt(2020, 11, 30).unwrap(), err: false, eq: false },
    ];
    let base = NaiveDate::from_ymd_opt(2020, 10, 20).unwrap();
    for d in data.iter() {
        let p = parse_date(d.i, base);
        if d.err {
            assert!(p.is_err(), "{}", d.i);
        } else {
            let p = p.unwrap();
            assert_eq!(p, d.o, "{}", d.i);
            if d.eq {
                let back = format_date(p);
                assert_eq!(&back, d.i, "{}", d.i);
            }
        }
    }
}

#[test]
fn rel_dates() {
    struct Test {
        i: &'static str,
        m: NaiveDate,
        e: NaiveDate,
    }
    let base_mid = NaiveDate::from_ymd_opt(2021, 3, 15).unwrap();
    let base_end = NaiveDate::from_ymd_opt(2021, 2, 28).unwrap();
    let data: Vec<Test> = vec![
        Test {
            i: "12d",
            m: NaiveDate::from_ymd_opt(2021, 3, 27).unwrap(),
            e: NaiveDate::from_ymd_opt(2021, 3, 12).unwrap(),
        },
        Test {
            i: "1w",
            m: NaiveDate::from_ymd_opt(2021, 3, 22).unwrap(),
            e: NaiveDate::from_ymd_opt(2021, 3, 7).unwrap(),
        },
        Test {
            i: "2m",
            m: NaiveDate::from_ymd_opt(2021, 5, 15).unwrap(),
            e: NaiveDate::from_ymd_opt(2021, 4, 30).unwrap(),
        },
        Test {
            i: "3y",
            m: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            e: NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
        },
    ];
    for d in data.iter() {
        let p = parse_date(d.i, base_mid).unwrap();
        assert_eq!(p, d.m, "mid: {}", d.i);
        let p = parse_date(d.i, base_end).unwrap();
        assert_eq!(p, d.e, "end: {}", d.i);
    }
}

#[test]
fn proj_and_ctx() {
    struct Test {
        i: &'static str,
        projs: Vec<&'static str>,
        ctxs: Vec<&'static str>,
    }
    let data: Vec<Test> = vec![
        Test { i: "", projs: Vec::new(), ctxs: Vec::new() },
        Test { i: "abcd efhg", projs: Vec::new(), ctxs: Vec::new() },
        Test { i: "ab@cd ef+hg", projs: Vec::new(), ctxs: Vec::new() },
        Test { i: "abcd efhg@", projs: Vec::new(), ctxs: Vec::new() },
        Test { i: "@abcd +efhg", projs: vec!["efhg"], ctxs: vec!["abcd"] },
        Test { i: "@abcd ww +1234 +efhg zz @890", projs: vec!["efhg", "1234"], ctxs: vec!["abcd", "890"] },
        Test { i: "@abcd +efhg something +efhg @abcd", projs: vec!["efhg"], ctxs: vec!["abcd"] },
        Test { i: "+ @abcd + +efhg @", projs: vec!["efhg"], ctxs: vec!["abcd"] },
    ];
    for d in data.iter() {
        let ps = extract_projects(d.i);
        let cs = extract_contexts(d.i);
        assert_eq!(ps.len(), d.projs.len(), "{}: projects {:?} == {:?}", d.i, d.projs, ps);
        assert_eq!(cs.len(), d.ctxs.len(), "{}: contexts {:?} == {:?}", d.i, d.ctxs, cs);
        let mut eq = true;
        for p in ps.iter() {
            if !d.projs.iter().any(|pp| pp == p) {
                eq = false;
                break;
            }
        }
        for c in cs.iter() {
            if !d.ctxs.iter().any(|cc| cc == c) {
                eq = false;
                break;
            }
        }
        assert!(eq, "{}: {:?} == {:?}, {:?} == {:?}", d.i, d.projs, ps, d.ctxs, cs);
    }
}

#[test]
fn tags() {
    struct Test {
        i: &'static str,
        tag_n: Vec<&'static str>,
        tag_v: Vec<&'static str>,
    }
    let data: Vec<Test> = vec![
        Test { i: "", tag_n: Vec::new(), tag_v: Vec::new() },
        Test { i: "abcd 0123", tag_n: Vec::new(), tag_v: Vec::new() },
        Test { i: ":abcd 0123:", tag_n: Vec::new(), tag_v: Vec::new() },
        Test { i: "abcd: :0123", tag_n: Vec::new(), tag_v: Vec::new() },
        Test { i: "**:abcd ñ:0123", tag_n: vec!["**", "ñ"], tag_v: vec!["abcd", "0123"] },
        Test {
            i: "abcd test:value1 another second:value2",
            tag_n: vec!["test", "second"],
            tag_v: vec!["value1", "value2"],
        },
        Test {
            i: "abcd test:value1 test:value3 second:value2",
            tag_n: vec!["test", "second"],
            tag_v: vec!["value3", "value2"],
        },
        Test {
            i: "abcd test:value1: inner:val:ue3 second::value2",
            tag_n: vec!["test", "inner", "second"],
            tag_v: vec!["value1:", "val:ue3", ":value2"],
        },
    ];

    for d in data.iter() {
        let mp = extract_tags(d.i);
        assert_eq!(mp.len(), d.tag_n.len(), "{} - {:?}", d.i, mp);
        for (key, val) in d.tag_n.iter().zip(d.tag_v.iter()) {
            let found = mp.get(&key.to_string());
            let vv = val.to_string();
            assert_eq!(Some(&vv), found, "{}", d.i);
        }
    }
}

#[test]
fn recurrents() {
    struct Test {
        i: &'static str,
        r: Recurrence,
        e: bool,
    }
    let data: Vec<Test> = vec![
        Test { i: "djd", r: Recurrence::default(), e: true },
        Test { i: "rec:ad", r: Recurrence::default(), e: true },
        Test { i: "rec:10", r: Recurrence::default(), e: true },
        Test { i: "rec:120d", r: Recurrence { period: Period::Day, count: 120, strict: false }, e: false },
        Test { i: "rec:17w", r: Recurrence { period: Period::Week, count: 17, strict: false }, e: false },
        Test { i: "rec:+2m", r: Recurrence { period: Period::Month, count: 2, strict: true }, e: false },
        Test { i: "rec:+1y", r: Recurrence { period: Period::Year, count: 1, strict: true }, e: false },
    ];

    for d in data.iter() {
        let r = d.i.parse::<Recurrence>();
        if d.e {
            assert!(r.is_err(), "{}", d.i);
        } else {
            assert!(r.is_ok(), "{}", d.i);
            let r = r.unwrap();
            assert_eq!(d.r, r, "{} == {}", d.i, r);
            let back = format!("{}", r);
            assert_eq!(d.i, &back, "{} == {}", d.i, back);
        }
    }
}
