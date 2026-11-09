use super::Ctx;
use crate::db::models::*;
use crate::db::stats;
use crate::i18n::tr;
use crate::ui::charts;
use crate::ui::dialogs::{self, Action};
use crate::ui::theme::pal;
use crate::ui::widgets::{
    self, apply_sort, card, cell, cell_strong, contains_ci, empty, error_text, filter_combo,
    filter_combo_id, metric, muted, page_header, primary, row_actions, search, section_title,
    table, tabs, Col, Sort,
};
use crate::util::format as f;
use eframe::egui::{self, Align, Layout, RichText, Ui};
use std::cmp::Ordering;

#[derive(Default)]
pub struct State {
    tab: usize,
    search: String,
    vehicle: i64,
    category: String,
    fuel_sort: Sort,
    exp_sort: Sort,
    fuel_form: Option<FuelForm>,
    exp_form: Option<ExpForm>,
}

#[derive(Clone, Default)]
struct FuelForm {
    id: i64,
    vehicle_id: i64,
    driver_id: i64,
    date: String,
    liters: String,
    price: String,
    mileage: String,
    station: String,
    full_tank: bool,
    error: String,
}

#[derive(Clone, Default)]
struct ExpForm {
    id: i64,
    vehicle_id: i64,
    category: String,
    date: String,
    amount: String,
    description: String,
    error: String,
}

impl FuelForm {
    fn blank() -> FuelForm {
        FuelForm {
            date: f::today_input(),
            full_tank: true,
            ..Default::default()
        }
    }

    fn from(x: &Fuel) -> FuelForm {
        FuelForm {
            id: x.id,
            vehicle_id: x.vehicle_id,
            driver_id: x.driver_id,
            date: f::date(&x.date),
            liters: f::num(x.liters, 2).replace(',', ""),
            price: f::num(x.price, 3).replace(',', ""),
            mileage: x.mileage.to_string(),
            station: x.station.clone(),
            full_tank: x.full_tank,
            error: String::new(),
        }
    }

    fn build(&self) -> Result<Fuel, String> {
        if self.vehicle_id == 0 {
            return Err("Select a vehicle".to_string());
        }
        let liters = f::parse_num(&self.liters, &tr("Liters"))?;
        let price = f::parse_num(&self.price, &tr("Price per liter"))?;
        if liters <= 0.0 || price <= 0.0 {
            return Err("Liters and price must be greater than zero".to_string());
        }
        Ok(Fuel {
            id: self.id,
            vehicle_id: self.vehicle_id,
            driver_id: self.driver_id,
            date: f::require_date(&self.date, &tr("Date"))?,
            liters,
            price,
            mileage: f::parse_int(&self.mileage, &tr("Mileage"))?,
            station: self.station.trim().to_string(),
            full_tank: self.full_tank,
        })
    }
}

impl ExpForm {
    fn blank() -> ExpForm {
        ExpForm {
            category: EXPENSE_CATS[0].to_string(),
            date: f::today_input(),
            ..Default::default()
        }
    }

    fn from(x: &Expense) -> ExpForm {
        ExpForm {
            id: x.id,
            vehicle_id: x.vehicle_id,
            category: x.category.clone(),
            date: f::date(&x.date),
            amount: f::num(x.amount, 2).replace(',', ""),
            description: x.description.clone(),
            error: String::new(),
        }
    }

    fn build(&self) -> Result<Expense, String> {
        if self.vehicle_id == 0 {
            return Err("Select a vehicle".to_string());
        }
        let date = f::require_date(&self.date, &tr("Date"))?;
        if date > f::today_iso() {
            return Err("The expense date cannot be in the future".to_string());
        }
        let amount = f::parse_num(&self.amount, &tr("Amount"))?;
        if amount <= 0.0 {
            return Err("The amount must be greater than zero".to_string());
        }
        Ok(Expense {
            id: self.id,
            vehicle_id: self.vehicle_id,
            category: self.category.clone(),
            date,
            amount,
            description: self.description.trim().to_string(),
        })
    }
}

pub fn show(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    page_header(
        ui,
        "Fuel & Expenses",
        "Record refuelings and expenses and analyze consumption",
        |ui| match st.tab {
            1 => {
                if primary(ui, "+ Add expense").clicked() {
                    st.exp_form = Some(ExpForm::blank());
                }
            }
            0 => {
                if primary(ui, "+ Add refueling").clicked() {
                    st.fuel_form = Some(FuelForm::blank());
                }
            }
            _ => {}
        },
    );
    tabs(ui, &mut st.tab, &["Refuelings", "Expenses", "Statistics"]);
    match st.tab {
        0 => fuel_tab(ui, cx, st),
        1 => expense_tab(ui, cx, st),
        _ => stats_tab(ui, cx),
    }
    fuel_dialog(ui, cx, st);
    expense_dialog(ui, cx, st);
}

fn fuel_tab(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    let vehicles = cache.vehicle_items(true);
    ui.horizontal(|ui| {
        search(ui, &mut st.search);
        filter_combo_id(ui, "ff_vehicle", &mut st.vehicle, &vehicles, "All vehicles");
    });
    ui.add_space(10.0);
    let mut idx: Vec<usize> = cache
        .fuel
        .iter()
        .enumerate()
        .filter(|(_, x)| {
            (st.vehicle == 0 || x.vehicle_id == st.vehicle)
                && (contains_ci(&x.station, &st.search)
                    || contains_ci(&cache.vehicle_name(x.vehicle_id), &st.search)
                    || contains_ci(&cache.driver_name(x.driver_id), &st.search))
        })
        .map(|(i, _)| i)
        .collect();
    apply_sort(&mut idx, st.fuel_sort, |col, a, b| {
        let (x, y) = (&cache.fuel[a], &cache.fuel[b]);
        match col {
            0 => x.date.cmp(&y.date),
            1 => cache
                .vehicle_name(x.vehicle_id)
                .cmp(&cache.vehicle_name(y.vehicle_id)),
            2 => cache
                .driver_name(x.driver_id)
                .cmp(&cache.driver_name(y.driver_id)),
            3 => x.liters.partial_cmp(&y.liters).unwrap_or(Ordering::Equal),
            4 => x.price.partial_cmp(&y.price).unwrap_or(Ordering::Equal),
            5 => (x.liters * x.price)
                .partial_cmp(&(y.liters * y.price))
                .unwrap_or(Ordering::Equal),
            6 => x.mileage.cmp(&y.mileage),
            7 => x.station.to_lowercase().cmp(&y.station.to_lowercase()),
            _ => Ordering::Equal,
        }
    });
    let mut act: Option<(u8, i64)> = None;
    if cache.fuel.is_empty() {
        empty(
            ui,
            "No refuelings yet",
            "Record the first refueling to start tracking consumption",
        );
    } else if idx.is_empty() {
        empty(ui, "No results", "Try changing the search or the filters");
    } else {
        let cols = [
            Col::new("Date", 100.0).sortable(),
            Col::new("Vehicle", 110.0).sortable(),
            Col::new("Driver", 0.0).sortable(),
            Col::new("Liters", 80.0).sortable(),
            Col::new("Price/L", 90.0).sortable(),
            Col::new("Total", 110.0).sortable(),
            Col::new("Mileage", 110.0).sortable(),
            Col::new("Station", 150.0).sortable(),
            Col::new("Tank", 70.0),
            Col::new("Actions", 150.0),
        ];
        let n = idx.len();
        table(ui, "fuel_table", &cols, n, &mut st.fuel_sort, |ui, r, c| {
            let x = &cache.fuel[idx[r]];
            match c {
                0 => cell(ui, &f::date(&x.date)),
                1 => cell_strong(ui, &cache.vehicle_name(x.vehicle_id)),
                2 => cell(ui, &cache.driver_name(x.driver_id)),
                3 => cell(ui, &f::num(x.liters, 2)),
                4 => cell(ui, &f::num(x.price, 3)),
                5 => cell(ui, &f::money(x.liters * x.price)),
                6 => cell(ui, &f::km(x.mileage)),
                7 => cell(ui, &x.station),
                8 => cell(ui, &tr(if x.full_tank { "Full" } else { "Partial" })),
                _ => {
                    let (_, e, d) = row_actions(ui, false, true, true);
                    if e {
                        act = Some((1, x.id));
                    }
                    if d {
                        act = Some((2, x.id));
                    }
                }
            }
        });
    }
    if let Some((kind, id)) = act {
        if let Some(x) = cache.fuel.iter().find(|x| x.id == id) {
            if kind == 1 {
                st.fuel_form = Some(FuelForm::from(x));
            } else {
                let msg = format!(
                    "{} {} ({})?",
                    tr("Delete refueling"),
                    cache.vehicle_name(x.vehicle_id),
                    f::date(&x.date)
                );
                cx.ask(
                    "Delete refueling",
                    &msg,
                    "Delete",
                    Action::DeleteFuel(id),
                    format!(
                        "Refueling deleted: {} ({})",
                        cache.vehicle_name(x.vehicle_id),
                        f::date(&x.date)
                    ),
                );
            }
        }
    }
}

fn expense_tab(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    let vehicles = cache.vehicle_items(true);
    ui.horizontal(|ui| {
        search(ui, &mut st.search);
        filter_combo(ui, "ef_cat", &mut st.category, &EXPENSE_CATS, "All categories");
        filter_combo_id(ui, "ef_vehicle", &mut st.vehicle, &vehicles, "All vehicles");
    });
    ui.add_space(10.0);
    let mut idx: Vec<usize> = cache
        .expenses
        .iter()
        .enumerate()
        .filter(|(_, x)| {
            (st.vehicle == 0 || x.vehicle_id == st.vehicle)
                && (st.category.is_empty() || x.category == st.category)
                && (contains_ci(&x.description, &st.search)
                    || contains_ci(&cache.vehicle_name(x.vehicle_id), &st.search))
        })
        .map(|(i, _)| i)
        .collect();
    apply_sort(&mut idx, st.exp_sort, |col, a, b| {
        let (x, y) = (&cache.expenses[a], &cache.expenses[b]);
        match col {
            0 => x.date.cmp(&y.date),
            1 => cache
                .vehicle_name(x.vehicle_id)
                .cmp(&cache.vehicle_name(y.vehicle_id)),
            2 => x.category.cmp(&y.category),
            3 => x.description.to_lowercase().cmp(&y.description.to_lowercase()),
            4 => x.amount.partial_cmp(&y.amount).unwrap_or(Ordering::Equal),
            _ => Ordering::Equal,
        }
    });
    let mut act: Option<(u8, i64)> = None;
    if cache.expenses.is_empty() {
        empty(
            ui,
            "No expenses yet",
            "Record tolls, parking, taxes and other vehicle expenses",
        );
    } else if idx.is_empty() {
        empty(ui, "No results", "Try changing the search or the filters");
    } else {
        let cols = [
            Col::new("Date", 100.0).sortable(),
            Col::new("Vehicle", 120.0).sortable(),
            Col::new("Category", 130.0).sortable(),
            Col::new("Description", 0.0).sortable(),
            Col::new("Amount", 120.0).sortable(),
            Col::new("Actions", 150.0),
        ];
        let n = idx.len();
        table(ui, "expense_table", &cols, n, &mut st.exp_sort, |ui, r, c| {
            let x = &cache.expenses[idx[r]];
            match c {
                0 => cell(ui, &f::date(&x.date)),
                1 => cell_strong(ui, &cache.vehicle_name(x.vehicle_id)),
                2 => cell(ui, &tr(&x.category)),
                3 => cell(ui, &x.description),
                4 => cell(ui, &f::money(x.amount)),
                _ => {
                    let (_, e, d) = row_actions(ui, false, true, true);
                    if e {
                        act = Some((1, x.id));
                    }
                    if d {
                        act = Some((2, x.id));
                    }
                }
            }
        });
    }
    if let Some((kind, id)) = act {
        if let Some(x) = cache.expenses.iter().find(|x| x.id == id) {
            if kind == 1 {
                st.exp_form = Some(ExpForm::from(x));
            } else {
                let msg = format!(
                    "{} {} - {} ({})?",
                    tr("Delete expense"),
                    cache.vehicle_name(x.vehicle_id),
                    tr(&x.category),
                    f::money(x.amount)
                );
                cx.ask(
                    "Delete expense",
                    &msg,
                    "Delete",
                    Action::DeleteExpense(id),
                    format!(
                        "Expense deleted: {} - {}",
                        cache.vehicle_name(x.vehicle_id),
                        x.category
                    ),
                );
            }
        }
    }
}

fn stats_tab(ui: &mut Ui, cx: &mut Ctx) {
    let cache = cx.cache;
    let s = &cache.stats;
    let p = pal();
    let total_fuel: f64 = cache.fuel.iter().map(|x| x.liters * x.price).sum();
    let total_liters: f64 = cache.fuel.iter().map(|x| x.liters).sum();
    let total_exp: f64 = cache.expenses.iter().map(|x| x.amount).sum();
    let dash = |v: f64, s: String| if v > 0.0 { s } else { "-".to_string() };
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let w = ui.available_width() - 4.0;
            card(ui, |ui| {
                ui.set_width(w - 38.0);
                section_title(ui, "Overview");
                ui.add_space(6.0);
                let mw = ((w - 38.0) / 5.0).floor();
                ui.horizontal(|ui| {
                    metric(ui, "Fuel cost", &f::money(total_fuel), mw);
                    metric(
                        ui,
                        "Liters",
                        &format!("{} L", f::num(total_liters, 1)),
                        mw,
                    );
                    metric(ui, "Other expenses", &f::money(total_exp), mw);
                    metric(
                        ui,
                        "Fleet average",
                        &dash(s.fleet_l100, format!("{} L/100 km", f::num(s.fleet_l100, 2))),
                        mw,
                    );
                    metric(
                        ui,
                        "Cost per km",
                        &dash(s.fleet_cost_km, f::money(s.fleet_cost_km)),
                        mw,
                    );
                });
            });
            ui.add_space(14.0);
            widgets::pair(
                ui,
                0.6,
                |ui, w| {
                    card(ui, |ui| {
                        ui.set_width(w - 38.0);
                        ui.set_min_height(260.0);
                        section_title(ui, "Consumption by vehicle");
                        ui.add_space(6.0);
                        let mut any = false;
                        egui::Grid::new("fuel_stats_grid")
                            .num_columns(6)
                            .striped(true)
                            .spacing([22.0, 8.0])
                            .show(ui, |ui| {
                                for h in [
                                    "Vehicle",
                                    "Distance",
                                    "Liters",
                                    "L/100 km",
                                    "Fuel cost/km",
                                    "Total cost/km",
                                ] {
                                    ui.label(RichText::new(tr(h)).color(p.muted).size(12.0).strong());
                                }
                                ui.end_row();
                                for v in &cache.vehicles {
                                    if let Some(c) =
                                        stats::consumption(cache, v.id, "0000-01-01", "9999-12-31")
                                    {
                                        any = true;
                                        ui.label(RichText::new(&v.plate).strong());
                                        ui.label(f::km(c.km));
                                        ui.label(f::num(c.liters, 1));
                                        ui.label(f::num(c.l100, 2));
                                        ui.label(f::money(c.cost_km));
                                        ui.label(f::money(c.all_km));
                                        ui.end_row();
                                    }
                                }
                            });
                        if !any {
                            muted(
                                ui,
                                &tr("Record at least two full-tank refuelings to see consumption."),
                            );
                        }
                    });
                },
                |ui, w| {
                    card(ui, |ui| {
                        ui.set_width(w - 38.0);
                        ui.set_min_height(260.0);
                        section_title(ui, "Expenses by category");
                        ui.add_space(6.0);
                        let mut segs: Vec<(String, f64, eframe::egui::Color32)> = Vec::new();
                        for (i, c) in EXPENSE_CATS.iter().enumerate() {
                            let v: f64 = cache
                                .expenses
                                .iter()
                                .filter(|x| x.category == *c)
                                .map(|x| x.amount)
                                .sum();
                            if v > 0.0 {
                                segs.push((tr(c), v, charts::palette(i)));
                            }
                        }
                        if segs.is_empty() {
                            muted(ui, &tr("No expenses yet"));
                        } else {
                            let total: f64 = segs.iter().map(|x| x.1).sum();
                            ui.vertical_centered(|ui| {
                                charts::donut(
                                    ui,
                                    "fuel_cat_donut",
                                    140.0,
                                    &segs,
                                    &charts::compact(total),
                                    &tr("Total"),
                                );
                            });
                            ui.add_space(6.0);
                            for (n, v, c) in &segs {
                                ui.horizontal(|ui| {
                                    let (r, _) = ui.allocate_exact_size(
                                        egui::vec2(10.0, 10.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(r, egui::Rounding::same(3.0), *c);
                                    ui.label(n.as_str());
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(RichText::new(f::money(*v)).strong());
                                    });
                                });
                            }
                        }
                    });
                },
            );
            ui.add_space(14.0);
            card(ui, |ui| {
                ui.set_width(w - 38.0);
                section_title(ui, "Monthly expenses");
                ui.add_space(4.0);
                charts::legend(
                    ui,
                    &[
                        (p.accent, tr("Fuel")),
                        (p.warn, tr("Maintenance")),
                        (p.purple, tr("Other expenses")),
                    ],
                );
                charts::stacked_bars(ui, "fuel_monthly", &s.monthly, 190.0, &|v| f::money(v));
            });
            ui.add_space(10.0);
        });
}

fn fuel_dialog(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    let mut close = false;
    if let Some(form) = st.fuel_form.as_mut() {
        let title = if form.id == 0 {
            "Add refueling"
        } else {
            "Edit refueling"
        };
        let vehicles = cache.vehicle_items(form.id != 0);
        let drivers = cache.driver_items(true);
        let mut cancel = false;
        let mut save = false;
        let closed = dialogs::modal(ui.ctx(), "fuel_form", title, 700.0, |ui| {
            error_text(ui, &form.error);
            egui::Grid::new("fuel_form_grid")
                .num_columns(4)
                .spacing([14.0, 10.0])
                .show(ui, |ui| {
                    widgets::form_label(ui, "Vehicle", true);
                    let ch = widgets::combo_id(
                        ui,
                        "ff_form_vehicle",
                        &mut form.vehicle_id,
                        &vehicles,
                        None,
                        220.0,
                    );
                    if ch && form.id == 0 {
                        if let Some(v) = cache.vehicle(form.vehicle_id) {
                            form.mileage = v.mileage.to_string();
                        }
                        if let Some(a) = cache.open_assignment_of_vehicle(form.vehicle_id) {
                            form.driver_id = a.driver_id;
                        }
                    }
                    widgets::form_label(ui, "Driver", false);
                    widgets::combo_id(
                        ui,
                        "ff_form_driver",
                        &mut form.driver_id,
                        &drivers,
                        Some("None"),
                        220.0,
                    );
                    ui.end_row();
                    widgets::form_date(ui, "Date", true, &mut form.date, 220.0);
                    widgets::form_text(ui, "Mileage", true, &mut form.mileage, 220.0);
                    ui.end_row();
                    widgets::form_text(ui, "Liters", true, &mut form.liters, 220.0);
                    widgets::form_text(ui, "Price per liter", true, &mut form.price, 220.0);
                    ui.end_row();
                    widgets::form_text(ui, "Fuel station", false, &mut form.station, 220.0);
                    widgets::form_label(ui, "Tank", false);
                    ui.checkbox(&mut form.full_tank, tr("Full tank"));
                    ui.end_row();
                });
            let l = f::parse_num(&form.liters, "").unwrap_or(0.0);
            let pr = f::parse_num(&form.price, "").unwrap_or(0.0);
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!("{}: {}", tr("Total cost"), f::money(l * pr)))
                    .strong()
                    .size(15.0),
            );
            dialogs::footer(ui, "Save", &mut cancel, &mut save);
        });
        if closed || cancel {
            close = true;
        }
        if save {
            match form.build() {
                Ok(x) => match cx.db.save_fuel(&x) {
                    Ok(_) => {
                        let verb = if x.id == 0 { "added" } else { "updated" };
                        let log = format!(
                            "Refueling {}: {} ({} L)",
                            verb,
                            cache.vehicle_name(x.vehicle_id),
                            f::num(x.liters, 1)
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
        st.fuel_form = None;
    }
}

fn expense_dialog(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    let mut close = false;
    if let Some(form) = st.exp_form.as_mut() {
        let title = if form.id == 0 {
            "Add expense"
        } else {
            "Edit expense"
        };
        let vehicles = cache.vehicle_items(form.id != 0);
        let mut cancel = false;
        let mut save = false;
        let closed = dialogs::modal(ui.ctx(), "expense_form", title, 640.0, |ui| {
            error_text(ui, &form.error);
            egui::Grid::new("exp_form_grid")
                .num_columns(4)
                .spacing([14.0, 10.0])
                .show(ui, |ui| {
                    widgets::form_label(ui, "Vehicle", true);
                    widgets::combo_id(
                        ui,
                        "ef_form_vehicle",
                        &mut form.vehicle_id,
                        &vehicles,
                        None,
                        200.0,
                    );
                    widgets::form_combo(
                        ui,
                        "Category",
                        true,
                        "ef_form_cat",
                        &mut form.category,
                        &EXPENSE_CATS,
                        200.0,
                    );
                    ui.end_row();
                    widgets::form_date(ui, "Date", true, &mut form.date, 200.0);
                    widgets::form_text(ui, "Amount", true, &mut form.amount, 200.0);
                    ui.end_row();
                    widgets::form_text(ui, "Description", false, &mut form.description, 200.0);
                    ui.end_row();
                });
            dialogs::footer(ui, "Save", &mut cancel, &mut save);
        });
        if closed || cancel {
            close = true;
        }
        if save {
            match form.build() {
                Ok(x) => match cx.db.save_expense(&x) {
                    Ok(_) => {
                        let verb = if x.id == 0 { "added" } else { "updated" };
                        let log = format!(
                            "Expense {}: {} - {}",
                            verb,
                            cache.vehicle_name(x.vehicle_id),
                            x.category
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
        st.exp_form = None;
    }
}
