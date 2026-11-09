mod it;

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

pub const LANGUAGES: [&str; 2] = ["English", "Italiano"];

static LANG: AtomicUsize = AtomicUsize::new(0);
static TABLE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

pub fn set_lang(i: usize) {
    LANG.store(i.min(LANGUAGES.len() - 1), Ordering::Relaxed);
}

pub fn lang() -> usize {
    LANG.load(Ordering::Relaxed)
}

fn table() -> &'static HashMap<&'static str, &'static str> {
    TABLE.get_or_init(|| it::ENTRIES.iter().copied().collect())
}

fn translate(s: &str) -> String {
    if let Some(t) = table().get(s) {
        return (*t).to_string();
    }
    if let Some((a, b)) = s.split_once(": ") {
        return format!("{}: {}", translate(a), translate(b));
    }
    if let Some((a, b)) = s.split_once(", expected ") {
        return format!("{}, {} {}", translate(a), translate("expected"), b);
    }
    if let Some(i) = s.find(" (") {
        let (a, b) = s.split_at(i);
        return format!("{}{}", translate(a), b);
    }
    s.to_string()
}

pub fn tr(s: &str) -> String {
    if lang() == 0 || s.is_empty() {
        return s.to_string();
    }
    translate(s)
}