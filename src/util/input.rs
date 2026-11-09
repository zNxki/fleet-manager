use super::format;

pub fn required(txt: &str, field: &str) -> Result<String, String> {
    let t = txt.trim();
    if t.is_empty() {
        Err(format!("{}: required", field))
    } else {
        Ok(t.to_string())
    }
}

pub fn opt_num(txt: &str, field: &str) -> Result<f64, String> {
    if txt.trim().is_empty() {
        Ok(0.0)
    } else {
        format::parse_num(txt, field)
    }
}

pub fn opt_int(txt: &str, field: &str) -> Result<i64, String> {
    if txt.trim().is_empty() {
        Ok(0)
    } else {
        format::parse_int(txt, field)
    }
}

pub fn email_ok(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return true;
    }
    match t.split_once('@') {
        Some((a, b)) => {
            !a.is_empty()
                && b.contains('.')
                && !b.starts_with('.')
                && !b.ends_with('.')
                && !t.contains(' ')
        }
        None => false,
    }
}

pub fn pick_file(title: &str, filter_name: &str, exts: &[&str]) -> Option<String> {
    let mut d = rfd::FileDialog::new().set_title(title);
    if !exts.is_empty() {
        d = d.add_filter(filter_name, exts);
    }
    d.pick_file().map(|p| p.to_string_lossy().to_string())
}
