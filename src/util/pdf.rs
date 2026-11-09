use std::path::Path;

fn enc(s: &str, max: usize) -> Vec<u8> {
    let mut o = Vec::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= max {
            break;
        }
        let b = if ch == '€' {
            0x80u8
        } else if (ch as u32) < 256 {
            ch as u8
        } else {
            b'?'
        };
        if b == b'(' || b == b')' || b == b'\\' {
            o.push(b'\\');
        }
        if b < 32 {
            o.push(b' ');
        } else {
            o.push(b);
        }
    }
    o
}

fn text(buf: &mut Vec<u8>, font: &str, size: f32, x: f32, y: f32, s: &[u8]) {
    buf.extend(format!("BT /{} {} Tf 1 0 0 1 {:.2} {:.2} Tm (", font, size, x, y).as_bytes());
    buf.extend(s);
    buf.extend(b") Tj ET\n");
}

fn obj(out: &mut Vec<u8>, offs: &mut Vec<usize>, body: &[u8]) {
    offs.push(out.len());
    out.extend(format!("{} 0 obj\n", offs.len()).as_bytes());
    out.extend(body);
    out.extend(b"\nendobj\n");
}

pub fn write(
    path: &Path,
    title: &str,
    sub: &str,
    headers: &[String],
    rows: &[Vec<String>],
) -> Result<(), String> {
    let n = headers.len().max(1);
    let mut maxlen = vec![0usize; n];
    for (i, h) in headers.iter().enumerate() {
        maxlen[i] = h.chars().count();
    }
    for r in rows {
        for (i, c) in r.iter().enumerate() {
            if i < n {
                maxlen[i] = maxlen[i].max(c.chars().count().min(40));
            }
        }
    }
    let avail = 762.0f32;
    let raw: Vec<f32> = maxlen.iter().map(|m| (*m as f32) * 4.8 + 10.0).collect();
    let total: f32 = raw.iter().sum();
    let scale = if total > avail { avail / total } else { 1.0 };
    let cw: Vec<f32> = raw.iter().map(|w| w * scale).collect();
    let per_page = 28usize;
    let pages = ((rows.len() + per_page - 1) / per_page).max(1);
    let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let mut contents: Vec<Vec<u8>> = Vec::new();
    for p in 0..pages {
        let mut b: Vec<u8> = Vec::new();
        text(&mut b, "F2", 16.0, 40.0, 552.0, &enc(title, 120));
        text(&mut b, "F1", 9.0, 40.0, 538.0, &enc(sub, 160));
        let mut y = 514.0f32;
        b.extend(format!("0.92 0.94 0.97 rg 40 {:.2} 762 16 re f 0 g\n", y - 4.0).as_bytes());
        let mut x = 40.0f32;
        for (i, h) in headers.iter().enumerate() {
            let maxc = ((cw[i] - 6.0) / 4.6).max(1.0) as usize;
            text(&mut b, "F2", 8.5, x + 4.0, y, &enc(h, maxc));
            x += cw[i];
        }
        y -= 18.0;
        for r in rows.iter().skip(p * per_page).take(per_page) {
            let mut x = 40.0f32;
            for i in 0..n {
                let cell = r.get(i).map(|s| s.as_str()).unwrap_or("");
                let maxc = ((cw[i] - 6.0) / 4.6).max(1.0) as usize;
                text(&mut b, "F1", 8.5, x + 4.0, y, &enc(cell, maxc));
                x += cw[i];
            }
            b.extend(
                format!("0.85 G 0.4 w 40 {:.2} m 802 {:.2} l S\n", y - 4.0, y - 4.0).as_bytes(),
            );
            y -= 16.0;
        }
        let foot = format!("FleetManager - {} - {}/{}", stamp, p + 1, pages);
        text(&mut b, "F1", 8.0, 40.0, 20.0, &enc(&foot, 100));
        contents.push(b);
    }
    let mut out: Vec<u8> = b"%PDF-1.4\n".to_vec();
    let mut offs: Vec<usize> = Vec::new();
    obj(&mut out, &mut offs, b"<< /Type /Catalog /Pages 2 0 R >>");
    let kids: String = (0..pages)
        .map(|i| format!("{} 0 R", 5 + 2 * i))
        .collect::<Vec<_>>()
        .join(" ");
    obj(
        &mut out,
        &mut offs,
        format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids, pages).as_bytes(),
    );
    obj(
        &mut out,
        &mut offs,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
    );
    obj(
        &mut out,
        &mut offs,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>",
    );
    for (i, c) in contents.iter().enumerate() {
        let page = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 842 595] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {} 0 R >>",
            6 + 2 * i
        );
        obj(&mut out, &mut offs, page.as_bytes());
        let mut body = format!("<< /Length {} >>\nstream\n", c.len()).into_bytes();
        body.extend(c);
        body.extend(b"endstream");
        obj(&mut out, &mut offs, &body);
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", offs.len() + 1).as_bytes());
    for o in &offs {
        out.extend(format!("{:010} 00000 n \n", o).as_bytes());
    }
    out.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
            offs.len() + 1,
            xref
        )
        .as_bytes(),
    );
    std::fs::write(path, out).map_err(|e| e.to_string())
}
