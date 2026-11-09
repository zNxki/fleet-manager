use super::theme::pal;
use crate::i18n::tr;
use eframe::egui::{
    self, vec2, Align2, Color32, Context, Frame, Id, Margin, Order, RichText, Rounding, Stroke,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Success,
    Error,
}

struct Item {
    kind: Kind,
    text: String,
    born: Option<f64>,
}

#[derive(Default)]
pub struct Toasts {
    items: Vec<Item>,
}

const LIFE: f64 = 4.6;

impl Toasts {
    fn push(&mut self, kind: Kind, msg: &str) {
        if self.items.len() >= 5 {
            self.items.remove(0);
        }
        self.items.push(Item {
            kind,
            text: tr(msg),
            born: None,
        });
    }

    pub fn success(&mut self, msg: &str) {
        self.push(Kind::Success, msg);
    }

    pub fn error(&mut self, msg: &str) {
        self.push(Kind::Error, msg);
    }

    pub fn show(&mut self, ctx: &Context) {
        if self.items.is_empty() {
            return;
        }
        let p = pal();
        let now = ctx.input(|i| i.time);
        for it in self.items.iter_mut() {
            if it.born.is_none() {
                it.born = Some(now);
            }
        }
        self.items.retain(|i| now - i.born.unwrap_or(now) < LIFE);
        let mut y = 22.0;
        for (n, it) in self.items.iter().enumerate().rev() {
            let age = now - it.born.unwrap_or(now);
            let slide = (1.0 - (age / 0.28).min(1.0)) as f32;
            let fade = if age > LIFE - 0.4 {
                ((LIFE - age) / 0.4).max(0.0) as f32
            } else {
                1.0
            };
            let color = match it.kind {
                Kind::Success => p.success,
                Kind::Error => p.danger,
            };
            let a = |c: Color32, v: u8| {
                Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (v as f32 * fade) as u8)
            };
            let resp = egui::Area::new(Id::new(("toast", n)))
                .order(Order::Tooltip)
                .anchor(Align2::RIGHT_BOTTOM, vec2(-22.0 + slide * 60.0, -y))
                .show(ctx, |ui| {
                    Frame::none()
                        .fill(a(p.card, 250))
                        .stroke(Stroke::new(1.0, a(color, 200)))
                        .rounding(Rounding::same(10.0))
                        .inner_margin(Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.set_max_width(380.0);
                            ui.horizontal(|ui| {
                                let (r, _) =
                                    ui.allocate_exact_size(vec2(6.0, 22.0), egui::Sense::hover());
                                ui.painter()
                                    .rect_filled(r, Rounding::same(3.0), a(color, 255));
                                ui.label(RichText::new(&it.text).color(a(p.text, 255)));
                            });
                        });
                });
            y += resp.response.rect.height() + 10.0;
        }
        ctx.request_repaint();
    }
}
