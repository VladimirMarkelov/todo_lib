use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate};

/// Empty priority - means a todo do not have any priority set
pub const NO_PRIORITY: u8 = 26;
pub const DUE_TAG: &str = "due";
pub const THR_TAG: &str = "t";
pub const REC_TAG: &str = "rec";
pub const DUE_TAG_FULL: &str = "due:";
pub const THR_TAG_FULL: &str = "t:";
pub const REC_TAG_FULL: &str = "rec:";

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Period {
    Day,
    Week,
    Month,
    Year,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Recurrence {
    pub period: Period,
    pub count: u8,
    pub strict: bool,
}

pub fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        2 => {
            if y % 4 == 0 {
                if y % 100 == 0 && y % 400 != 0 {
                    28
                } else {
                    29
                }
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Split tag into its name and value if possible.
/// Input string must be a correct tag: "name:value", where name contains only alpha-numeric
/// characters(Unicode letters are supported), and value is a non-empty string.
/// If a string is incorrect tag, the function returns None.
pub fn split_tag(s: &str) -> Option<(&str, &str)> {
    if s.is_empty() {
        return None;
    }
    if let Some(pos) = s.find(':') {
        if pos > 0 && pos < s.len() - 1 {
            Some((&s[..pos], &s[pos + 1..]))
        } else {
            None
        }
    } else {
        None
    }
}

/// Parse a string as a priority, returns an error if the string is not a valid priority.
/// A string must be a capital Latin letter enclosed in parentheses.
pub fn parse_priority(s: &str) -> Result<u8, String> {
    if s.len() != 3 {
        return Err(format!("invalid priority '{s}'"));
    }
    let trimmed = s.trim_matches(|c| c == ' ' || c == '(' || c == ')');
    if trimmed.len() != 1 {
        return Err(format!("invalid priority '{s}'"));
    }
    let priority = trimmed.bytes().next().expect("impossible");
    if !priority.is_ascii_uppercase() {
        return Err(format!("invalid priority '{s}'"));
    }
    Ok(priority - b'A')
}

pub fn str_to_priority(s: &str) -> u8 {
    if s.len() > 1 {
        return NO_PRIORITY;
    }
    if let Some(c) = s.chars().next() {
        char_to_priority(c)
    } else {
        NO_PRIORITY
    }
}

pub fn char_to_priority(c: char) -> u8 {
    if c.is_ascii_uppercase() {
        c as u8 - b'A'
    } else {
        NO_PRIORITY
    }
}

pub fn priority_to_char(priority: u8) -> char {
    if priority >= NO_PRIORITY {
        ' '
    } else {
        (b'A' + priority) as char
    }
}

pub fn format_priority(priority: u8) -> String {
    if priority >= NO_PRIORITY {
        String::new()
    } else {
        format!("({})", priority_to_char(priority))
    }
}

/// Input string must a date in format "Year-Month-Day".
/// If a date is incorrect one (e.g., month is greater than 12), an error is returned.
/// Special case: if "Day" is greater than the number of days in a month, but it is between 1 and
/// 31, the day is set to the last day of the month. Example, "2019-02-30" becomes "2019-02-28".
pub fn parse_date(s: &str, base: NaiveDate) -> Result<NaiveDate, String> {
    let trimmed = s.trim();

    if s.find('-').is_none() {
        match s.parse::<Recurrence>() {
            Err(_) => return Err(format!("invalid date '{s}'")),
            Ok(rec) => return Ok(rec.next_date(base)),
        }
    }

    let mut vals: Vec<u32> = Vec::new();
    for spl in trimmed.split('-') {
        match spl.parse::<u32>() {
            Err(_) => return Err(format!("invalid date '{s}'")),
            Ok(n) => vals.push(n),
        }
    }
    if vals.len() != 3 {
        return Err(format!("invalid date '{s}'"));
    }
    if vals[0] == 0 {
        return Err(format!("invalid year '{s}'"));
    }
    if vals[1] == 0 || vals[1] > 12 {
        return Err(format!("invalid month '{s}'"));
    }
    if vals[2] == 0 || vals[2] > 31 {
        return Err(format!("invalid day '{s}'"));
    }
    let mx = days_in_month(vals[0] as i32, vals[1]);
    if vals[2] > mx {
        vals[2] = mx;
    }
    match NaiveDate::from_ymd_opt(vals[0] as i32, vals[1], vals[2]) {
        Some(d) => Ok(d),
        None => Err(format!("invalid date generated '{}-{}-{}'", vals[0], vals[1], vals[2])),
    }
}

pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn extract_projects(s: &str) -> Vec<String> {
    extract_anything(&format!(" {s} "), " +")
}

pub fn extract_contexts(s: &str) -> Vec<String> {
    extract_anything(&format!(" {s} "), " @")
}

fn extract_anything(s: &str, start_from: &str) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    let mut idx = 0;
    let mut s_in = s;
    let p_len = start_from.len();
    if p_len == 0 {
        return items;
    }
    loop {
        s_in = &s_in[idx..];
        let start = match s_in.find(start_from) {
            None => break,
            Some(p) => p,
        };
        let end = s_in[start + p_len..].find(' ').expect("impossible");
        let item = &s_in[start + p_len..start + p_len + end];
        if !item.is_empty() && items.iter().all(|it| it != item) {
            items.push(item.to_string());
        }
        idx = start + p_len + end;
    }
    items
}

pub fn extract_tags(s: &str) -> HashMap<String, String> {
    let mut hm = HashMap::new();
    for word in s.split(' ') {
        if word.is_empty() {
            continue;
        }
        if let Some((name, value)) = split_tag(word) {
            hm.insert(name.to_string(), value.to_string());
        }
    }
    hm
}

pub fn extract_hashtags(s: &str) -> Vec<String> {
    let mut hashtags = Vec::new();
    for word in s.split(' ') {
        if word.starts_with('#') {
            hashtags.push(word.trim_start_matches('#').to_string());
        }
    }
    hashtags
}

/// Replaces a word with another one. If `new` is empty, it removed the old value.
/// A word is a group of characters between spaces(start and end of the string are virtual spaces).
pub fn replace_word(s: &mut String, old: &str, new: &str) {
    if old == new {
        return;
    }
    if s == old {
        s.replace_range(.., new);
        return;
    }
    if s.starts_with(&format!("{old} ")) {
        let l = if new.is_empty() { old.len() + 1 } else { old.len() };
        s.replace_range(..l, new);
    }
    if s.ends_with(&format!(" {old}")) {
        let l = if new.is_empty() { old.len() + 1 } else { old.len() };
        s.replace_range(s.len() - l.., new);
    }
    if new.is_empty() {
        *s = s.replace(&format!(" {old} "), " ");
    } else {
        *s = s.replace(&format!(" {old} "), &format!(" {new} "));
    }
}

impl Default for Recurrence {
    fn default() -> Self {
        Recurrence { period: Period::Day, count: 0, strict: false }
    }
}

impl std::str::FromStr for Recurrence {
    type Err = String;
    fn from_str(s: &str) -> Result<Recurrence, String> {
        Recurrence::parse(s)
    }
}

impl std::fmt::Display for Recurrence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(REC_TAG_FULL)?;
        if self.strict {
            f.write_str("+")?;
        }
        f.write_fmt(format_args!("{}", self.count))?;
        match self.period {
            Period::Day => f.write_str("d"),
            Period::Week => f.write_str("w"),
            Period::Month => f.write_str("m"),
            Period::Year => f.write_str("y"),
        }
    }
}

impl Recurrence {
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = if let Some(stripped) = s.strip_prefix(REC_TAG_FULL) { stripped } else { s };
        let mut rec = Recurrence::default();
        if s.ends_with('d') {
            rec.period = Period::Day;
        } else if s.ends_with('w') {
            rec.period = Period::Week;
        } else if s.ends_with('m') {
            rec.period = Period::Month;
        } else if s.ends_with('y') {
            rec.period = Period::Year;
        } else {
            return Err(format!("invalid recurrence '{s}'"));
        }
        if s.starts_with('+') {
            rec.strict = true;
        }
        let num = s[..s.len() - 1].parse::<u8>();
        match num {
            Err(_) => Err(format!("invalid recurrence '{s}'")),
            Ok(n) => {
                rec.count = n;
                Ok(rec)
            }
        }
    }

    /// Returns the "base" date increased by a recurrence value.
    /// Special case: when recurrence value is the number of months or years, and the "base" date
    /// is the last day of the month, the next date is always the end of a month.
    pub fn next_date(&self, base: chrono::NaiveDate) -> chrono::NaiveDate {
        let last = base.day() == days_in_month(base.year(), base.month());
        match self.period {
            Period::Day => base + Duration::days(self.count as i64),
            Period::Week => base + Duration::weeks(self.count as i64),
            Period::Month => {
                let mut y = base.year();
                let mut m = base.month() + self.count as u32;
                let mut d = base.day();
                if m > 12 {
                    y += ((m - 1) / 12) as i32;
                    m = (m - 1) % 12 + 1;
                }
                let mx = days_in_month(y, m);
                if (last && mx != d) || (mx < d) {
                    d = mx;
                }
                if let Some(d) = NaiveDate::from_ymd_opt(y, m, d) {
                    d
                } else {
                    base
                }
            }
            Period::Year => {
                let y = base.year() + self.count as i32;
                let m = base.month();
                let mut d = base.day();
                let mx = days_in_month(y, m);
                if (last && mx != d) || (mx < d) {
                    d = mx;
                }
                if let Some(d) = NaiveDate::from_ymd_opt(y, m, d) {
                    d
                } else {
                    base
                }
            }
        }
    }
}
