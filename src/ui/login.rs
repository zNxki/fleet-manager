use super::icons::{self, Icon};
use super::theme::{pal, tint};
use super::widgets::{self, error_text};
use crate::db::users::User;
use crate::db::Db;
use crate::i18n::tr;
use eframe::egui::{
    self, vec2, Color32, Context, FontId, Frame, Key, Margin, RichText, Rounding, Sense, Stroke,
    TextEdit, Ui,
};
use std::time::{Duration, Instant};

const MAX_ATTEMPTS: u32 = 5;
const LOCK_SECS: u64 = 30;
const FIELD_W: f32 = 320.0;

#[derive(Default)]
pub struct State {
    pub username: String,
    pub password: String,
    pub error: String,
    /// Temporary admin password, shown only until the admin has changed it.
    pub initial_password: Option<String>,
    fails: u32,
    locked_until: Option<Instant>,
    focus_done: bool,
}

#[derive(Default)]
pub struct ChangeState {
    pub current: String,
    pub new1: String,
    pub new2: String,
    pub error: String,
}

pub enum Change {
    None,
    Done,
    Cancel,
}

fn logo(ui: &mut Ui, title: &str, subtitle: &str) {
    let p = pal();
    let (r, _) = ui.allocate_exact_size(vec2(56.0, 56.0), Sense::hover());
    ui.painter().rect_filled(r, Rounding::same(15.0), p.accent);
    icons::draw(ui.painter(), Icon::Vehicles, r.center(), 34.0, Color32::WHITE);
    ui.add_space(10.0);
    ui.label(RichText::new(title).size(24.0).strong());
    ui.label(RichText::new(tr(subtitle)).size(13.5).color(p.muted));
    ui.add_space(16.0);
}

fn wide_button(ui: &mut Ui, text: &str, enabled: bool) -> bool {
    let p = pal();
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(tr(text)).color(Color32::WHITE).strong())
            .fill(p.accent)
            .rounding(Rounding::same(8.0))
            .min_size(vec2(FIELD_W, 38.0)),
    )
    .on_hover_cursor(egui::CursorIcon::PointingHand)
    .clicked()
}

fn centered_card(ctx: &Context, height: f32, add: impl FnOnce(&mut Ui)) {
    let p = pal();
    egui::CentralPanel::default()
        .frame(Frame::none().fill(p.bg))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(((ui.available_height() - height) / 2.0).max(12.0));
                Frame::none()
                    .fill(p.card)
                    .stroke(Stroke::new(1.0, p.border))
                    .rounding(Rounding::same(16.0))
                    .inner_margin(Margin::same(32.0))
                    .show(ui, |ui| {
                        ui.set_width(FIELD_W);
                        ui.vertical_centered(|ui| add(ui));
                    });
            });
        });
}

fn left(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), add);
}

fn initial_notice(ui: &mut Ui, pw: &str) {
    let p = pal();
    Frame::none()
        .fill(tint(p.warn, 26))
        .stroke(Stroke::new(1.0, tint(p.warn, 110)))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_width(FIELD_W - 26.0);
            left(ui, |ui| {
                ui.label(
                    RichText::new(tr("First start: temporary administrator credentials"))
                        .strong()
                        .color(p.warn),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(tr("Username")).color(p.muted));
                    ui.label(RichText::new("admin").font(FontId::monospace(15.0)).strong());
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new(tr("Password")).color(p.muted));
                    ui.label(RichText::new(pw).font(FontId::monospace(15.0)).strong());
                    if widgets::small(ui, "Copy").clicked() {
                        ui.ctx().copy_text(pw.to_string());
                    }
                });
                ui.add_space(2.0);
                widgets::muted(
                    ui,
                    &tr("You will be asked to choose a new password at first sign-in."),
                );
            });
        });
    ui.add_space(12.0);
}

fn attempt(db: &Db, st: &mut State) -> Option<User> {
    if let Some(t) = st.locked_until {
        if Instant::now() < t {
            return None;
        }
        st.locked_until = None;
        st.fails = 0;
    }
    if st.username.trim().is_empty() || st.password.is_empty() {
        st.error = "Enter username and password".to_string();
        return None;
    }
    match db.authenticate(&st.username, &st.password) {
        Ok(u) => {
            st.username.clear();
            st.password.clear();
            st.error.clear();
            st.fails = 0;
            Some(u)
        }
        Err(e) => {
            st.password.clear();
            st.fails += 1;
            if st.fails >= MAX_ATTEMPTS {
                st.locked_until = Some(Instant::now() + Duration::from_secs(LOCK_SECS));
                st.error = "Too many failed attempts. Please wait 30 seconds.".to_string();
            } else {
                st.error = e;
            }
            None
        }
    }
}

pub fn show_login(ctx: &Context, db: &Db, st: &mut State) -> Option<User> {
    let mut result = None;
    let locked = st
        .locked_until
        .map(|t| Instant::now() < t)
        .unwrap_or(false);
    if locked {
        ctx.request_repaint_after(Duration::from_millis(500));
    }
    let height = if st.initial_password.is_some() { 640.0 } else { 440.0 };
    centered_card(ctx, height, |ui| {
        logo(ui, "FleetManager", "Sign in to continue");
        if let Some(pw) = st.initial_password.clone() {
            initial_notice(ui, &pw);
        }
        left(ui, |ui| {
            error_text(ui, &st.error);
            widgets::form_label(ui, "Username", false);
            let u = ui.add(
                TextEdit::singleline(&mut st.username)
                    .desired_width(FIELD_W)
                    .margin(vec2(10.0, 7.0)),
            );
            if !st.focus_done {
                u.request_focus();
                st.focus_done = true;
            }
            ui.add_space(8.0);
            let pw = widgets::form_password(ui, "Password", false, &mut st.password, FIELD_W);
            let pressed = ui.input(|i| i.key_pressed(Key::Enter));
            if u.lost_focus() && pressed && st.password.is_empty() {
                pw.request_focus();
            }
            let enter = pressed
                && (pw.lost_focus() || (u.lost_focus() && !st.password.is_empty()));
            ui.add_space(14.0);
            if (wide_button(ui, "Sign in", !locked) || enter) && !locked {
                result = attempt(db, st);
                if result.is_none() && !st.username.is_empty() {
                    pw.request_focus();
                }
            }
        });
    });
    result
}

/// Password fields shared by the forced and the voluntary change forms.
/// Returns true when the user pressed Enter in the last field.
pub fn password_fields(ui: &mut Ui, st: &mut ChangeState, ask_current: bool, width: f32) -> bool {
    error_text(ui, &st.error);
    if ask_current {
        widgets::form_password(ui, "Current password", true, &mut st.current, width);
        ui.add_space(8.0);
    }
    widgets::form_password(ui, "New password", true, &mut st.new1, width);
    ui.add_space(8.0);
    let last = widgets::form_password(ui, "Confirm new password", true, &mut st.new2, width);
    ui.add_space(4.0);
    widgets::muted(ui, &tr("At least 8 characters"));
    last.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter))
}

/// Screen shown right after signing in with a temporary password.
pub fn show_forced_change(ctx: &Context, db: &Db, user: &User, st: &mut ChangeState) -> Change {
    let mut out = Change::None;
    centered_card(ctx, 520.0, |ui| {
        logo(ui, &tr("Choose a new password"), "You must set a new password before continuing.");
        left(ui, |ui| {
            ui.horizontal(|ui| {
                widgets::muted(ui, &tr("Signed in as"));
                ui.label(RichText::new(&user.username).strong());
            });
            ui.add_space(10.0);
            let enter = password_fields(ui, st, false, FIELD_W);
            ui.add_space(14.0);
            if wide_button(ui, "Save password", true) || enter {
                match db.change_password(user.id, None, &st.new1, &st.new2) {
                    Ok(()) => {
                        *st = ChangeState::default();
                        out = Change::Done;
                    }
                    Err(e) => st.error = e,
                }
            }
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                if widgets::secondary(ui, "Log out").clicked() {
                    out = Change::Cancel;
                }
            });
        });
    });
    out
}
