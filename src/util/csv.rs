use std::path::Path;

fn line(cells: &[String]) -> String {
    cells
        .iter()
        .map(|c| {
            if c.contains(|ch: char| ch == ',' || ch == '"' || ch == '\n' || ch == ';') {
                format!("\"{}\"", c.replace('"', "\"\""))
            } else {
                c.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub fn write(path: &Path, headers: &[String], rows: &[Vec<String>]) -> Result<(), String> {
    let mut out = String::from("\u{feff}");
    out.push_str(&line(headers));
    out.push_str("\r\n");
    for r in rows {
        out.push_str(&line(r));
        out.push_str("\r\n");
    }
    std::fs::write(path, out).map_err(|e| e.to_string())
}
