use super::Ctx;
use crate::db::reports::{self, KINDS};
use crate::i18n::tr;
use crate::ui::theme::pal;
use crate::ui::widgets::{
    self, card, empty, error_text, muted, page_header, primary, secondary, small, table, Col, Sort,
};
use crate::util::{csv, format as f, pdf};
use chrono::Datelike;
use eframe::egui::{self, RichText, Ui};

#[derive(Default)]
pub struct State {
    init: bool,
    kind: usize,
    from: String,
    to: String,
    sort: Sort,
    error: String,
}

fn range(st: &State) -> Result<(String, String), String> {
    let from = if st.from.trim().is_empty() {
        "0000-01-01".to_string()
    } else {
        f::parse_date(&st.from, &tr("From"))?
    };
    let to = if st.to.trim().is_empty() {
        "9999-12-31".to_string()
    } else {
        f::parse_date(&st.to, &tr("To"))?
    };
    if from > to {
        return Err("The start date cannot be after the end date".to_string());
    }
    Ok((from, to))
}

fn period_label(from: &str, to: &str) -> String {
    let a = if from == "0000-01-01" {
        tr("Beginning")
    } else {
        f::date(from)
    };
    let b = if to == "9999-12-31" {
        f::date(&f::today_iso())
    } else {
        f::date(to)
    };
    format!("{}: {} - {}", tr("Period"), a, b)
}

fn file_stem(kind: usize) -> String {
    let name = KINDS[kind].to_lowercase().replace(' ', "_");
    format!("fleet_{}_{}", name, chrono::Local::now().format("%Y%m%d"))
}

pub fn show(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    let p = pal();
    if !st.init {
        st.init = true;
        let t = f::today();
        if let Some(d) = chrono::NaiveDate::from_ymd_opt(t.year(), 1, 1) {
            st.from = f::to_input(d);
        }
    }
    page_header(
        ui,
        "Reports",
        "Generate reports and export them to CSV or PDF",
        |_| {},
    );
    card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            for (i, k) in KINDS.iter().enumerate() {
                let sel = st.kind == i;
                let txt = RichText::new(tr(k)).strong().color(if sel {
                    egui::Color32::WHITE
                } else {
                    p.text
                });
                let mut b = egui::Button::new(txt).rounding(egui::Rounding::same(8.0));
                if sel {
                    b = b.fill(p.accent);
                }
                if ui.add(b).clicked() {
                    st.kind = i;
                }
            }
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            widgets::form_label(ui, "From", false);
            ui.add(
                egui::TextEdit::singleline(&mut st.from)
                    .hint_text(f::date_hint())
                    .desired_width(110.0),
            );
            widgets::form_label(ui, "To", false);
            ui.add(
                egui::TextEdit::singleline(&mut st.to)
                    .hint_text(f::date_hint())
                    .desired_width(110.0),
            );
            ui.add_space(8.0);
            let today = f::today();
            let fmt = f::to_input;
            if small(ui, "This month").clicked() {
                if let Some(d) = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1) {
                    st.from = fmt(d);
                    st.to = fmt(today);
                }
            }
            if small(ui, "This year").clicked() {
                if let Some(d) = chrono::NaiveDate::from_ymd_opt(today.year(), 1, 1) {
                    st.from = fmt(d);
                    st.to = fmt(today);
                }
            }
            if small(ui, "Last 12 months").clicked() {
                if let Some(d) = today.checked_sub_signed(chrono::Duration::days(365)) {
                    st.from = fmt(d);
                    st.to = fmt(today);
                }
            }
            if small(ui, "All time").clicked() {
                st.from.clear();
                st.to.clear();
            }
        });
    });
    ui.add_space(12.0);

    let (from, to) = match range(st) {
        Ok(r) => {
            st.error.clear();
            r
        }
        Err(e) => {
            st.error = e;
            error_text(ui, &st.error);
            return;
        }
    };
    let t = reports::build(st.kind, cache, &from, &to);
    let period = period_label(&from, &to);

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(&t.title).size(18.0).strong());
            muted(ui, &period);
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if primary(ui, "Export PDF").clicked() {
                let dlg = rfd::FileDialog::new()
                    .set_title(tr("Export PDF"))
                    .set_file_name(format!("{}.pdf", file_stem(st.kind)))
                    .add_filter("PDF", &["pdf"]);
                if let Some(path) = dlg.save_file() {
                    match pdf::write(&path, &t.title, &period, &t.headers, &t.rows) {
                        Ok(()) => {
                            cx.db.log(&format!("Report exported: {} (PDF)", KINDS[st.kind]));
                            cx.toasts.success("Report exported");
                        }
                        Err(e) => cx.fail(&e),
                    }
                }
            }
            if secondary(ui, "Export CSV").clicked() {
                let dlg = rfd::FileDialog::new()
                    .set_title(tr("Export CSV"))
                    .set_file_name(format!("{}.csv", file_stem(st.kind)))
                    .add_filter("CSV", &["csv"]);
                if let Some(path) = dlg.save_file() {
                    match csv::write(&path, &t.headers, &t.rows) {
                        Ok(()) => {
                            cx.db.log(&format!("Report exported: {} (CSV)", KINDS[st.kind]));
                            cx.toasts.success("Report exported");
                        }
                        Err(e) => cx.fail(&e),
                    }
                }
            }
        });
    });
    ui.add_space(10.0);

    let data_rows = t.rows.len().saturating_sub(if st.kind <= 2 { 1 } else { 0 });
    if data_rows == 0 {
        empty(
            ui,
            "No data for this report",
            "Change the period or record more data",
        );
        return;
    }
    let n = t.headers.len().max(1);
    let w = (((ui.available_width() - 12.0) / n as f32).floor()).max(90.0);
    let cols: Vec<Col> = t.headers.iter().map(|h| Col::new(h, w)).collect();
    let total = tr("TOTAL");
    let rows = t.rows.len();
    table(ui, "report_table", &cols, rows, &mut st.sort, |ui, r, c| {
        let row = &t.rows[r];
        let txt = row.get(c).map(|s| s.as_str()).unwrap_or("");
        if row.first().map(|s| s == &total).unwrap_or(false) {
            widgets::cell_strong(ui, txt);
        } else if c == 0 {
            widgets::cell_strong(ui, txt);
        } else {
            widgets::cell(ui, txt);
        }
    });
}
