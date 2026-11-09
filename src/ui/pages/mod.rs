pub mod assignments;
pub mod dashboard;
pub mod documents;
pub mod drivers;
pub mod fuel;
pub mod maintenance;
pub mod reports;
pub mod settings;
pub mod users;
pub mod vehicles;

use super::dialogs::{Action, Confirm};
use super::icons::Icon;
use super::images::ImageCache;
use super::toast::Toasts;
use crate::db::cache::Cache;
use crate::db::users::User;
use crate::db::Db;
use eframe::egui::Ui;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Page {
    Dashboard,
    Vehicles,
    Drivers,
    Assignments,
    Maintenance,
    Fuel,
    Documents,
    Reports,
    Settings,
}

impl Page {
    pub const ALL: [Page; 9] = [
        Page::Dashboard,
        Page::Vehicles,
        Page::Drivers,
        Page::Assignments,
        Page::Maintenance,
        Page::Fuel,
        Page::Documents,
        Page::Reports,
        Page::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Page::Dashboard => "Dashboard",
            Page::Vehicles => "Vehicles",
            Page::Drivers => "Drivers",
            Page::Assignments => "Assignments",
            Page::Maintenance => "Maintenance",
            Page::Fuel => "Fuel & Expenses",
            Page::Documents => "Documents",
            Page::Reports => "Reports",
            Page::Settings => "Settings",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Page::Dashboard => Icon::Dashboard,
            Page::Vehicles => Icon::Vehicles,
            Page::Drivers => Icon::Drivers,
            Page::Assignments => Icon::Assignments,
            Page::Maintenance => Icon::Maintenance,
            Page::Fuel => Icon::Fuel,
            Page::Documents => Icon::Documents,
            Page::Reports => Icon::Reports,
            Page::Settings => Icon::Settings,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Page::Dashboard => "dashboard",
            Page::Vehicles => "vehicles",
            Page::Drivers => "drivers",
            Page::Assignments => "assignments",
            Page::Maintenance => "maintenance",
            Page::Fuel => "fuel",
            Page::Documents => "documents",
            Page::Reports => "reports",
            Page::Settings => "settings",
        }
    }

    pub fn from_code(s: &str) -> Page {
        Page::ALL
            .iter()
            .copied()
            .find(|p| p.code() == s)
            .unwrap_or(Page::Dashboard)
    }
}

pub struct Ctx<'a> {
    pub db: &'a Db,
    pub user: &'a User,
    pub cache: &'a Cache,
    pub toasts: &'a mut Toasts,
    pub confirm: &'a mut Option<Confirm>,
    pub dirty: &'a mut bool,
    pub nav: &'a mut Option<Page>,
    pub images: &'a mut ImageCache,
}

impl<'a> Ctx<'a> {
    pub fn saved(&mut self, log: &str) {
        self.db.log(log);
        *self.dirty = true;
        self.toasts.success("Saved successfully");
    }

    pub fn fail(&mut self, e: &str) {
        self.toasts.error(e);
    }

    pub fn ask(&mut self, title: &str, message: &str, button: &str, action: Action, log: String) {
        *self.confirm = Some(Confirm {
            title: title.to_string(),
            message: message.to_string(),
            button: button.to_string(),
            action,
            log,
        });
    }

    pub fn go(&mut self, page: Page) {
        *self.nav = Some(page);
    }
}

#[derive(Default)]
pub struct Pages {
    pub vehicles: vehicles::State,
    pub drivers: drivers::State,
    pub assignments: assignments::State,
    pub maintenance: maintenance::State,
    pub fuel: fuel::State,
    pub documents: documents::State,
    pub reports: reports::State,
    pub settings: settings::State,
}

pub fn show(page: Page, ui: &mut Ui, cx: &mut Ctx, st: &mut Pages) {
    match page {
        Page::Dashboard => dashboard::show(ui, cx),
        Page::Vehicles => vehicles::show(ui, cx, &mut st.vehicles),
        Page::Drivers => drivers::show(ui, cx, &mut st.drivers),
        Page::Assignments => assignments::show(ui, cx, &mut st.assignments),
        Page::Maintenance => maintenance::show(ui, cx, &mut st.maintenance),
        Page::Fuel => fuel::show(ui, cx, &mut st.fuel),
        Page::Documents => documents::show(ui, cx, &mut st.documents),
        Page::Reports => reports::show(ui, cx, &mut st.reports),
        Page::Settings => settings::show(ui, cx, &mut st.settings),
    }
}
