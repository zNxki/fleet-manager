use super::Ctx;
use crate::db::models::*;
use crate::i18n::tr;
use crate::ui::dialogs::{self, Action};
use crate::ui::widgets::{
    self, apply_sort, cell, cell_strong, contains_ci, empty, error_text, filter_combo,
    filter_combo_id, muted, page_header, primary, search, small, small_danger, status_badge, table,
    Col, Sort,
};
use crate::util::format as f;
use eframe::egui::{self, Ui};
use std::cmp::Ordering;

const STATUS_FILTER: [&str; 2] = ["Open", "Completed"];

#[derive(Default)]
pub struct State {
    search: String,
    status: String,
    vehicle: i64,
    driver: i64,
    sort: Sort,
    form: Option<Form>,
    ret: Option<ReturnForm>,
}

#[derive(Clone, Default)]
struct Form {
    vehicle_id: i64,
    driver_id: i64,
    start_date: String,
    start_km: String,
    end_date: String,
    end_km: String,
    notes: String,
    error: String,
}

#[derive(Clone, Default)]
struct ReturnForm {
    id: i64,
    vehicle_id: i64,
    driver_id: i64,
    start_date: String,
    start_km: i64,
    end_date: String,
    end_km: String,
    error: String,
}

impl Form {
    fn blank() -> Form {
        Form {
            start_date: f::today_input(),
            ..Default::default()
        }
    }

    fn build(&self) -> Result<Assignment, String> {
        if self.vehicle_id == 0 {
            return Err("Select a vehicle".to_string());
        }
        if self.driver_id == 0 {
            return Err("Select a driver".to_string());
        }
        let start_date = f::require_date(&self.start_date, &tr("Start date"))?;
        let start_km = f::parse_int(&self.start_km, &tr("Start mileage"))?;
        let end_date = f::parse_date(&self.end_date, &tr("Return date"))?;
        let end_km = if end_date.is_empty() {
            if !self.end_km.trim().is_empty() {
                return Err("Enter the return date together with the return mileage".to_string());
            }
            None
        } else {
            Some(f::parse_int(&self.end_km, &tr("Return mileage"))?)
        };
        Ok(Assignment {
            id: 0,
            vehicle_id: self.vehicle_id,
            driver_id: self.driver_id,
            start_date,
            end_date,
            start_km,
            end_km,
            notes: self.notes.trim().to_string(),
        })
    }
}

pub fn show(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    page_header(
        ui,
        "Assignments",
        "Assign vehicles to drivers and track returns",
        |ui| {
            if primary(ui, "+ New assignment").clicked() {
                st.form = Some(Form::blank());
            }
        },
    );

    let vehicles = cache.vehicle_items(true);
    let drivers = cache.driver_items(true);
    ui.horizontal(|ui| {
        search(ui, &mut st.search);
        filter_combo(
            ui,
            "af_status",
            &mut st.status,
            &STATUS_FILTER,
            "All statuses",
        );
        filter_combo_id(ui, "af_vehicle", &mut st.vehicle, &vehicles, "All vehicles");
        filter_combo_id(ui, "af_driver", &mut st.driver, &drivers, "All drivers");
    });
    ui.add_space(10.0);

    let mut idx: Vec<usize> = cache
        .assignments
        .iter()
        .enumerate()
        .filter(|(_, a)| {
            let status = if a.end_date.is_empty() {
                "Open"
            } else {
                "Completed"
            };
            (st.status.is_empty() || st.status == status)
                && (st.vehicle == 0 || a.vehicle_id == st.vehicle)
                && (st.driver == 0 || a.driver_id == st.driver)
                && (contains_ci(&cache.vehicle_name(a.vehicle_id), &st.search)
                    || contains_ci(&cache.driver_name(a.driver_id), &st.search)
                    || contains_ci(&a.notes, &st.search))
        })
        .map(|(i, _)| i)
        .collect();
    apply_sort(&mut idx, st.sort, |col, a, b| {
        let (x, y) = (&cache.assignments[a], &cache.assignments[b]);
        match col {
            0 => cache
                .vehicle_name(x.vehicle_id)
                .cmp(&cache.vehicle_name(y.vehicle_id)),
            1 => cache
                .driver_name(x.driver_id)
                .cmp(&cache.driver_name(y.driver_id)),
            2 => x.start_date.cmp(&y.start_date),
            3 => x.end_date.cmp(&y.end_date),
            4 => x.start_km.cmp(&y.start_km),
            5 => {
                let dx = x.end_km.map(|k| k - x.start_km).unwrap_or(-1);
                let dy = y.end_km.map(|k| k - y.start_km).unwrap_or(-1);
                dx.cmp(&dy)
            }
            6 => x.end_date.is_empty().cmp(&y.end_date.is_empty()),
            _ => Ordering::Equal,
        }
    });

    let mut act: Option<(u8, i64)> = None;
    if cache.assignments.is_empty() {
        empty(
            ui,
            "No assignments yet",
            "Create an assignment to give a vehicle to a driver",
        );
    } else if idx.is_empty() {
        empty(ui, "No results", "Try changing the search or the filters");
    } else {
        let cols = [
            Col::new("Vehicle", 120.0).sortable(),
            Col::new("Driver", 0.0).sortable(),
            Col::new("From", 110.0).sortable(),
            Col::new("To", 110.0).sortable(),
            Col::new("Start mileage", 120.0).sortable(),
            Col::new("Distance", 110.0).sortable(),
            Col::new("Status", 110.0).sortable(),
            Col::new("Actions", 170.0),
        ];
        let n = idx.len();
        table(
            ui,
            "assignments_table",
            &cols,
            n,
            &mut st.sort,
            |ui, r, c| {
                let a = &cache.assignments[idx[r]];
                match c {
                    0 => cell_strong(ui, &cache.vehicle_name(a.vehicle_id)),
                    1 => cell(ui, &cache.driver_name(a.driver_id)),
                    2 => cell(ui, &f::date(&a.start_date)),
                    3 => cell(
                        ui,
                        &if a.end_date.is_empty() {
                            "-".to_string()
                        } else {
                            f::date(&a.end_date)
                        },
                    ),
                    4 => cell(ui, &f::km(a.start_km)),
                    5 => cell(
                        ui,
                        &match a.end_km {
                            Some(k) => f::km(k - a.start_km),
                            None => "-".to_string(),
                        },
                    ),
                    6 => status_badge(
                        ui,
                        if a.end_date.is_empty() {
                            "Open"
                        } else {
                            "Completed"
                        },
                    ),
                    _ => {
                        if a.end_date.is_empty() && small(ui, "Return").clicked() {
                            act = Some((1, a.id));
                        }
                        if small_danger(ui, "Delete").clicked() {
                            act = Some((2, a.id));
                        }
                    }
                }
            },
        );
    }

    if let Some((kind, id)) = act {
        if let Some(a) = cache.assignments.iter().find(|a| a.id == id) {
            if kind == 1 {
                let km = cache
                    .vehicle(a.vehicle_id)
                    .map(|v| v.mileage)
                    .unwrap_or(a.start_km)
                    .max(a.start_km);
                st.ret = Some(ReturnForm {
                    id: a.id,
                    vehicle_id: a.vehicle_id,
                    driver_id: a.driver_id,
                    start_date: a.start_date.clone(),
                    start_km: a.start_km,
                    end_date: f::today_input(),
                    end_km: km.to_string(),
                    error: String::new(),
                });
            } else {
                let msg = format!(
                    "{} {} / {}?",
                    tr("Delete assignment"),
                    cache.vehicle_name(a.vehicle_id),
                    cache.driver_name(a.driver_id)
                );
                cx.ask(
                    "Delete assignment",
                    &msg,
                    "Delete",
                    Action::DeleteAssignment(id),
                    format!(
                        "Assignment deleted: {} / {}",
                        cache.vehicle_name(a.vehicle_id),
                        cache.driver_name(a.driver_id)
                    ),
                );
            }
        }
    }

    let mut close = false;
    if let Some(form) = st.form.as_mut() {
        let free_vehicles: Vec<(i64, String)> = cache
            .vehicles
            .iter()
            .filter(|v| v.status == "Available")
            .map(|v| (v.id, format!("{} - {} {}", v.plate, v.make, v.model)))
            .collect();
        let free_drivers: Vec<(i64, String)> = cache
            .drivers
            .iter()
            .filter(|d| d.active && cache.open_assignment_of_driver(d.id).is_none())
            .map(|d| (d.id, format!("{} {}", d.last_name, d.first_name)))
            .collect();
        let mut cancel = false;
        let mut save = false;
        let closed = dialogs::modal(ui.ctx(), "assign_form", "New assignment", 640.0, |ui| {
            form_ui(ui, cx, form, &free_vehicles, &free_drivers);
            dialogs::footer(ui, "Save", &mut cancel, &mut save);
        });
        if closed || cancel {
            close = true;
        }
        if save {
            match form.build() {
                Ok(a) => match cx.db.create_assignment(&a) {
                    Ok(_) => {
                        let log = format!(
                            "Assignment created: {} / {}",
                            cache.vehicle_name(a.vehicle_id),
                            cache.driver_name(a.driver_id)
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

    let mut close_ret = false;
    if let Some(rf) = st.ret.as_mut() {
        let mut cancel = false;
        let mut save = false;
        let closed = dialogs::modal(ui.ctx(), "assign_return", "Register return", 520.0, |ui| {
            error_text(ui, &rf.error);
            egui::Grid::new("ret_info")
                .num_columns(2)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    widgets::kv(ui, "Vehicle", &cache.vehicle_name(rf.vehicle_id));
                    widgets::kv(ui, "Driver", &cache.driver_name(rf.driver_id));
                    widgets::kv(ui, "Start date", &f::date(&rf.start_date));
                    widgets::kv(ui, "Start mileage", &f::km(rf.start_km));
                });
            ui.add_space(10.0);
            egui::Grid::new("ret_form")
                .num_columns(2)
                .spacing([14.0, 10.0])
                .show(ui, |ui| {
                    widgets::form_date(ui, "Return date", true, &mut rf.end_date, 220.0);
                    widgets::form_text(ui, "Return mileage", true, &mut rf.end_km, 220.0);
                    ui.end_row();
                });
            dialogs::footer(ui, "Register return", &mut cancel, &mut save);
        });
        if closed || cancel {
            close_ret = true;
        }
        if save {
            let res = f::require_date(&rf.end_date, &tr("Return date"))
                .and_then(|d| f::parse_int(&rf.end_km, &tr("Return mileage")).map(|k| (d, k)))
                .and_then(|(d, k)| cx.db.close_assignment(rf.id, &d, k));
            match res {
                Ok(()) => {
                    let log = format!(
                        "Vehicle returned: {} / {}",
                        cache.vehicle_name(rf.vehicle_id),
                        cache.driver_name(rf.driver_id)
                    );
                    cx.saved(&log);
                    close_ret = true;
                }
                Err(e) => rf.error = e,
            }
        }
    }
    if close_ret {
        st.ret = None;
    }
}

fn form_ui(
    ui: &mut Ui,
    cx: &Ctx,
    form: &mut Form,
    vehicles: &[(i64, String)],
    drivers: &[(i64, String)],
) {
    error_text(ui, &form.error);
    if vehicles.is_empty() {
        muted(ui, &tr("No vehicle is currently available for assignment"));
        ui.add_space(6.0);
    }
    if drivers.is_empty() {
        muted(ui, &tr("No active driver without an open assignment"));
        ui.add_space(6.0);
    }
    egui::Grid::new("assign_form_grid")
        .num_columns(2)
        .spacing([14.0, 10.0])
        .show(ui, |ui| {
            widgets::form_label(ui, "Vehicle", true);
            let changed = widgets::combo_id(ui, "af_new_vehicle", &mut form.vehicle_id, vehicles, None, 340.0);
            if changed {
                if let Some(v) = cx.cache.vehicle(form.vehicle_id) {
                    form.start_km = v.mileage.to_string();
                }
            }
            ui.end_row();
            widgets::form_label(ui, "Driver", true);
            widgets::combo_id(ui, "af_new_driver", &mut form.driver_id, drivers, None, 340.0);
            ui.end_row();
            widgets::form_date(ui, "Start date", true, &mut form.start_date, 340.0);
            ui.end_row();
            widgets::form_text(ui, "Start mileage", true, &mut form.start_km, 340.0);
            ui.end_row();
        });
    ui.add_space(8.0);
    widgets::section_title(ui, "Return (only for past assignments)");
    ui.add_space(4.0);
    egui::Grid::new("assign_form_ret")
        .num_columns(2)
        .spacing([14.0, 10.0])
        .show(ui, |ui| {
            widgets::form_date(ui, "Return date", false, &mut form.end_date, 340.0);
            ui.end_row();
            widgets::form_text(ui, "Return mileage", false, &mut form.end_km, 340.0);
            ui.end_row();
            widgets::form_multiline(ui, "Notes", &mut form.notes, 340.0);
            ui.end_row();
        });
}
