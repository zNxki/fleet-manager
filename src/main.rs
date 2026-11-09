#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod db;
mod i18n;
mod ui;
mod util;

fn main() {
    let db = match db::Db::open() {
        Ok(d) => d,
        Err(e) => {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("FleetManager")
                .set_description(e)
                .show();
            return;
        }
    };
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("FleetManager")
            .with_inner_size([1366.0, 850.0])
            .with_min_inner_size([1060.0, 660.0]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "FleetManager",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, db)))),
    );
    if let Err(e) = result {
        eprintln!("{e}");
    }
}
