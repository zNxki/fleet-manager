use super::{users, Ctx, Page};
use crate::db::Db;
use crate::i18n::{self, tr};
use crate::ui::dialogs::Action;
use crate::ui::theme::{self, pal};
use crate::ui::widgets::{self, card, muted, page_header, section_title};
use crate::util::{files, format as f};
use eframe::egui::{self, RichText, Ui};

#[derive(Default)]
pub struct State {
    init: bool,
    dark: bool,
    animations: bool,
    lang: usize,
    currency: usize,
    date_fmt: usize,
    start_page: usize,
    users: users::State,
}

fn load(st: &mut State, db: &Db) {
    st.init = true;
    st.dark = db.setting("theme").map(|v| v != "light").unwrap_or(true);
    st.animations = db.setting("animations").map(|v| v != "off").unwrap_or(true);
    st.lang = db
        .setting("lang")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
        .min(i18n::LANGUAGES.len() - 1);
    st.currency = db
        .setting("currency")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
        .min(f::CURRENCIES.len() - 1);
    st.date_fmt = db
        .setting("date_fmt")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
        .min(f::DATE_FORMATS.len() - 1);
    let code = db.setting("start_page").unwrap_or_default();
    st.start_page = Page::ALL
        .iter()
        .position(|p| p.code() == code)
        .unwrap_or(0);
}

pub(super) fn row(ui: &mut Ui, title: &str, hint: &str, add: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(300.0);
            ui.label(RichText::new(tr(title)).strong());
            muted(ui, &tr(hint));
        });
        ui.with_layout(
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| add(ui),
        );
    });
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
}

pub fn show(ui: &mut Ui, cx: &mut Ctx, st: &mut State) {
    if !st.init {
        load(st, cx.db);
    }
    page_header(ui, "Settings", "Customize FleetManager and manage your data", |_| {});
    let ctx = ui.ctx().clone();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let w = (ui.available_width() - 8.0).min(900.0);
            card(ui, |ui| {
                ui.set_width(w - 38.0);
                section_title(ui, "Appearance");
                ui.add_space(8.0);
                row(ui, "Theme", "Choose between dark and light interface", |ui| {
                    let mut changed = false;
                    if ui.selectable_label(!st.dark, tr("Light")).clicked() && st.dark {
                        st.dark = false;
                        changed = true;
                    }
                    if ui.selectable_label(st.dark, tr("Dark")).clicked() && !st.dark {
                        st.dark = true;
                        changed = true;
                    }
                    if changed {
                        theme::apply(&ctx, st.dark);
                        cx.db.set_setting("theme", if st.dark { "dark" } else { "light" });
                    }
                });
                row(
                    ui,
                    "Animations",
                    "Smooth transitions and animated charts",
                    |ui| {
                        if ui.checkbox(&mut st.animations, tr("Enabled")).changed() {
                            theme::set_animations(st.animations);
                            cx.db
                                .set_setting("animations", if st.animations { "on" } else { "off" });
                        }
                    },
                );
            });
            ui.add_space(14.0);
            card(ui, |ui| {
                ui.set_width(w - 38.0);
                section_title(ui, "Regional");
                ui.add_space(8.0);
                row(ui, "Language", "Language of the application", |ui| {
                    let cur = i18n::LANGUAGES[st.lang];
                    egui::ComboBox::from_id_salt("set_lang")
                        .width(200.0)
                        .selected_text(cur)
                        .show_ui(ui, |ui| {
                            for (i, l) in i18n::LANGUAGES.iter().enumerate() {
                                if ui.selectable_label(st.lang == i, *l).clicked() {
                                    st.lang = i;
                                    i18n::set_lang(i);
                                    cx.db.set_setting("lang", &i.to_string());
                                    crate::ui::anim::bump();
                                }
                            }
                        });
                });
                row(ui, "Currency", "Currency used for amounts", |ui| {
                    egui::ComboBox::from_id_salt("set_cur")
                        .width(200.0)
                        .selected_text(f::CURRENCIES[st.currency])
                        .show_ui(ui, |ui| {
                            for (i, c) in f::CURRENCIES.iter().enumerate() {
                                if ui.selectable_label(st.currency == i, *c).clicked() {
                                    st.currency = i;
                                    f::set_cur_idx(i);
                                    cx.db.set_setting("currency", &i.to_string());
                                }
                            }
                        });
                });
                row(ui, "Date format", "How dates are displayed and entered", |ui| {
                    egui::ComboBox::from_id_salt("set_date")
                        .width(200.0)
                        .selected_text(f::DATE_FORMATS[st.date_fmt].0)
                        .show_ui(ui, |ui| {
                            for (i, (name, _)) in f::DATE_FORMATS.iter().enumerate() {
                                if ui.selectable_label(st.date_fmt == i, *name).clicked() {
                                    st.date_fmt = i;
                                    f::set_date_idx(i);
                                    cx.db.set_setting("date_fmt", &i.to_string());
                                }
                            }
                        });
                });
            });
            ui.add_space(14.0);
            card(ui, |ui| {
                ui.set_width(w - 38.0);
                section_title(ui, "Preferences");
                ui.add_space(8.0);
                row(ui, "Start page", "Page shown when the application starts", |ui| {
                    egui::ComboBox::from_id_salt("set_start")
                        .width(200.0)
                        .selected_text(tr(Page::ALL[st.start_page].label()))
                        .show_ui(ui, |ui| {
                            for (i, pg) in Page::ALL.iter().enumerate() {
                                if ui.selectable_label(st.start_page == i, tr(pg.label())).clicked()
                                {
                                    st.start_page = i;
                                    cx.db.set_setting("start_page", pg.code());
                                }
                            }
                        });
                });
            });
            ui.add_space(14.0);
            users::account_card(ui, cx, &mut st.users, w);
            if cx.user.is_admin {
                ui.add_space(14.0);
                users::users_card(ui, cx, &mut st.users, w);
            }
            ui.add_space(14.0);
            card(ui, |ui| {
                ui.set_width(w - 38.0);
                section_title(ui, "Database");
                ui.add_space(8.0);
                row(ui, "Database location", &cx.db.path.to_string_lossy(), |ui| {
                    if widgets::secondary(ui, "Open folder").clicked() {
                        if let Some(dir) = cx.db.path.parent() {
                            files::open_path(&dir.to_string_lossy());
                        }
                    }
                });
                if cx.user.is_admin {
                row(
                    ui,
                    "Backup",
                    "Save a copy of all your data to a file",
                    |ui| {
                        if widgets::primary(ui, "Create backup").clicked() {
                            let name = format!(
                                "fleetmanager_backup_{}.db",
                                chrono::Local::now().format("%Y%m%d_%H%M")
                            );
                            let dlg = rfd::FileDialog::new()
                                .set_title(tr("Create backup"))
                                .set_file_name(name)
                                .add_filter("FleetManager backup", &["db"]);
                            if let Some(path) = dlg.save_file() {
                                match cx.db.backup(&path) {
                                    Ok(()) => {
                                        cx.db.log("Database backup created");
                                        *cx.dirty = true;
                                        cx.toasts.success("Backup created successfully");
                                    }
                                    Err(e) => cx.fail(&e),
                                }
                            }
                        }
                    },
                );
                row(
                    ui,
                    "Restore",
                    "Replace all current data with a backup file",
                    |ui| {
                        if widgets::secondary(ui, "Restore backup").clicked() {
                            let dlg = rfd::FileDialog::new()
                                .set_title(tr("Restore backup"))
                                .add_filter("FleetManager backup", &["db"]);
                            if let Some(path) = dlg.pick_file() {
                                match Db::validate_backup(&path) {
                                    Ok(()) => cx.ask(
                                        "Restore backup",
                                        "All current data will be replaced by the content of the selected backup.",
                                        "Restore",
                                        Action::Restore(path),
                                        "Database restored from backup".to_string(),
                                    ),
                                    Err(e) => cx.fail(&e),
                                }
                            }
                        }
                    },
                );
                }
            });
            ui.add_space(14.0);
            card(ui, |ui| {
                ui.set_width(w - 38.0);
                section_title(ui, "About");
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!("FleetManager {}", env!("CARGO_PKG_VERSION")))
                        .strong()
                        .color(pal().accent),
                );
                muted(
                    ui,
                    &tr("Native desktop application. All data is stored locally and works offline."),
                );
            });
            ui.add_space(10.0);
        });
    users::dialogs(&ctx, cx, &mut st.users);
}
