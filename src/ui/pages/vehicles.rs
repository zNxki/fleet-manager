use super::Ctx;
use crate::db::models::*;
use crate::db::stats;
use crate::i18n::tr;
use crate::ui::dialogs::{self, Action};
use crate::ui::images::{self, ImageCache};
use crate::ui::theme::pal;
use crate::ui::widgets::{
    self, apply_sort, cell, cell_strong, contains_ci, empty, error_text, filter_combo, kv, metric,
    muted, page_header, primary, row_actions, search, status_badge, table, Col, Sort,
};
use crate::util::{files, format as f, input};
use chrono::Datelike;
use eframe::egui::{self, vec2, Align, Layout, Ui};
use std::cmp::Ordering;

#[derive(Default)]
pub struct State {
    search: String,
    status: String,
    fuel: String,
    sort: Sort,
    form: Option<Form>,
    view: Option<i64>,
}

#[derive(Clone, Default)]
pub struct Form {
    id: i64,
    plate: String,
    make: String,
    model: String,
    vin: String,
    year: String,
    fuel_type: String,
    mileage: String,
    acq_date: String,
    acq_price: String,
    acq_kind: String,
    base_status: String,
    image_path: String,
    image_new: String,
    notes: String,
    error: String,
}

impl Form {
    fn blank() -> Form {
        Form {
            year: f::today().year().to_string(),
            fuel_type: FUEL_TYPES[0].to_string(),
            mileage: "0".to_string(),
            acq_price: "0".to_string(),
            acq_kind: ACQ_KINDS[0].to_string(),
            base_status: BASE_STATUS[0].to_string(),
            ..Default::default()
        }
    }

    fn from(v: &Vehicle) -> Form {
        Form {
            id: v.id,
            plate: v.plate.clone(),
            make: v.make.clone(),
            model: v.model.clone(),
            vin: v.vin.clone(),
            year: v.year.to_string(),
            fuel_type: v.fuel_type.clone(),
            mileage: v.mileage.to_string(),
            acq_date: f::date(&v.acq_date),
            acq_price: f::num(v.acq_price, 2).replace(',', ""),
            acq_kind: v.acq_kind.clone(),
            base_status: v.base_status.clone(),
            image_path: v.image_path.clone(),
            image_new: String::new(),
            notes: v.notes.clone(),
            error: String::new(),
        }
    }

    fn build(&self) -> Result<Vehicle, String> {
        let plate = input::required(&self.plate, &tr("Registration plate"))?.to_uppercase();
        let make = input::required(&self.make, &tr("Manufacturer"))?;
        let model = input::required(&self.model, &tr("Model"))?;
        let vin = input::required(&self.vin, "VIN")?.to_uppercase();
        if vin.len() != 17
            || !vin
                .chars()
                .all(|c| c.is_ascii_alphanumeric() && !"IOQ".contains(c))
        {
            return Err(
                "VIN must contain 17 letters and digits (I, O and Q are not allowed)".to_string(),
            );
        }
        let year = f::parse_int(&self.year, &tr("Year"))?;
        if year < 1950 || year > f::today().year() as i64 + 1 {
            return Err("Invalid year".to_string());
        }
        let mileage = f::parse_int(&self.mileage, &tr("Mileage"))?;
        let acq_date = f::parse_date(&self.acq_date, &tr("Acquisition date"))?;
        if !acq_date.is_empty() && acq_date > f::today_iso() {
            return Err("The acquisition date cannot be in the future".to_string());
        }
        let acq_price = input::opt_num(&self.acq_price, &tr("Acquisition price"))?;
        Ok(Vehicle {
            id: self.id,
            plate,
            make,
            model,
            vin,
            year,
            fuel_type: self.fuel_type.clone(),
            mileage,
            acq_date,
            acq_price,
            acq_kind: self.acq_kind.clone(),
            base_status: self.base_status.clone(),
            image_path: self.image_path.clone(),
            notes: self.notes.trim().to_string(),
            status: String::new(),
        })
    }
}

pub fn show(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    page_header(
        ui,
        "Vehicles",
        "Register and manage the vehicles of your fleet",
        |ui| {
            if primary(ui, "+ Add vehicle").clicked() {
                st.form = Some(Form::blank());
            }
        },
    );

    ui.horizontal(|ui| {
        search(ui, &mut st.search);
        filter_combo(
            ui,
            "vf_status",
            &mut st.status,
            &VEHICLE_STATUS,
            "All statuses",
        );
        filter_combo(ui, "vf_fuel", &mut st.fuel, &FUEL_TYPES, "All fuel types");
    });
    ui.add_space(10.0);

    let mut idx: Vec<usize> = cache
        .vehicles
        .iter()
        .enumerate()
        .filter(|(_, v)| {
            (st.status.is_empty() || v.status == st.status)
                && (st.fuel.is_empty() || v.fuel_type == st.fuel)
                && (contains_ci(&v.plate, &st.search)
                    || contains_ci(&v.make, &st.search)
                    || contains_ci(&v.model, &st.search)
                    || contains_ci(&v.vin, &st.search))
        })
        .map(|(i, _)| i)
        .collect();
    apply_sort(&mut idx, st.sort, |col, a, b| {
        let (x, y) = (&cache.vehicles[a], &cache.vehicles[b]);
        match col {
            0 => x.plate.to_lowercase().cmp(&y.plate.to_lowercase()),
            1 => (x.make.to_lowercase(), x.model.to_lowercase())
                .cmp(&(y.make.to_lowercase(), y.model.to_lowercase())),
            2 => x.year.cmp(&y.year),
            3 => x.fuel_type.cmp(&y.fuel_type),
            4 => x.mileage.cmp(&y.mileage),
            5 => x.status.cmp(&y.status),
            _ => Ordering::Equal,
        }
    });

    let mut act: Option<(u8, i64)> = None;
    if cache.vehicles.is_empty() {
        empty(
            ui,
            "No vehicles yet",
            "Add your first vehicle to start managing your fleet",
        );
    } else if idx.is_empty() {
        empty(ui, "No results", "Try changing the search or the filters");
    } else {
        let cols = [
            Col::new("Plate", 110.0).sortable(),
            Col::new("Vehicle", 0.0).sortable(),
            Col::new("Year", 70.0).sortable(),
            Col::new("Fuel", 100.0).sortable(),
            Col::new("Mileage", 120.0).sortable(),
            Col::new("Status", 130.0).sortable(),
            Col::new("Current driver", 170.0),
            Col::new("Actions", 210.0),
        ];
        let n = idx.len();
        table(ui, "vehicles_table", &cols, n, &mut st.sort, |ui, r, c| {
            let v = &cache.vehicles[idx[r]];
            match c {
                0 => cell_strong(ui, &v.plate),
                1 => cell(ui, &format!("{} {}", v.make, v.model)),
                2 => cell(ui, &v.year.to_string()),
                3 => cell(ui, &tr(&v.fuel_type)),
                4 => cell(ui, &f::km(v.mileage)),
                5 => status_badge(ui, &v.status),
                6 => {
                    let d = cache
                        .open_assignment_of_vehicle(v.id)
                        .map(|a| cache.driver_name(a.driver_id))
                        .unwrap_or_else(|| "-".to_string());
                    cell(ui, &d);
                }
                _ => {
                    let (a, b, d) = row_actions(ui, true, true, true);
                    if a {
                        act = Some((1, v.id));
                    }
                    if b {
                        act = Some((2, v.id));
                    }
                    if d {
                        act = Some((3, v.id));
                    }
                }
            }
        });
    }

    if let Some((kind, id)) = act {
        if let Some(v) = cache.vehicle(id) {
            match kind {
                1 => st.view = Some(id),
                2 => st.form = Some(Form::from(v)),
                _ => {
                    let msg = format!(
                        "{} {} ({} {})? {}",
                        tr("Delete vehicle"),
                        v.plate,
                        v.make,
                        v.model,
                        tr("All related records will be deleted too.")
                    );
                    cx.ask(
                        "Delete vehicle",
                        &msg,
                        "Delete",
                        Action::DeleteVehicle(id),
                        format!("Vehicle deleted: {}", v.plate),
                    );
                }
            }
        }
    }

    let mut close = false;
    if let Some(form) = st.form.as_mut() {
        let title = if form.id == 0 {
            "Add vehicle"
        } else {
            "Edit vehicle"
        };
        let mut cancel = false;
        let mut save = false;
        let closed = dialogs::modal(ui.ctx(), "vehicle_form", title, 760.0, |ui| {
            form_ui(ui, cx.images, form);
            dialogs::footer(ui, "Save", &mut cancel, &mut save);
        });
        if closed || cancel {
            close = true;
        }
        if save {
            match form.build() {
                Ok(mut v) => {
                    let mut ok = true;
                    if !form.image_new.is_empty() {
                        match files::attach(&form.image_new) {
                            Ok(p) => v.image_path = p,
                            Err(e) => {
                                form.error = e;
                                ok = false;
                            }
                        }
                    }
                    if ok {
                        match cx.db.save_vehicle(&v) {
                            Ok(_) => {
                                let log = if v.id == 0 {
                                    format!("Vehicle added: {}", v.plate)
                                } else {
                                    format!("Vehicle updated: {}", v.plate)
                                };
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

    if let Some(id) = st.view {
        match cache.vehicle(id) {
            Some(v) => {
                let closed =
                    dialogs::info(ui.ctx(), "vehicle_view", "Vehicle details", 820.0, |ui| {
                        view_ui(ui, cx, v)
                    });
                if closed {
                    st.view = None;
                }
            }
            None => st.view = None,
        }
    }
}

fn form_ui(ui: &mut Ui, images: &mut ImageCache, form: &mut Form) {
    error_text(ui, &form.error);
    let path = if form.image_new.is_empty() {
        form.image_path.clone()
    } else {
        form.image_new.clone()
    };
    ui.horizontal(|ui| {
        match images.get(ui.ctx(), &path) {
            Some(tex) => images::show_cover(ui, &tex, vec2(150.0, 100.0)),
            None => images::placeholder(ui, vec2(150.0, 100.0)),
        }
        ui.vertical(|ui| {
            muted(ui, &tr("Vehicle image (PNG or JPEG)"));
            ui.horizontal(|ui| {
                if widgets::secondary(ui, "Choose image").clicked() {
                    if let Some(p) =
                        input::pick_file("Select image", "Images", &["png", "jpg", "jpeg"])
                    {
                        form.image_new = p;
                    }
                }
                if widgets::secondary(ui, "Remove").clicked() {
                    form.image_new.clear();
                    form.image_path.clear();
                }
            });
        });
    });
    ui.add_space(10.0);
    egui::Grid::new("veh_form")
        .num_columns(4)
        .spacing([14.0, 10.0])
        .show(ui, |ui| {
            widgets::form_text(ui, "Registration plate", true, &mut form.plate, 210.0);
            widgets::form_text(ui, "VIN", true, &mut form.vin, 210.0);
            ui.end_row();
            widgets::form_text(ui, "Manufacturer", true, &mut form.make, 210.0);
            widgets::form_text(ui, "Model", true, &mut form.model, 210.0);
            ui.end_row();
            widgets::form_text(ui, "Year", true, &mut form.year, 210.0);
            widgets::form_combo(
                ui,
                "Fuel type",
                true,
                "vf_fuel_type",
                &mut form.fuel_type,
                &FUEL_TYPES,
                210.0,
            );
            ui.end_row();
            widgets::form_text(ui, "Mileage", true, &mut form.mileage, 210.0);
            widgets::form_combo(
                ui,
                "Status",
                true,
                "vf_base_status",
                &mut form.base_status,
                &BASE_STATUS,
                210.0,
            );
            ui.end_row();
            widgets::form_combo(
                ui,
                "Acquisition type",
                false,
                "vf_acq_kind",
                &mut form.acq_kind,
                &ACQ_KINDS,
                210.0,
            );
            widgets::form_date(ui, "Acquisition date", false, &mut form.acq_date, 210.0);
            ui.end_row();
            widgets::form_text(ui, "Acquisition price", false, &mut form.acq_price, 210.0);
            ui.end_row();
        });
    ui.add_space(8.0);
    egui::Grid::new("veh_form_notes")
        .num_columns(2)
        .spacing([14.0, 10.0])
        .show(ui, |ui| {
            widgets::form_multiline(ui, "Notes", &mut form.notes, 590.0);
            ui.end_row();
        });
}

fn view_ui(ui: &mut Ui, cx: &mut Ctx, v: &Vehicle) {
    let cache = cx.cache;
    let p = pal();
    ui.horizontal_top(|ui| {
        match cx.images.get(ui.ctx(), &v.image_path) {
            Some(tex) => images::show_cover(ui, &tex, vec2(230.0, 165.0)),
            None => images::placeholder(ui, vec2(230.0, 165.0)),
        }
        ui.add_space(10.0);
        egui::Grid::new("veh_view")
            .num_columns(2)
            .spacing([16.0, 6.0])
            .show(ui, |ui| {
                kv(ui, "Registration plate", &v.plate);
                kv(ui, "Vehicle", &format!("{} {}", v.make, v.model));
                kv(ui, "VIN", &v.vin);
                kv(ui, "Year", &v.year.to_string());
                kv(ui, "Fuel type", &tr(&v.fuel_type));
                kv(ui, "Mileage", &f::km(v.mileage));
                ui.label(egui::RichText::new(tr("Status")).color(p.muted).size(13.0));
                status_badge(ui, &v.status);
                ui.end_row();
                let drv = cache
                    .open_assignment_of_vehicle(v.id)
                    .map(|a| cache.driver_name(a.driver_id))
                    .unwrap_or_default();
                kv(ui, "Current driver", &drv);
                let acq = format!(
                    "{} - {} - {}",
                    tr(&v.acq_kind),
                    if v.acq_date.is_empty() {
                        "-".to_string()
                    } else {
                        f::date(&v.acq_date)
                    },
                    f::money(v.acq_price)
                );
                kv(ui, "Acquisition", &acq);
                kv(ui, "Notes", &v.notes);
            });
    });
    ui.add_space(12.0);
    let cs = stats::consumption(cache, v.id, "0000-01-01", "9999-12-31");
    let fuel_cost: f64 = cache
        .fuel
        .iter()
        .filter(|x| x.vehicle_id == v.id)
        .map(|x| x.liters * x.price)
        .sum();
    let maint_cost: f64 = cache
        .maintenance
        .iter()
        .filter(|m| m.vehicle_id == v.id && m.status == "Completed")
        .map(|m| m.cost)
        .sum();
    let other: f64 = cache
        .expenses
        .iter()
        .filter(|e| e.vehicle_id == v.id)
        .map(|e| e.amount)
        .sum();
    ui.horizontal(|ui| {
        metric(ui, "Fuel cost", &f::money(fuel_cost), 140.0);
        metric(ui, "Maintenance cost", &f::money(maint_cost), 150.0);
        metric(ui, "Other expenses", &f::money(other), 140.0);
        metric(
            ui,
            "Average consumption",
            &cs.as_ref()
                .map(|c| format!("{} L/100 km", f::num(c.l100, 2)))
                .unwrap_or_else(|| "-".to_string()),
            160.0,
        );
        metric(
            ui,
            "Cost per km",
            &cs.as_ref()
                .map(|c| f::money(c.all_km))
                .unwrap_or_else(|| "-".to_string()),
            110.0,
        );
    });
    ui.add_space(8.0);
    widgets::section_title(ui, "Vehicle history");
    ui.add_space(4.0);
    let mut ev: Vec<(String, &str, String, f64)> = Vec::new();
    for a in cache.assignments.iter().filter(|a| a.vehicle_id == v.id) {
        ev.push((
            a.start_date.clone(),
            "Assignment",
            format!(
                "{} {} ({})",
                tr("Assigned to"),
                cache.driver_name(a.driver_id),
                f::km(a.start_km)
            ),
            0.0,
        ));
        if !a.end_date.is_empty() {
            ev.push((
                a.end_date.clone(),
                "Assignment",
                format!(
                    "{} {} ({})",
                    tr("Returned by"),
                    cache.driver_name(a.driver_id),
                    f::km(a.end_km.unwrap_or(a.start_km))
                ),
                0.0,
            ));
        }
    }
    for m in cache.maintenance.iter().filter(|m| m.vehicle_id == v.id) {
        let d = if m.completed_date.is_empty() {
            &m.scheduled_date
        } else {
            &m.completed_date
        };
        ev.push((
            d.clone(),
            "Maintenance",
            format!("{} ({})", m.description, tr(&m.status)),
            if m.status == "Completed" { m.cost } else { 0.0 },
        ));
    }
    for x in cache.fuel.iter().filter(|x| x.vehicle_id == v.id) {
        ev.push((
            x.date.clone(),
            "Fuel",
            format!(
                "{} {} L {}",
                tr("Refueling"),
                f::num(x.liters, 1),
                x.station
            ),
            x.liters * x.price,
        ));
    }
    for x in cache.expenses.iter().filter(|x| x.vehicle_id == v.id) {
        let extra = if x.description.is_empty() {
            String::new()
        } else {
            format!(" - {}", x.description)
        };
        ev.push((
            x.date.clone(),
            "Expense",
            format!("{}{}", tr(&x.category), extra),
            x.amount,
        ));
    }
    for d in cache
        .documents
        .iter()
        .filter(|d| d.vehicle_id == v.id && !d.issue_date.is_empty())
    {
        ev.push((
            d.issue_date.clone(),
            "Document",
            format!("{} {}", tr(&d.doc_type), d.number),
            0.0,
        ));
    }
    ev.sort_by(|a, b| b.0.cmp(&a.0));
    if ev.is_empty() {
        muted(ui, &tr("No history recorded for this vehicle"));
    } else {
        egui::Grid::new("veh_history")
            .num_columns(4)
            .striped(true)
            .spacing([16.0, 7.0])
            .show(ui, |ui| {
                for (d, k, text, amount) in ev.iter().take(60) {
                    ui.label(egui::RichText::new(f::date(d)).color(p.muted));
                    let c = match *k {
                        "Assignment" => p.purple,
                        "Maintenance" => p.warn,
                        "Fuel" => p.accent,
                        "Expense" => p.danger,
                        _ => p.success,
                    };
                    widgets::badge(ui, k, c);
                    ui.label(widgets::cut(text, 58));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if *amount > 0.0 {
                            ui.label(f::money(*amount));
                        }
                    });
                    ui.end_row();
                }
            });
    }
}
