use crate::db::cache::Cache;
use crate::db::users::User;
use crate::db::Db;
use crate::i18n;
use crate::ui::dialogs::{self, Action, Confirm};
use crate::ui::images::ImageCache;
use crate::ui::login::{self, Change};
use crate::ui::pages::{self, Ctx, Page, Pages};
use crate::ui::theme::{self, pal};
use crate::ui::toast::Toasts;
use crate::ui::{anim, sidebar};
use crate::util::format as f;
use eframe::egui::{self, Frame, Margin};

pub struct App {
    db: Db,
    cache: Cache,
    toasts: Toasts,
    confirm: Option<Confirm>,
    dirty: bool,
    nav: Option<Page>,
    images: ImageCache,
    page: Page,
    pages: Pages,
    user: Option<User>,
    login: login::State,
    pw: login::ChangeState,
}

fn apply_saved(ctx: &egui::Context, db: &Db) {
    let dark = db.setting("theme").map(|v| v != "light").unwrap_or(true);
    theme::apply(ctx, dark);
    theme::set_animations(db.setting("animations").map(|v| v != "off").unwrap_or(true));
    i18n::set_lang(
        db.setting("lang")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
    );
    f::set_cur_idx(
        db.setting("currency")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
    );
    f::set_date_idx(
        db.setting("date_fmt")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
    );
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, db: Db) -> App {
        apply_saved(&cc.egui_ctx, &db);
        let page = Page::from_code(&db.setting("start_page").unwrap_or_default());
        let mut toasts = Toasts::default();
        let cache = match Cache::load(&db) {
            Ok(c) => c,
            Err(e) => {
                toasts.error(&e);
                Cache::default()
            }
        };
        let mut login = login::State::default();
        match db.bootstrap_admin() {
            Ok(Some(pw)) => login.initial_password = Some(pw),
            Ok(None) => login.initial_password = db.pending_initial_password(),
            Err(e) => toasts.error(&e),
        }
        App {
            db,
            cache,
            toasts,
            confirm: None,
            dirty: false,
            nav: None,
            images: ImageCache::default(),
            page,
            pages: Pages::default(),
            user: None,
            login,
            pw: login::ChangeState::default(),
        }
    }

    fn after_login(&mut self) {
        self.pages = Pages::default();
        self.page = Page::from_code(&self.db.setting("start_page").unwrap_or_default());
        self.confirm = None;
        self.nav = None;
        self.dirty = true;
        anim::bump();
    }

    fn logout(&mut self) {
        self.user = None;
        self.confirm = None;
        self.pw = login::ChangeState::default();
        self.login = login::State::default();
        self.login.initial_password = self.db.pending_initial_password();
        anim::bump();
    }

    fn reload(&mut self) {
        match Cache::load(&self.db) {
            Ok(c) => self.cache = c,
            Err(e) => self.toasts.error(&e),
        }
        self.dirty = false;
    }

    fn run(&mut self, ctx: &egui::Context, c: Confirm) {
        let res = match &c.action {
            Action::DeleteVehicle(id) => self.db.delete_vehicle(*id),
            Action::DeleteDriver(id) => self.db.delete_driver(*id),
            Action::DeleteAssignment(id) => self.db.delete_assignment(*id),
            Action::DeleteMaintenance(id) => self.db.delete_maintenance(*id),
            Action::DeleteFuel(id) => self.db.delete_fuel(*id),
            Action::DeleteExpense(id) => self.db.delete_expense(*id),
            Action::DeleteDocument(id) => self.db.delete_document(*id),
            Action::DeleteUser(id) => self.db.delete_user(*id),
            Action::Restore(path) => self.db.restore(path),
        };
        match res {
            Ok(()) => {
                if matches!(c.action, Action::Restore(_)) {
                    apply_saved(ctx, &self.db);
                    self.toasts.success("Backup restored successfully");
                    // users come from the backup too: everybody signs in again
                    let fresh = self.db.bootstrap_admin();
                    self.logout();
                    match fresh {
                        Ok(Some(pw)) => self.login.initial_password = Some(pw),
                        Ok(None) => {}
                        Err(e) => self.toasts.error(&e),
                    }
                } else {
                    self.toasts.success("Deleted successfully");
                }
                self.db.log(&c.log);
                self.dirty = true;
            }
            Err(e) => self.toasts.error(&e),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Some(user) = self.user.clone() else {
            if let Some(u) = login::show_login(ctx, &self.db, &mut self.login) {
                let must_change = u.must_change;
                self.user = Some(u);
                self.pw = login::ChangeState::default();
                if !must_change {
                    self.after_login();
                }
            }
            self.toasts.show(ctx);
            return;
        };
        if user.must_change {
            match login::show_forced_change(ctx, &self.db, &user, &mut self.pw) {
                Change::Done => {
                    self.login.initial_password = None;
                    match self.db.get_user(user.id) {
                        Ok(Some(u)) => {
                            self.user = Some(u);
                            self.after_login();
                            self.toasts.success("Password changed successfully");
                        }
                        _ => self.logout(),
                    }
                }
                Change::Cancel => self.logout(),
                Change::None => {}
            }
            self.toasts.show(ctx);
            return;
        }
        let mut logout = false;
        if self.dirty {
            self.reload();
        }
        if let Some(p) = self.nav.take() {
            if p != self.page {
                self.page = p;
                anim::bump();
            }
        }
        let p = pal();
        egui::SidePanel::left("sidebar")
            .exact_width(232.0)
            .resizable(false)
            .frame(
                Frame::none()
                    .fill(p.sidebar)
                    .stroke(egui::Stroke::new(1.0, p.border))
                    .inner_margin(Margin::same(14.0)),
            )
            .show(ctx, |ui| {
                let changed = sidebar::show(
                    ui,
                    &mut self.page,
                    &self.db,
                    self.cache.stats.expired,
                    self.cache.stats.expiring,
                    &user,
                    &mut logout,
                );
                if changed {
                    anim::bump();
                }
            });
        egui::CentralPanel::default()
            .frame(Frame::none().fill(p.bg).inner_margin(Margin::same(24.0)))
            .show(ctx, |ui| {
                let t = anim::progress(ctx, ("page", self.page.code()), 0.22);
                ui.set_opacity(0.35 + 0.65 * t);
                let mut cx = Ctx {
                    db: &self.db,
                    user: &user,
                    cache: &self.cache,
                    toasts: &mut self.toasts,
                    confirm: &mut self.confirm,
                    dirty: &mut self.dirty,
                    nav: &mut self.nav,
                    images: &mut self.images,
                };
                pages::show(self.page, ui, &mut cx, &mut self.pages);
            });
        if let Some(c) = self.confirm.clone() {
            match dialogs::confirm(ctx, &c) {
                Some(true) => {
                    self.confirm = None;
                    self.run(ctx, c);
                }
                Some(false) => self.confirm = None,
                None => {}
            }
        }
        self.toasts.show(ctx);
        if self.dirty {
            ctx.request_repaint();
        }
        if logout {
            self.logout();
        }
    }
}
