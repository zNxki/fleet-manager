use super::settings::row;
use super::Ctx;
use crate::db::users::User;
use crate::i18n::tr;
use crate::ui::dialogs::{self, Action};
use crate::ui::login::{self, ChangeState};
use crate::ui::theme::pal;
use crate::ui::widgets::{self, badge, card, error_text, muted, section_title};
use eframe::egui::{self, Context, FontId, RichText, Ui};

#[derive(Default)]
struct AddForm {
    username: String,
    admin: bool,
    error: String,
}

struct Created {
    title: &'static str,
    username: String,
    password: String,
}

#[derive(Default)]
pub struct State {
    users: Vec<User>,
    add: Option<AddForm>,
    created: Option<Created>,
    change: Option<ChangeState>,
}

/// The list is tiny, so it is re-read every frame: always current after any change
/// (including deletions confirmed in the global dialog).
fn refresh(st: &mut State, cx: &mut Ctx) {
    if let Ok(v) = cx.db.list_users() {
        st.users = v;
    }
}

fn role_label(u: &User) -> &'static str {
    if u.is_admin {
        "Administrator"
    } else {
        "User"
    }
}

/// "Account" card: visible to everybody, lets the signed-in user change their password.
pub fn account_card(ui: &mut Ui, cx: &mut Ctx, st: &mut State, w: f32) {
    card(ui, |ui| {
        ui.set_width(w - 38.0);
        section_title(ui, "Account");
        ui.add_space(8.0);
        let hint = format!("{} - {}", cx.user.username, tr(role_label(cx.user)));
        row(ui, "Signed in as", &hint, |ui| {
            if widgets::secondary(ui, "Change password").clicked() {
                st.change = Some(ChangeState::default());
            }
        });
    });
}

/// "Users" card: administrators only.
pub fn users_card(ui: &mut Ui, cx: &mut Ctx, st: &mut State, w: f32) {
    refresh(st, cx);
    let p = pal();
    let me = cx.user.id;
    let mut reset: Option<i64> = None;
    let mut toggle: Option<(i64, bool)> = None;
    let mut delete: Option<(i64, String)> = None;
    card(ui, |ui| {
        ui.set_width(w - 38.0);
        ui.horizontal(|ui| {
            section_title(ui, "Users");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::primary(ui, "+ Add user").clicked() {
                    st.add = Some(AddForm::default());
                }
            });
        });
        ui.add_space(8.0);
        egui::Grid::new("users_grid")
            .num_columns(5)
            .spacing([18.0, 10.0])
            .striped(false)
            .show(ui, |ui| {
                for h in ["Username", "Role", "Status", "Last sign-in", ""] {
                    ui.label(RichText::new(tr(h)).size(12.0).strong().color(p.muted));
                }
                ui.end_row();
                for u in &st.users {
                    let is_me = u.id == me;
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&u.username).strong());
                        if is_me {
                            muted(ui, &tr("(you)"));
                        }
                    });
                    badge(ui, role_label(u), if u.is_admin { p.purple } else { p.accent });
                    ui.horizontal(|ui| {
                        if u.active {
                            badge(ui, "Active", p.success);
                        } else {
                            badge(ui, "Disabled", p.muted);
                        }
                        if u.must_change {
                            badge(ui, "Password change pending", p.warn);
                        }
                    });
                    let last = if u.last_login.is_empty() {
                        tr("Never")
                    } else {
                        u.last_login.clone()
                    };
                    muted(ui, &last);
                    ui.horizontal(|ui| {
                        if widgets::small(ui, "Reset password").clicked() {
                            reset = Some(u.id);
                        }
                        if !is_me {
                            let label = if u.active { "Disable" } else { "Enable" };
                            if widgets::small(ui, label).clicked() {
                                toggle = Some((u.id, !u.active));
                            }
                            if widgets::small_danger(ui, "Delete").clicked() {
                                delete = Some((u.id, u.username.clone()));
                            }
                        }
                    });
                    ui.end_row();
                }
            });
    });

    if let Some(id) = reset {
        let name = st
            .users
            .iter()
            .find(|u| u.id == id)
            .map(|u| u.username.clone())
            .unwrap_or_default();
        match cx.db.reset_password(id) {
            Ok(pw) => {
                st.created = Some(Created {
                    title: "Password reset",
                    username: name,
                    password: pw,
                });
                *cx.dirty = true;
            }
            Err(e) => cx.fail(&e),
        }
    }
    if let Some((id, active)) = toggle {
        let name = st
            .users
            .iter()
            .find(|u| u.id == id)
            .map(|u| u.username.clone())
            .unwrap_or_default();
        match cx.db.set_user_active(id, active) {
            Ok(()) => {
                let msg = if active { "User enabled" } else { "User disabled" };
                cx.saved(&format!("{msg}: {name}"));
            }
            Err(e) => cx.fail(&e),
        }
    }
    if let Some((id, name)) = delete {
        cx.ask(
            "Delete user",
            "The user will no longer be able to sign in.",
            "Delete",
            Action::DeleteUser(id),
            format!("User deleted: {name}"),
        );
    }
}

/// Modal dialogs of this section: call once per frame after the page content.
pub fn dialogs(ctx: &Context, cx: &mut Ctx, st: &mut State) {
    if let Some(mut f) = st.add.take() {
        let mut cancel = false;
        let mut save = false;
        let closed = dialogs::modal(ctx, "user_add", "Add user", 460.0, |ui| {
            error_text(ui, &f.error);
            widgets::form_text(ui, "Username", true, &mut f.username, 340.0);
            ui.add_space(8.0);
            ui.checkbox(&mut f.admin, tr("Administrator (can manage users)"));
            ui.add_space(8.0);
            muted(
                ui,
                &tr("A temporary password will be generated. The user must change it at first sign-in."),
            );
            dialogs::footer(ui, "Create user", &mut cancel, &mut save);
        });
        if save {
            match cx.db.create_user(&f.username, f.admin) {
                Ok(pw) => {
                    st.created = Some(Created {
                        title: "User created",
                        username: f.username.trim().to_string(),
                        password: pw,
                    });
                        *cx.dirty = true;
                }
                Err(e) => {
                    f.error = e;
                    st.add = Some(f);
                }
            }
        } else if !(closed || cancel) {
            st.add = Some(f);
        }
    }

    if let Some(c) = &st.created {
        let p = pal();
        let closed = dialogs::info(ctx, "user_created", c.title, 480.0, |ui| {
            ui.horizontal(|ui| {
                muted(ui, &tr("Username"));
                ui.label(RichText::new(&c.username).strong());
            });
            ui.add_space(6.0);
            muted(ui, &tr("Temporary password"));
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&c.password)
                        .font(FontId::monospace(20.0))
                        .strong()
                        .color(p.accent),
                );
                if widgets::small(ui, "Copy").clicked() {
                    ui.ctx().copy_text(c.password.clone());
                }
            });
            ui.add_space(8.0);
            muted(
                ui,
                &tr("Write it down now: it will not be shown again. The user must change it at first sign-in."),
            );
        });
        if closed {
            st.created = None;
        }
    }

    if let Some(mut c) = st.change.take() {
        let mut cancel = false;
        let mut save = false;
        let mut enter = false;
        let closed = dialogs::modal(ctx, "pw_change", "Change password", 440.0, |ui| {
            enter = login::password_fields(ui, &mut c, true, 340.0);
            dialogs::footer(ui, "Save password", &mut cancel, &mut save);
        });
        if save || enter {
            match cx
                .db
                .change_password(cx.user.id, Some(&c.current), &c.new1, &c.new2)
            {
                Ok(()) => {
                    *cx.dirty = true;
                    cx.toasts.success("Password changed successfully");
                }
                Err(e) => {
                    c.error = e;
                    st.change = Some(c);
                }
            }
        } else if !(closed || cancel) {
            st.change = Some(c);
        }
    }
}
