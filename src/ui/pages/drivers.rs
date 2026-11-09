use super::Ctx;
use crate::db::models::*;
use crate::i18n::tr;
use crate::ui::dialogs::{self, Action};
use crate::ui::theme::{level_color, pal};
use crate::ui::widgets::{
    self, apply_sort, cell, cell_strong, contains_ci, empty, error_text, expiry_level,
    filter_combo, kv, level_badge, muted, page_header, primary, row_actions, search, status_badge,
    table, Col, Sort,
};
use crate::util::{format as f, input};
use eframe::egui::{self, Align, Layout, RichText, Ui};
use std::cmp::Ordering;

const LICENSE_FILTER: [&str; 3] = ["Expired", "Expiring soon", "Valid"];
const DRIVER_STATUS: [&str; 2] = ["Active", "Inactive"];

#[derive(Default)]
pub struct State {
    search: String,
    status: String,
    license: String,
    sort: Sort,
    form: Option<Form>,
    view: Option<i64>,
}

#[derive(Clone, Default)]
pub struct Form {
    id: i64,
    first_name: String,
    last_name: String,
    tax_code: String,
    birth_date: String,
    phone: String,
    email: String,
    address: String,
    license_number: String,
    license_category: String,
    license_expiry: String,
    hire_date: String,
    job_title: String,
    employment_type: String,
    active: bool,
    notes: String,
    error: String,
}

impl Form {
    fn blank() -> Form {
        Form {
            license_category: "B".to_string(),
            employment_type: EMPLOYMENT[0].to_string(),
            active: true,
            hire_date: f::today_input(),
            ..Default::default()
        }
    }

    fn from(d: &Driver) -> Form {
        Form {
            id: d.id,
            first_name: d.first_name.clone(),
            last_name: d.last_name.clone(),
            tax_code: d.tax_code.clone(),
            birth_date: f::date(&d.birth_date),
            phone: d.phone.clone(),
            email: d.email.clone(),
            address: d.address.clone(),
            license_number: d.license_number.clone(),
            license_category: d.license_category.clone(),
            license_expiry: f::date(&d.license_expiry),
            hire_date: f::date(&d.hire_date),
            job_title: d.job_title.clone(),
            employment_type: d.employment_type.clone(),
            active: d.active,
            notes: d.notes.clone(),
            error: String::new(),
        }
    }

    fn build(&self) -> Result<Driver, String> {
        let first_name = input::required(&self.first_name, &tr("First name"))?;
        let last_name = input::required(&self.last_name, &tr("Last name"))?;
        let license_number =
            input::required(&self.license_number, &tr("License number"))?.to_uppercase();
        let license_expiry = f::require_date(&self.license_expiry, &tr("License expiry"))?;
        let birth_date = f::parse_date(&self.birth_date, &tr("Birth date"))?;
        if !birth_date.is_empty() {
            let b = f::parse_iso(&birth_date).ok_or("Invalid birth date")?;
            let age_days = (f::today() - b).num_days();
            if age_days < 0 {
                return Err("The birth date cannot be in the future".to_string());
            }
            if age_days < 18 * 365 {
                return Err("The driver must be at least 18 years old".to_string());
            }
        }
        let hire_date = f::parse_date(&self.hire_date, &tr("Hire date"))?;
        if !input::email_ok(&self.email) {
            return Err("Invalid email address".to_string());
        }
        let phone = self.phone.trim().to_string();
        if !phone
            .chars()
            .all(|c| c.is_ascii_digit() || "+-() ./".contains(c))
        {
            return Err("Invalid phone number".to_string());
        }
        Ok(Driver {
            id: self.id,
            first_name,
            last_name,
            tax_code: self.tax_code.trim().to_uppercase(),
            birth_date,
            phone,
            email: self.email.trim().to_string(),
            address: self.address.trim().to_string(),
            license_number,
            license_category: self.license_category.clone(),
            license_expiry,
            hire_date,
            job_title: self.job_title.trim().to_string(),
            employment_type: self.employment_type.clone(),
            active: self.active,
            notes: self.notes.trim().to_string(),
        })
    }
}

fn license_state(d: &Driver) -> &'static str {
    match expiry_level(&d.license_expiry).0 {
        2 => "Expired",
        1 => "Expiring soon",
        _ => "Valid",
    }
}

pub fn show(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    let cache = cx.cache;
    page_header(
        ui,
        "Drivers",
        "Manage drivers, licenses and employment details",
        |ui| {
            if primary(ui, "+ Add driver").clicked() {
                st.form = Some(Form::blank());
            }
        },
    );

    ui.horizontal(|ui| {
        search(ui, &mut st.search);
        filter_combo(
            ui,
            "df_status",
            &mut st.status,
            &DRIVER_STATUS,
            "All statuses",
        );
        filter_combo(
            ui,
            "df_license",
            &mut st.license,
            &LICENSE_FILTER,
            "All licenses",
        );
    });
    ui.add_space(10.0);

    let mut idx: Vec<usize> = cache
        .drivers
        .iter()
        .enumerate()
        .filter(|(_, d)| {
            let status = if d.active { "Active" } else { "Inactive" };
            (st.status.is_empty() || status == st.status)
                && (st.license.is_empty() || license_state(d) == st.license)
                && (contains_ci(&d.first_name, &st.search)
                    || contains_ci(&d.last_name, &st.search)
                    || contains_ci(&d.license_number, &st.search)
                    || contains_ci(&d.tax_code, &st.search)
                    || contains_ci(&d.email, &st.search))
        })
        .map(|(i, _)| i)
        .collect();
    apply_sort(&mut idx, st.sort, |col, a, b| {
        let (x, y) = (&cache.drivers[a], &cache.drivers[b]);
        match col {
            0 => (x.last_name.to_lowercase(), x.first_name.to_lowercase())
                .cmp(&(y.last_name.to_lowercase(), y.first_name.to_lowercase())),
            1 => x.license_number.cmp(&y.license_number),
            2 => x.license_category.cmp(&y.license_category),
            3 => x.license_expiry.cmp(&y.license_expiry),
            4 => x.employment_type.cmp(&y.employment_type),
            5 => x.active.cmp(&y.active),
            _ => Ordering::Equal,
        }
    });

    let mut act: Option<(u8, i64)> = None;
    if cache.drivers.is_empty() {
        empty(
            ui,
            "No drivers yet",
            "Add your first driver to start assigning vehicles",
        );
    } else if idx.is_empty() {
        empty(ui, "No results", "Try changing the search or the filters");
    } else {
        let cols = [
            Col::new("Driver", 0.0).sortable(),
            Col::new("License", 150.0).sortable(),
            Col::new("Category", 90.0).sortable(),
            Col::new("License expiry", 150.0).sortable(),
            Col::new("Employment", 120.0).sortable(),
            Col::new("Status", 100.0).sortable(),
            Col::new("Vehicle", 120.0),
            Col::new("Actions", 210.0),
        ];
        let n = idx.len();
        table(ui, "drivers_table", &cols, n, &mut st.sort, |ui, r, c| {
            let d = &cache.drivers[idx[r]];
            match c {
                0 => cell_strong(ui, &format!("{} {}", d.last_name, d.first_name)),
                1 => cell(ui, &d.license_number),
                2 => cell(ui, &d.license_category),
                3 => {
                    let (l, _) = expiry_level(&d.license_expiry);
                    level_badge(ui, l, &f::date(&d.license_expiry));
                }
                4 => cell(ui, &tr(&d.employment_type)),
                5 => status_badge(ui, if d.active { "Active" } else { "Inactive" }),
                6 => {
                    let v = cache
                        .open_assignment_of_driver(d.id)
                        .map(|a| cache.vehicle_name(a.vehicle_id))
                        .unwrap_or_else(|| "-".to_string());
                    cell(ui, &v);
                }
                _ => {
                    let (a, b, c2) = row_actions(ui, true, true, true);
                    if a {
                        act = Some((1, d.id));
                    }
                    if b {
                        act = Some((2, d.id));
                    }
                    if c2 {
                        act = Some((3, d.id));
                    }
                }
            }
        });
    }

    if let Some((kind, id)) = act {
        if let Some(d) = cache.driver(id) {
            match kind {
                1 => st.view = Some(id),
                2 => st.form = Some(Form::from(d)),
                _ => {
                    let name = format!("{} {}", d.last_name, d.first_name);
                    let msg = format!(
                        "{} {}? {}",
                        tr("Delete driver"),
                        name,
                        tr("Assignment history and documents of this driver will be deleted too.")
                    );
                    cx.ask(
                        "Delete driver",
                        &msg,
                        "Delete",
                        Action::DeleteDriver(id),
                        format!("Driver deleted: {}", name),
                    );
                }
            }
        }
    }

    let mut close = false;
    if let Some(form) = st.form.as_mut() {
        let title = if form.id == 0 {
            "Add driver"
        } else {
            "Edit driver"
        };
        let mut cancel = false;
        let mut save = false;
        let closed = dialogs::modal(ui.ctx(), "driver_form", title, 960.0, |ui| {
            form_ui(ui, form);
            dialogs::footer(ui, "Save", &mut cancel, &mut save);
        });
        if closed || cancel {
            close = true;
        }
        if save {
            match form.build() {
                Ok(d) => match cx.db.save_driver(&d) {
                    Ok(_) => {
                        let log = if d.id == 0 {
                            format!("Driver added: {} {}", d.last_name, d.first_name)
                        } else {
                            format!("Driver updated: {} {}", d.last_name, d.first_name)
                        };
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

    if let Some(id) = st.view {
        match cache.driver(id) {
            Some(d) => {
                let closed =
                    dialogs::info(ui.ctx(), "driver_view", "Driver details", 800.0, |ui| {
                        view_ui(ui, cx, d)
                    });
                if closed {
                    st.view = None;
                }
            }
            None => st.view = None,
        }
    }
}

fn form_ui(ui: &mut Ui, form: &mut Form) {
    error_text(ui, &form.error);
    widgets::section_title(ui, "Personal information");
    ui.add_space(4.0);
    egui::Grid::new("drv_form_a")
        .num_columns(4)
        .spacing([14.0, 10.0])
        .show(ui, |ui| {
            widgets::form_text(ui, "First name", true, &mut form.first_name, 300.0);
            widgets::form_text(ui, "Last name", true, &mut form.last_name, 300.0);
            ui.end_row();
            widgets::form_text(ui, "Tax code", false, &mut form.tax_code, 300.0);
            widgets::form_date(ui, "Birth date", false, &mut form.birth_date, 300.0);
            ui.end_row();
            widgets::form_text(ui, "Phone", false, &mut form.phone, 300.0);
            widgets::form_text(ui, "Email", false, &mut form.email, 300.0);
            ui.end_row();
            widgets::form_text(ui, "Address", false, &mut form.address, 680.0);
            ui.end_row();
        });
    ui.add_space(10.0);
    widgets::section_title(ui, "Driving license");
    ui.add_space(4.0);
    egui::Grid::new("drv_form_b")
        .num_columns(4)
        .spacing([14.0, 10.0])
        .show(ui, |ui| {
            widgets::form_text(ui, "License number", true, &mut form.license_number, 300.0);
            widgets::form_combo(
                ui,
                "Category",
                true,
                "df_cat",
                &mut form.license_category,
                &LICENSE_CATS,
                300.0,
            );
            ui.end_row();
            widgets::form_date(ui, "License expiry", true, &mut form.license_expiry, 300.0);
            ui.end_row();
        });
    ui.add_space(10.0);
    widgets::section_title(ui, "Employment");
    ui.add_space(4.0);
    egui::Grid::new("drv_form_c")
        .num_columns(4)
        .spacing([14.0, 10.0])
        .show(ui, |ui| {
            widgets::form_text(ui, "Job title", false, &mut form.job_title, 300.0);
            widgets::form_combo(
                ui,
                "Employment type",
                false,
                "df_emp",
                &mut form.employment_type,
                &EMPLOYMENT,
                300.0,
            );
            ui.end_row();
            widgets::form_date(ui, "Hire date", false, &mut form.hire_date, 300.0);
            widgets::form_label(ui, "Status", false);
            ui.checkbox(&mut form.active, tr("Active"));
            ui.end_row();
        });
    ui.add_space(8.0);
    egui::Grid::new("drv_form_d")
        .num_columns(2)
        .spacing([14.0, 10.0])
        .show(ui, |ui| {
            widgets::form_multiline(ui, "Notes", &mut form.notes, 780.0);
            ui.end_row();
        });
}

fn view_ui(ui: &mut Ui, cx: &mut Ctx, d: &Driver) {
    let cache = cx.cache;
    let p = pal();
    let (level, due) = expiry_level(&d.license_expiry);
    if level > 0 {
        let c = level_color(level);
        egui::Frame::none()
            .fill(crate::ui::theme::tint(c, 28))
            .stroke(egui::Stroke::new(1.0, crate::ui::theme::tint(c, 120)))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "{}: {} ({})",
                        tr("Driving license"),
                        tr(license_state(d)),
                        due
                    ))
                    .color(c)
                    .strong(),
                );
            });
        ui.add_space(8.0);
    }
    egui::Grid::new("drv_view")
        .num_columns(4)
        .spacing([22.0, 7.0])
        .show(ui, |ui| {
            kv(ui, "Driver", &format!("{} {}", d.last_name, d.first_name));
            kv(ui, "Tax code", &d.tax_code);
            kv(ui, "Birth date", &f::date(&d.birth_date));
            kv(ui, "Phone", &d.phone);
            kv(ui, "Email", &d.email);
            kv(ui, "Address", &d.address);
            kv(ui, "License number", &d.license_number);
            kv(ui, "Category", &d.license_category);
            kv(ui, "License expiry", &f::date(&d.license_expiry));
            kv(ui, "Job title", &d.job_title);
            kv(ui, "Employment type", &tr(&d.employment_type));
            kv(ui, "Hire date", &f::date(&d.hire_date));
            ui.label(RichText::new(tr("Status")).color(p.muted).size(13.0));
            status_badge(ui, if d.active { "Active" } else { "Inactive" });
            ui.end_row();
            kv(ui, "Notes", &d.notes);
        });
    ui.add_space(12.0);
    widgets::section_title(ui, "Assignment history");
    ui.add_space(4.0);
    let list: Vec<_> = cache
        .assignments
        .iter()
        .filter(|a| a.driver_id == d.id)
        .collect();
    if list.is_empty() {
        muted(ui, &tr("No assignments recorded for this driver"));
    } else {
        egui::Grid::new("drv_hist")
            .num_columns(5)
            .striped(true)
            .spacing([20.0, 7.0])
            .show(ui, |ui| {
                for h in ["Vehicle", "From", "To", "Distance", "Status"] {
                    ui.label(RichText::new(tr(h)).color(p.muted).size(12.0).strong());
                }
                ui.end_row();
                for a in list.iter().take(40) {
                    ui.label(cache.vehicle_name(a.vehicle_id));
                    ui.label(f::date(&a.start_date));
                    ui.label(if a.end_date.is_empty() {
                        "-".to_string()
                    } else {
                        f::date(&a.end_date)
                    });
                    ui.label(match a.end_km {
                        Some(k) => f::km(k - a.start_km),
                        None => "-".to_string(),
                    });
                    widgets::status_badge(
                        ui,
                        if a.end_date.is_empty() {
                            "Open"
                        } else {
                            "Completed"
                        },
                    );
                    ui.end_row();
                }
            });
    }
    ui.add_space(10.0);
    let docs: Vec<_> = cache
        .documents
        .iter()
        .filter(|x| x.driver_id == d.id)
        .collect();
    if !docs.is_empty() {
        widgets::section_title(ui, "Documents");
        ui.add_space(4.0);
        for x in docs {
            let (l, t) = expiry_level(&x.expiry_date);
            ui.horizontal(|ui| {
                ui.label(format!("{} {}", tr(&x.doc_type), x.number));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    level_badge(ui, l, &t);
                });
            });
        }
    }
}
