use super::Ctx;
use crate::db::cache::Cache;
use crate::db::models::*;
use crate::i18n::tr;
use crate::ui::dialogs::{self, Action};
use crate::ui::theme::{level_color, pal};
use crate::ui::widgets::{
    self, apply_sort, card, cell, cell_strong, contains_ci, empty, error_text, filter_combo,
    filter_combo_id, muted, page_header, primary, row_actions, search, section_title, small,
    status_badge, table, Col, Sort,
};
use crate::util::{format as f, input};
use eframe::egui::{self, Align, Layout, RichText, Ui};
use std::cmp::Ordering;

#[derive(Default)]
pub struct State {
    search: String,
    status: String,
    kind: String,
    vehicle: i64,
    sort: Sort,
    form: Option<Form>,
}

#[derive(Clone, Default)]
struct Form {
    id: i64,
    vehicle_id: i64,
    kind: String,
    description: String,
    provider: String,
    scheduled_date: String,
    completed_date: String,
    cost: String,
    mileage: String,
    status: String,
    next_due_km: String,
    next_due_date: String,
    error: String,
}

impl Form {
    fn blank() -> Form {
        Form {
            kind: MAINT_KINDS[0].to_string(),
            status: MAINT_STATUS[0].to_string(),
            scheduled_date: f::today_input(),
            cost: "0".to_string(),
            ..Default::default()
        }
    }

    fn from(m: &Maintenance) -> Form {
        Form {
            id: m.id,
            vehicle_id: m.vehicle_id,
            kind: m.kind.clone(),
            description: m.description.clone(),
            provider: m.provider.clone(),
            scheduled_date: f::date(&m.scheduled_date),
            completed_date: f::date(&m.completed_date),
            cost: f::num(m.cost, 2).replace(',', ""),
            mileage: if m.mileage > 0 {
                m.mileage.to_string()
            } else {
                String::new()
            },
            status: m.status.clone(),
            next_due_km: if m.next_due_km > 0 {
                m.next_due_km.to_string()
            } else {
                String::new()
            },
            next_due_date: f::date(&m.next_due_date),
            error: String::new(),
        }
    }

    fn complete(m: &Maintenance, vehicle_km: i64) -> Form {
        let mut x = Form::from(m);
        x.status = "Completed".to_string();
        x.completed_date = f::today_input();
        if m.mileage == 0 {
            x.mileage = vehicle_km.to_string();
        }
        x
    }

    fn build(&self) -> Result<Maintenance, String> {
        if self.vehicle_id == 0 {
            return Err("Select a vehicle".to_string());
        }
        let description = input::required(&self.description, &tr("Description"))?;
        let scheduled_date = f::parse_date(&self.scheduled_date, &tr("Scheduled date"))?;
        let completed_date = if self.status == "Completed" {
            f::require_date(&self.completed_date, &tr("Completion date"))?
        } else {
            String::new()
        };
        if self.status != "Completed" && scheduled_date.is_empty() {
            return Err("The scheduled date is required".to_string());
        }
        let next_due_date = f::parse_date(&self.next_due_date, &tr("Next due date"))?;
        let base = if completed_date.is_empty() {
            scheduled_date.clone()
        } else {
            completed_date.clone()
        };
        if !next_due_date.is_empty() && !base.is_empty() && next_due_date < base {
            return Err("The next due date cannot precede the maintenance date".to_string());
        }
        let mileage = input::opt_int(&self.mileage, &tr("Mileage"))?;
        let next_due_km = input::opt_int(&self.next_due_km, &tr("Next due mileage"))?;
        if next_due_km > 0 && mileage > 0 && next_due_km <= mileage {
            return Err("The next due mileage must be greater than the current mileage".to_string());
        }
        Ok(Maintenance {
            id: self.id,
            vehicle_id: self.vehicle_id,
            kind: self.kind.clone(),
            description,
            provider: self.provider.trim().to_string(),
            scheduled_date,
            completed_date,
            cost: input::opt_num(&self.cost, &tr("Cost"))?,
            mileage,
            status: self.status.clone(),
            next_due_km,
            next_due_date,
        })
    }
}

fn upcoming(ui: &mut Ui, cache: &Cache) {
    let p = pal();
    let list: Vec<_> = cache
        .deadlines
        .iter()
        .filter(|d| d.kind == "Maintenance" && (d.level > 0 || d.days <= 60))
        .take(5)
        .collect();
    card(ui, |ui| {
        ui.set_width(ui.available_width());
        section_title(ui, "Upcoming maintenance");
        ui.add_space(4.0);
        if list.is_empty() {
            muted(ui, &tr("No upcoming maintenance"));
        }
        for d in list {
            let c = level_color(d.level);
            ui.horizontal(|ui| {
                ui.label(RichText::new(widgets::cut(&d.subject, 14)).strong());
                ui.label(RichText::new(widgets::cut(&d.label, 56)).color(p.muted));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let t = if d.days == i64::MAX {
                        d.detail.clone()
                    } else {
                        format!("{} - {}", d.detail, widgets::due_label(d.days))
                    };
                    ui.label(RichText::new(t).color(c).strong());
                });
            });
        }
    });
    ui.add_space(12.0);
}

pub fn show(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    page_header(
        ui,
        "Maintenance",
        "Schedule services and repairs and track their history",
        |ui| {
            if primary(ui, "+ Schedule maintenance").clicked() {
                st.form = Some(Form::blank());
            }
        },
    );

    upcoming(ui, cache);

    let vehicles = cache.vehicle_items(true);
    ui.horizontal(|ui| {
        search(ui, &mut st.search);
        filter_combo(
            ui,
            "mf_status",
            &mut st.status,
            &MAINT_STATUS,
            "All statuses",
        );
        filter_combo(ui, "mf_kind", &mut st.kind, &MAINT_KINDS, "All types");
        filter_combo_id(ui, "mf_vehicle", &mut st.vehicle, &vehicles, "All vehicles");
    });
    ui.add_space(10.0);

    let date_of = |m: &Maintenance| -> String {
        if m.completed_date.is_empty() {
            m.scheduled_date.clone()
        } else {
            m.completed_date.clone()
        }
    };
    let mut idx: Vec<usize> = cache
        .maintenance
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            (st.status.is_empty() || m.status == st.status)
                && (st.kind.is_empty() || m.kind == st.kind)
                && (st.vehicle == 0 || m.vehicle_id == st.vehicle)
                && (contains_ci(&m.description, &st.search)
                    || contains_ci(&m.provider, &st.search)
                    || contains_ci(&cache.vehicle_name(m.vehicle_id), &st.search))
        })
        .map(|(i, _)| i)
        .collect();
    apply_sort(&mut idx, st.sort, |col, a, b| {
        let (x, y) = (&cache.maintenance[a], &cache.maintenance[b]);
        match col {
            0 => cache
                .vehicle_name(x.vehicle_id)
                .cmp(&cache.vehicle_name(y.vehicle_id)),
            1 => x.kind.cmp(&y.kind),
            2 => x.description.to_lowercase().cmp(&y.description.to_lowercase()),
            3 => x.provider.to_lowercase().cmp(&y.provider.to_lowercase()),
            4 => date_of(x).cmp(&date_of(y)),
            5 => x.cost.partial_cmp(&y.cost).unwrap_or(Ordering::Equal),
            6 => x.status.cmp(&y.status),
            _ => Ordering::Equal,
        }
    });

    let mut act: Option<(u8, i64)> = None;
    if cache.maintenance.is_empty() {
        empty(
            ui,
            "No maintenance records yet",
            "Schedule a service or register a repair",
        );
    } else if idx.is_empty() {
        empty(ui, "No results", "Try changing the search or the filters");
    } else {
        let cols = [
            Col::new("Vehicle", 110.0).sortable(),
            Col::new("Type", 130.0).sortable(),
            Col::new("Description", 0.0).sortable(),
            Col::new("Provider", 140.0).sortable(),
            Col::new("Date", 105.0).sortable(),
            Col::new("Cost", 110.0).sortable(),
            Col::new("Status", 110.0).sortable(),
            Col::new("Actions", 250.0),
        ];
        let n = idx.len();
        table(
            ui,
            "maintenance_table",
            &cols,
            n,
            &mut st.sort,
            |ui, r, c| {
                let m = &cache.maintenance[idx[r]];
                match c {
                    0 => cell_strong(ui, &cache.vehicle_name(m.vehicle_id)),
                    1 => cell(ui, &tr(&m.kind)),
                    2 => cell(ui, &m.description),
                    3 => cell(ui, &m.provider),
                    4 => cell(ui, &f::date(&date_of(m))),
                    5 => cell(ui, &f::money(m.cost)),
                    6 => status_badge(ui, &m.status),
                    _ => {
                        if m.status != "Completed" && small(ui, "Complete").clicked() {
                            act = Some((1, m.id));
                        }
                        let (_, e, d) = row_actions(ui, false, true, true);
                        if e {
                            act = Some((2, m.id));
                        }
                        if d {
                            act = Some((3, m.id));
                        }
                    }
                }
            },
        );
    }

    if let Some((kind, id)) = act {
        if let Some(m) = cache.maintenance.iter().find(|m| m.id == id) {
            match kind {
                1 => {
                    let km = cache.vehicle(m.vehicle_id).map(|v| v.mileage).unwrap_or(0);
                    st.form = Some(Form::complete(m, km));
                }
                2 => st.form = Some(Form::from(m)),
                _ => {
                    let msg = format!(
                        "{} {} - {}?",
                        tr("Delete maintenance"),
                        cache.vehicle_name(m.vehicle_id),
                        m.description
                    );
                    cx.ask(
                        "Delete maintenance",
                        &msg,
                        "Delete",
                        Action::DeleteMaintenance(id),
                        format!(
                            "Maintenance deleted: {} - {}",
                            cache.vehicle_name(m.vehicle_id),
                            m.description
                        ),
                    );
                }
            }
        }
    }

    let mut close = false;
    if let Some(form) = st.form.as_mut() {
        let title = if form.id == 0 {
            "Schedule maintenance"
        } else {
            "Edit maintenance"
        };
        let items = cache.vehicle_items(form.id != 0);
        let mut cancel = false;
        let mut save = false;
        let closed = dialogs::modal(ui.ctx(), "maint_form", title, 760.0, |ui| {
            form_ui(ui, form, &items);
            dialogs::footer(ui, "Save", &mut cancel, &mut save);
        });
        if closed || cancel {
            close = true;
        }
        if save {
            match form.build() {
                Ok(m) => match cx.db.save_maintenance(&m) {
                    Ok(_) => {
                        let verb = if m.id == 0 { "added" } else { "updated" };
                        let log = format!(
                            "Maintenance {}: {} - {}",
                            verb,
                            cache.vehicle_name(m.vehicle_id),
                            m.description
                        );
                        cx.saved(&log);
                        close = true;
                    }
                    Err(e) => form.error = e,
                },
                Err(e) => form.error = e,
            }
        }
    }
    if close {
        st.form = None;
    }
}

fn form_ui(ui: &mut Ui, form: &mut Form, vehicles: &[(i64, String)]) {
    error_text(ui, &form.error);
    egui::Grid::new("maint_form_a")
        .num_columns(4)
        .spacing([14.0, 10.0])
        .show(ui, |ui| {
            widgets::form_label(ui, "Vehicle", true);
            widgets::combo_id(ui, "mf_form_vehicle", &mut form.vehicle_id, vehicles, None, 220.0);
            widgets::form_combo(
                ui,
                "Type",
                true,
                "mf_form_kind",
                &mut form.kind,
                &MAINT_KINDS,
                220.0,
            );
            ui.end_row();
            widgets::form_text(ui, "Description", true, &mut form.description, 220.0);
            widgets::form_text(ui, "Service provider", false, &mut form.provider, 220.0);
            ui.end_row();
            widgets::form_combo(
                ui,
                "Status",
                true,
                "mf_form_status",
                &mut form.status,
                &MAINT_STATUS,
                220.0,
            );
            widgets::form_date(ui, "Scheduled date", form.status != "Completed", &mut form.scheduled_date, 220.0);
            ui.end_row();
            if form.status == "Completed" {
                widgets::form_date(ui, "Completion date", true, &mut form.completed_date, 220.0);
            } else {
                ui.label("");
                ui.label("");
            }
            widgets::form_text(ui, "Cost", false, &mut form.cost, 220.0);
            ui.end_row();
            widgets::form_text(ui, "Mileage", false, &mut form.mileage, 220.0);
            ui.end_row();
        });
    ui.add_space(10.0);
    section_title(ui, "Next maintenance");
    ui.add_space(4.0);
    egui::Grid::new("maint_form_b")
        .num_columns(4)
        .spacing([14.0, 10.0])
        .show(ui, |ui| {
            widgets::form_text(ui, "Next due mileage", false, &mut form.next_due_km, 220.0);
            widgets::form_date(ui, "Next due date", false, &mut form.next_due_date, 220.0);
            ui.end_row();
        });
}
