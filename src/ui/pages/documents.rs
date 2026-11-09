use super::Ctx;
use crate::db::models::*;
use crate::i18n::tr;
use crate::ui::dialogs::{self, Action};
use crate::ui::theme::{level_color, pal, tint};
use crate::ui::widgets::{
    self, apply_sort, cell, cell_muted, cell_strong, contains_ci, empty, error_text, expiry_level,
    filter_combo, level_badge, muted, page_header, primary, row_actions, search, small, table,
    tabs, Col, Sort,
};
use crate::util::{files, format as f, input};
use eframe::egui::{self, RichText, Ui};
use std::cmp::Ordering;

const OWNER_FILTER: [&str; 2] = ["Vehicles", "Drivers"];
const VALIDITY_FILTER: [&str; 3] = ["Expired", "Expiring soon", "Valid"];

#[derive(Default)]
pub struct State {
    tab: usize,
    search: String,
    kind: String,
    owner: String,
    validity: String,
    sort: Sort,
    dl_sort: Sort,
    form: Option<Form>,
}

#[derive(Clone, Default)]
struct Form {
    id: i64,
    for_driver: bool,
    vehicle_id: i64,
    driver_id: i64,
    doc_type: String,
    number: String,
    issuer: String,
    issue_date: String,
    expiry_date: String,
    file_path: String,
    file_new: String,
    notes: String,
    error: String,
}

impl Form {
    fn blank() -> Form {
        Form {
            doc_type: DOC_TYPES[0].to_string(),
            issue_date: f::today_input(),
            ..Default::default()
        }
    }

    fn from(d: &Document) -> Form {
        Form {
            id: d.id,
            for_driver: d.driver_id != 0,
            vehicle_id: d.vehicle_id,
            driver_id: d.driver_id,
            doc_type: d.doc_type.clone(),
            number: d.number.clone(),
            issuer: d.issuer.clone(),
            issue_date: f::date(&d.issue_date),
            expiry_date: f::date(&d.expiry_date),
            file_path: d.file_path.clone(),
            file_new: String::new(),
            notes: d.notes.clone(),
            error: String::new(),
        }
    }

    fn build(&self) -> Result<Document, String> {
        if self.for_driver && self.driver_id == 0 {
            return Err("Select a driver".to_string());
        }
        if !self.for_driver && self.vehicle_id == 0 {
            return Err("Select a vehicle".to_string());
        }
        let issue_date = f::parse_date(&self.issue_date, &tr("Issue date"))?;
        let expiry_date = f::require_date(&self.expiry_date, &tr("Expiry date"))?;
        if !issue_date.is_empty() && expiry_date < issue_date {
            return Err("The expiry date cannot precede the issue date".to_string());
        }
        Ok(Document {
            id: self.id,
            vehicle_id: if self.for_driver { 0 } else { self.vehicle_id },
            driver_id: if self.for_driver { self.driver_id } else { 0 },
            doc_type: self.doc_type.clone(),
            number: self.number.trim().to_string(),
            issuer: self.issuer.trim().to_string(),
            issue_date,
            expiry_date,
            file_path: self.file_path.clone(),
            notes: self.notes.trim().to_string(),
        })
    }
}

fn validity(d: &Document) -> &'static str {
    match expiry_level(&d.expiry_date).0 {
        2 => "Expired",
        1 => "Expiring soon",
        _ => "Valid",
    }
}

fn subject(cx: &Ctx, d: &Document) -> String {
    if d.vehicle_id != 0 {
        cx.cache.vehicle_name(d.vehicle_id)
    } else {
        cx.cache.driver_name(d.driver_id)
    }
}

fn summary(ui: &mut Ui, cx: &Ctx) {
    let p = pal();
    let cache = cx.cache;
    let expired = cache.documents.iter().filter(|d| validity(d) == "Expired").count();
    let expiring = cache
        .documents
        .iter()
        .filter(|d| validity(d) == "Expiring soon")
        .count();
    let valid = cache.documents.len() - expired - expiring;
    if expired > 0 {
        egui::Frame::none()
            .fill(tint(p.danger, 26))
            .stroke(egui::Stroke::new(1.0, tint(p.danger, 120)))
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(egui::Margin::symmetric(14.0, 9.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        expired,
                        tr("documents are expired and need to be renewed")
                    ))
                    .color(p.danger)
                    .strong(),
                );
            });
        ui.add_space(10.0);
    } else if expiring > 0 {
        egui::Frame::none()
            .fill(tint(p.warn, 26))
            .stroke(egui::Stroke::new(1.0, tint(p.warn, 120)))
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(egui::Margin::symmetric(14.0, 9.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        expiring,
                        tr("documents expire within 30 days")
                    ))
                    .color(p.warn)
                    .strong(),
                );
            });
        ui.add_space(10.0);
    }
    ui.horizontal(|ui| {
        widgets::metric(ui, "Total documents", &cache.documents.len().to_string(), 150.0);
        widgets::metric(ui, "Valid", &valid.to_string(), 110.0);
        widgets::metric(ui, "Expiring soon", &expiring.to_string(), 130.0);
        widgets::metric(ui, "Expired", &expired.to_string(), 110.0);
    });
    ui.add_space(6.0);
}

pub fn show(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    page_header(
        ui,
        "Documents",
        "Insurance, inspections, registrations and deadlines",
        |ui| {
            if primary(ui, "+ Add document").clicked() {
                st.form = Some(Form::blank());
            }
        },
    );
    summary(ui, cx);
    tabs(ui, &mut st.tab, &["Documents", "Deadlines"]);
    if st.tab == 0 {
        docs_tab(ui, cx, st);
    } else {
        deadlines_tab(ui, cx, st);
    }
    dialog(ui, cx, st);
}

fn docs_tab(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    ui.horizontal(|ui| {
        search(ui, &mut st.search);
        filter_combo(ui, "docf_kind", &mut st.kind, &DOC_TYPES, "All types");
        filter_combo(ui, "docf_owner", &mut st.owner, &OWNER_FILTER, "All owners");
        filter_combo(
            ui,
            "docf_valid",
            &mut st.validity,
            &VALIDITY_FILTER,
            "All validities",
        );
    });
    ui.add_space(10.0);
    let mut idx: Vec<usize> = cache
        .documents
        .iter()
        .enumerate()
        .filter(|(_, d)| {
            let owner = if d.vehicle_id != 0 { "Vehicles" } else { "Drivers" };
            (st.kind.is_empty() || d.doc_type == st.kind)
                && (st.owner.is_empty() || owner == st.owner)
                && (st.validity.is_empty() || validity(d) == st.validity)
                && (contains_ci(&d.number, &st.search)
                    || contains_ci(&d.issuer, &st.search)
                    || contains_ci(&d.notes, &st.search)
                    || contains_ci(&subject(cx, d), &st.search))
        })
        .map(|(i, _)| i)
        .collect();
    apply_sort(&mut idx, st.sort, |col, a, b| {
        let (x, y) = (&cache.documents[a], &cache.documents[b]);
        match col {
            0 => x.doc_type.cmp(&y.doc_type),
            1 => subject(cx, x).cmp(&subject(cx, y)),
            2 => x.number.cmp(&y.number),
            3 => x.issuer.to_lowercase().cmp(&y.issuer.to_lowercase()),
            4 => x.issue_date.cmp(&y.issue_date),
            5 => x.expiry_date.cmp(&y.expiry_date),
            _ => Ordering::Equal,
        }
    });
    let mut act: Option<(u8, i64)> = None;
    if cache.documents.is_empty() {
        empty(
            ui,
            "No documents yet",
            "Add insurance, inspection and registration documents to track their deadlines",
        );
    } else if idx.is_empty() {
        empty(ui, "No results", "Try changing the search or the filters");
    } else {
        let cols = [
            Col::new("Type", 130.0).sortable(),
            Col::new("Subject", 0.0).sortable(),
            Col::new("Number", 130.0).sortable(),
            Col::new("Issuer", 140.0).sortable(),
            Col::new("Issue date", 105.0).sortable(),
            Col::new("Expiry date", 180.0).sortable(),
            Col::new("File", 80.0),
            Col::new("Actions", 150.0),
        ];
        let n = idx.len();
        table(ui, "documents_table", &cols, n, &mut st.sort, |ui, r, c| {
            let d = &cache.documents[idx[r]];
            match c {
                0 => cell_strong(ui, &tr(&d.doc_type)),
                1 => cell(ui, &subject(cx, d)),
                2 => cell(ui, &d.number),
                3 => cell(ui, &d.issuer),
                4 => cell(
                    ui,
                    &if d.issue_date.is_empty() {
                        "-".to_string()
                    } else {
                        f::date(&d.issue_date)
                    },
                ),
                5 => {
                    let (l, t) = expiry_level(&d.expiry_date);
                    level_badge(ui, l, &format!("{} - {}", f::date(&d.expiry_date), t));
                }
                6 => {
                    if d.file_path.is_empty() {
                        cell_muted(ui, "-");
                    } else if small(ui, "Open file").clicked() {
                        act = Some((1, d.id));
                    }
                }
                _ => {
                    let (_, e, x) = row_actions(ui, false, true, true);
                    if e {
                        act = Some((2, d.id));
                    }
                    if x {
                        act = Some((3, d.id));
                    }
                }
            }
        });
    }
    if let Some((kind, id)) = act {
        if let Some(d) = cache.documents.iter().find(|d| d.id == id) {
            match kind {
                1 => {
                    if std::path::Path::new(&d.file_path).exists() {
                        files::open_path(&d.file_path);
                    } else {
                        cx.fail("The attached file no longer exists");
                    }
                }
                2 => st.form = Some(Form::from(d)),
                _ => {
                    let msg = format!(
                        "{} {} ({})?",
                        tr("Delete document"),
                        tr(&d.doc_type),
                        subject(cx, d)
                    );
                    cx.ask(
                        "Delete document",
                        &msg,
                        "Delete",
                        Action::DeleteDocument(id),
                        format!("Document deleted: {} ({})", d.doc_type, subject(cx, d)),
                    );
                }
            }
        }
    }
}

fn deadlines_tab(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    let p = pal();
    if cache.deadlines.is_empty() {
        empty(
            ui,
            "No deadlines",
            "Documents, licenses and maintenance with a due date appear here",
        );
        return;
    }
    muted(
        ui,
        &tr("Deadlines are sorted by urgency. Expired items are shown first."),
    );
    ui.add_space(8.0);
    let cols = [
        Col::new("Category", 130.0),
        Col::new("Subject", 160.0),
        Col::new("Item", 0.0),
        Col::new("Due", 200.0),
        Col::new("Status", 170.0),
    ];
    let n = cache.deadlines.len();
    table(ui, "deadlines_table", &cols, n, &mut st.dl_sort, |ui, r, c| {
        let d = &cache.deadlines[r];
        match c {
            0 => cell(ui, &tr(&d.kind)),
            1 => cell_strong(ui, &d.subject),
            2 => cell(ui, &tr(&d.label)),
            3 => {
                ui.label(RichText::new(&d.detail).color(p.muted));
            }
            _ => {
                let text = if d.days == i64::MAX {
                    tr(match d.level {
                        2 => "Overdue",
                        1 => "Due soon",
                        _ => "Scheduled",
                    })
                } else {
                    widgets::due_label(d.days)
                };
                level_badge(ui, d.level, &text);
                let _ = level_color(d.level);
            }
        }
    });
}

fn dialog(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    let mut close = false;
    if let Some(form) = st.form.as_mut() {
        let title = if form.id == 0 {
            "Add document"
        } else {
            "Edit document"
        };
        let vehicles = cache.vehicle_items(true);
        let drivers = cache.driver_items(true);
        let mut cancel = false;
        let mut save = false;
        let closed = dialogs::modal(ui.ctx(), "doc_form", title, 700.0, |ui| {
            error_text(ui, &form.error);
            ui.horizontal(|ui| {
                widgets::form_label(ui, "Belongs to", true);
                if ui
                    .selectable_label(!form.for_driver, tr("Vehicle"))
                    .clicked()
                {
                    form.for_driver = false;
                }
                if ui.selectable_label(form.for_driver, tr("Driver")).clicked() {
                    form.for_driver = true;
                }
            });
            ui.add_space(8.0);
            egui::Grid::new("doc_form_grid")
                .num_columns(4)
                .spacing([14.0, 10.0])
                .show(ui, |ui| {
                    if form.for_driver {
                        widgets::form_label(ui, "Driver", true);
                        widgets::combo_id(
                            ui,
                            "doc_form_driver",
                            &mut form.driver_id,
                            &drivers,
                            None,
                            220.0,
                        );
                    } else {
                        widgets::form_label(ui, "Vehicle", true);
                        widgets::combo_id(
                            ui,
                            "doc_form_vehicle",
                            &mut form.vehicle_id,
                            &vehicles,
                            None,
                            220.0,
                        );
                    }
                    widgets::form_combo(
                        ui,
                        "Type",
                        true,
                        "doc_form_type",
                        &mut form.doc_type,
                        &DOC_TYPES,
                        220.0,
                    );
                    ui.end_row();
                    widgets::form_text(ui, "Number", false, &mut form.number, 220.0);
                    widgets::form_text(ui, "Issuer", false, &mut form.issuer, 220.0);
                    ui.end_row();
                    widgets::form_date(ui, "Issue date", false, &mut form.issue_date, 220.0);
                    widgets::form_date(ui, "Expiry date", true, &mut form.expiry_date, 220.0);
                    ui.end_row();
                });
            ui.add_space(8.0);
            widgets::form_label(ui, "Attached file", false);
            ui.horizontal(|ui| {
                let shown = if !form.file_new.is_empty() {
                    files::display_name(&form.file_new)
                        .rsplit(['/', '\\'])
                        .next()
                        .unwrap_or("")
                        .to_string()
                } else if !form.file_path.is_empty() {
                    files::display_name(&form.file_path)
                } else {
                    tr("No file attached")
                };
                ui.label(widgets::cut(&shown, 50));
                if widgets::secondary(ui, "Attach file").clicked() {
                    if let Some(p) = input::pick_file(
                        "Select document",
                        "Documents",
                        &["pdf", "png", "jpg", "jpeg", "doc", "docx", "txt"],
                    ) {
                        form.file_new = p;
                    }
                }
                if widgets::secondary(ui, "Remove").clicked() {
                    form.file_new.clear();
                    form.file_path.clear();
                }
            });
            ui.add_space(8.0);
            egui::Grid::new("doc_form_notes")
                .num_columns(2)
                .spacing([14.0, 10.0])
                .show(ui, |ui| {
                    widgets::form_multiline(ui, "Notes", &mut form.notes, 600.0);
                    ui.end_row();
                });
            dialogs::footer(ui, "Save", &mut cancel, &mut save);
        });
        if closed || cancel {
            close = true;
        }
        if save {
            match form.build() {
                Ok(mut d) => {
                    let mut ok = true;
                    if !form.file_new.is_empty() {
                        match files::attach(&form.file_new) {
                            Ok(p) => d.file_path = p,
                            Err(e) => {
                                form.error = e;
                                ok = false;
                            }
                        }
                    }
                    if ok {
                        match cx.db.save_document(&d) {
                            Ok(_) => {
                                let who = if d.vehicle_id != 0 {
                                    cache.vehicle_name(d.vehicle_id)
                                } else {
                                    cache.driver_name(d.driver_id)
                                };
                                let verb = if d.id == 0 { "added" } else { "updated" };
                                let log = format!("Document {}: {} ({})", verb, d.doc_type, who);
                                cx.saved(&log);
                                close = true;
                            }
                            Err(e) => form.error = e,
                        }
                    }
                }
                Err(e) => form.error = e,
            }
        }
    }
    if close {
        st.form = None;
    }
}
