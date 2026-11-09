use super::theme::{pal, tint};
use super::widgets;
use crate::i18n::tr;
use eframe::egui::{
    self, vec2, Align2, Color32, Context, Frame, Id, Margin, Order, Pos2, RichText, Rounding,
    Sense, Stroke, Ui,
};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Action {
    DeleteVehicle(i64),
    DeleteDriver(i64),
    DeleteAssignment(i64),
    DeleteMaintenance(i64),
    DeleteFuel(i64),
    DeleteExpense(i64),
    DeleteDocument(i64),
    DeleteUser(i64),
    Restore(PathBuf),
}

#[derive(Clone, Debug)]
pub struct Confirm {
    pub title: String,
    pub message: String,
    pub button: String,
    pub action: Action,
    pub log: String,
}

pub fn modal(ctx: &Context, id: &str, title: &str, width: f32, add: impl FnOnce(&mut Ui)) -> bool {
    let p = pal();
    let screen = ctx.screen_rect();
    egui::Area::new(Id::new(("dim", id)))
        .order(Order::Middle)
        .fixed_pos(Pos2::ZERO)
        .show(ctx, |ui| {
            ui.allocate_rect(screen, Sense::click_and_drag());
            ui.painter()
                .rect_filled(screen, Rounding::ZERO, Color32::from_black_alpha(165));
        });
    let t = super::anim::progress(ctx, ("modal", id), 0.18);
    let mut close = false;
    let max_h = (screen.height() * 0.84).max(300.0);
    egui::Window::new(title)
        .id(Id::new(("modal", id)))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .order(Order::Foreground)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, (1.0 - t) * 18.0))
        .frame(
            Frame::none()
                .fill(p.card)
                .stroke(Stroke::new(1.0, p.border))
                .rounding(Rounding::same(14.0))
                .inner_margin(Margin::same(22.0)),
        )
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.horizontal(|ui| {
                ui.label(RichText::new(tr(title)).size(19.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("x").size(15.0).color(p.muted))
                                .frame(false),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        close = true;
                    }
                });
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .max_height(max_h - 120.0)
                .auto_shrink([true, true])
                .show(ui, |ui| {
                    ui.set_width(width);
                    add(ui);
                });
        });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close = true;
    }
    close
}

pub fn footer(ui: &mut Ui, save_label: &str, on_cancel: &mut bool, on_save: &mut bool) {
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(6.0);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if widgets::primary(ui, save_label).clicked() {
            *on_save = true;
        }
        if widgets::secondary(ui, "Cancel").clicked() {
            *on_cancel = true;
        }
    });
}

pub fn confirm(ctx: &Context, c: &Confirm) -> Option<bool> {
    let p = pal();
    let mut result: Option<bool> = None;
    let close = modal(ctx, "confirm", &c.title, 440.0, |ui| {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
            ui.painter()
                .circle_filled(r.center(), 22.0, tint(p.danger, 36));
            super::icons::draw(
                ui.painter(),
                super::icons::Icon::Alert,
                r.center(),
                22.0,
                p.danger,
            );
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.set_max_width(340.0);
                ui.label(RichText::new(tr(&c.message)).size(14.5));
                ui.add_space(4.0);
                ui.label(
                    RichText::new(tr("This action cannot be undone."))
                        .color(p.muted)
                        .size(13.0),
                );
            });
        });
        ui.add_space(14.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::danger(ui, &c.button).clicked() {
                result = Some(true);
            }
            if widgets::secondary(ui, "Cancel").clicked() {
                result = Some(false);
            }
        });
    });
    if close {
        return Some(false);
    }
    result
}

pub fn info(ctx: &Context, id: &str, title: &str, width: f32, add: impl FnOnce(&mut Ui)) -> bool {
    let mut closed = false;
    let x = modal(ctx, id, title, width, |ui| {
        add(ui);
        ui.add_space(12.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::secondary(ui, "Close").clicked() {
                closed = true;
            }
        });
    });
    x || closed
}
