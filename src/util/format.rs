use chrono::{Local, NaiveDate};
use std::sync::atomic::{AtomicUsize, Ordering};

pub const DATE_FORMATS: [(&str, &str); 3] = [
    ("DD/MM/YYYY", "%d/%m/%Y"),
    ("YYYY-MM-DD", "%Y-%m-%d"),
    ("MM/DD/YYYY", "%m/%d/%Y"),
];
pub const CURRENCIES: [&str; 4] = ["€", "$", "£", "CHF"];

static DATE_IDX: AtomicUsize = AtomicUsize::new(0);
static CUR_IDX: AtomicUsize = AtomicUsize::new(0);

pub fn set_date_idx(i: usize) {
    DATE_IDX.store(i.min(2), Ordering::Relaxed);
}

pub fn set_cur_idx(i: usize) {
    CUR_IDX.store(i.min(3), Ordering::Relaxed);
}

fn pattern() -> &'static str {
    DATE_FORMATS[DATE_IDX.load(Ordering::Relaxed).min(2)].1
}

pub fn date_hint() -> &'static str {
    DATE_FORMATS[DATE_IDX.load(Ordering::Relaxed).min(2)].0
}

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

pub fn today_iso() -> String {
    today().format("%Y-%m-%d").to_string()
}

pub fn to_input(d: NaiveDate) -> String {
    d.format(pattern()).to_string()
}

pub fn today_input() -> String {
    today().format(pattern()).to_string()
}

pub fn parse_iso(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

pub fn days_until(s: &str) -> Option<i64> {
    parse_iso(s).map(|d| (d - today()).num_days())
}

pub fn date(iso: &str) -> String {
    parse_iso(iso)
        .map(|d| d.format(pattern()).to_string())
        .unwrap_or_default()
}

pub fn parse_date(txt: &str, field: &str) -> Result<String, String> {
    let t = txt.trim();
    if t.is_empty() {
        return Ok(String::new());
    }
    NaiveDate::parse_from_str(t, pattern())
        .map(|d| d.format("%Y-%m-%d").to_string())
        .map_err(|_| format!("{}: invalid date, expected {}", field, date_hint()))
}

pub fn require_date(txt: &str, field: &str) -> Result<String, String> {
    let d = parse_date(txt, field)?;
    if d.is_empty() {
        Err(format!("{}: required", field))
    } else {
        Ok(d)
    }
}

pub fn parse_num(txt: &str, field: &str) -> Result<f64, String> {
    txt.trim()
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.0)
        .ok_or_else(|| format!("{}: invalid number", field))
}

pub fn parse_int(txt: &str, field: &str) -> Result<i64, String> {
    txt.trim()
        .parse::<i64>()
        .ok()
        .filter(|v| *v >= 0)
        .ok_or_else(|| format!("{}: invalid whole number", field))
}

pub fn group(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 {
        format!("-{}", out)
    } else {
        out
    }
}

pub fn km(n: i64) -> String {
    format!("{} km", group(n))
}

pub fn num(v: f64, dec: usize) -> String {
    let s = format!("{:.*}", dec, v);
    match s.split_once('.') {
        Some((i, d)) => format!("{}.{}", group(i.parse().unwrap_or(0)), d),
        None => group(s.parse().unwrap_or(0)),
    }
}

pub fn money(v: f64) -> String {
    format!(
        "{} {}",
        CURRENCIES[CUR_IDX.load(Ordering::Relaxed).min(3)],
        num(v, 2)
    )
}
