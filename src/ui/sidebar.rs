use super::icons::{self, Icon};
use super::pages::Page;
use super::theme::{self, pal, tint};
use crate::db::users::User;
use crate::db::Db;
use crate::i18n::tr;
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CursorIcon, FontId, Layout, RichText, Rounding,
    Sense, Ui,
};

fn item(
    ui: &mut Ui,
    icon: Icon,
    label: &str,
    active: bool,
    badge: Option<(usize, Color32)>,
) -> bool {
    let p = pal();
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
    let h = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("hover"), resp.hovered(), 0.12);
    let a = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("active"), active, 0.18);
    let painter = ui.painter();
    if a > 0.0 {
        painter.rect_filled(rect, Rounding::same(9.0), tint(p.accent, (46.0 * a) as u8));
        let bar = egui::Rect::from_min_max(
            pos2(rect.left(), rect.top() + 9.0),
            pos2(rect.left() + 3.0, rect.bottom() - 9.0),
        );
        painter.rect_filled(bar, Rounding::same(2.0), tint(p.accent, (255.0 * a) as u8));
    } else if h > 0.0 {
        painter.rect_filled(rect, Rounding::same(9.0), tint(p.text, (16.0 * h) as u8));
    }
    let col = if active {
        p.accent
    } else {
        p.muted.lerp_to_gamma(p.text, h)
    };
    icons::draw(
        painter,
        icon,
        pos2(rect.left() + 26.0, rect.center().y),
        20.0,
        col,
    );
    painter.text(
        pos2(rect.left() + 48.0, rect.center().y),
        Align2::LEFT_CENTER,
        tr(label),
        FontId::proportional(14.5),
        if active { p.text } else { col },
    );
    if let Some((n, c)) = badge {
        if n > 0 {
            let r = egui::Rect::from_center_size(
                pos2(rect.right() - 20.0, rect.center().y),
                vec2(24.0, 18.0),
            );
            painter.rect_filled(r, Rounding::same(9.0), c);
            painter.text(
                r.center(),
                Align2::CENTER_CENTER,
                n.to_string(),
                FontId::proportional(11.5),
                Color32::WHITE,
            );
        }
    }
    resp.on_hover_cursor(CursorIcon::PointingHand).clicked()
}

pub fn show(
    ui: &mut Ui,
    page: &mut Page,
    db: &Db,
    expired: usize,
    expiring: usize,
    user: &User,
    logout: &mut bool,
) -> bool {
    let p = pal();
    let mut changed = false;
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::hover());
        ui.painter().rect_filled(r, Rounding::same(11.0), p.accent);
        icons::draw(
            ui.painter(),
            Icon::Vehicles,
            r.center(),
            24.0,
            Color32::WHITE,
        );
        ui.add_space(2.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(RichText::new("FleetManager").size(18.0).strong());
            ui.label(
                RichText::new(tr("Fleet management"))
                    .size(11.5)
                    .color(p.muted),
            );
        });
    });
    ui.add_space(26.0);
    ui.label(RichText::new(tr("MENU")).size(11.0).strong().color(p.muted));
    ui.add_space(4.0);
    ui.spacing_mut().item_spacing.y = 4.0;
    for pg in Page::ALL {
        let badge = if pg == Page::Documents {
            if expired > 0 {
                Some((expired, p.danger))
            } else {
                Some((expiring, p.warn))
            }
        } else {
            None
        };
        if item(ui, pg.icon(), pg.label(), *page == pg, badge) && *page != pg {
            *page = pg;
            changed = true;
        }
    }
    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                .size(11.5)
                .color(p.muted),
        );
        let dark = theme::is_dark();
        let (label, icon) = if dark {
            ("Light mode", Icon::Sun)
        } else {
            ("Dark mode", Icon::Moon)
        };
        if item(ui, icon, label, false, None) {
            theme::apply(ui.ctx(), !dark);
            db.set_setting("theme", if dark { "light" } else { "dark" });
        }
        if item(ui, Icon::Logout, "Log out", false, None) {
            *logout = true;
        }
        let role = if user.is_admin { "Administrator" } else { "User" };
        ui.label(
            RichText::new(format!("{} - {}", user.username, tr(role)))
                .size(12.0)
                .color(p.muted),
        );
        ui.add_space(4.0);
    });
    changed
}
