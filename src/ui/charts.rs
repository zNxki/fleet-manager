use super::anim;
use super::theme::{pal, tint};
use crate::i18n::tr;
use eframe::egui::{
    pos2, vec2, Align2, Color32, FontId, Id, Rect, RichText, Rounding, Sense, Shape, Stroke, Ui,
};
use std::f32::consts::{FRAC_PI_2, TAU};

pub fn palette(i: usize) -> Color32 {
    let p = pal();
    let list = [
        p.accent,
        p.success,
        p.warn,
        p.purple,
        p.danger,
        Color32::from_rgb(20, 184, 166),
        Color32::from_rgb(236, 72, 153),
    ];
    list[i % list.len()]
}

pub fn compact(v: f64) -> String {
    if v >= 1_000_000.0 {
        format!("{:.1}M", v / 1_000_000.0)
    } else if v >= 10_000.0 {
        format!("{:.0}k", v / 1_000.0)
    } else if v >= 1_000.0 {
        format!("{:.1}k", v / 1_000.0)
    } else if v >= 10.0 {
        format!("{:.0}", v)
    } else {
        format!("{:.1}", v)
    }
}

fn nice_top(max: f64) -> f64 {
    if max <= 0.0 {
        return 4.0;
    }
    let raw = max / 4.0;
    let mag = 10f64.powf(raw.log10().floor());
    let n = raw / mag;
    let step = if n <= 1.0 {
        1.0
    } else if n <= 2.0 {
        2.0
    } else if n <= 2.5 {
        2.5
    } else if n <= 5.0 {
        5.0
    } else {
        10.0
    };
    step * mag * 4.0
}

pub fn legend(ui: &mut Ui, items: &[(Color32, String)]) {
    let p = pal();
    ui.horizontal_wrapped(|ui| {
        for (c, t) in items {
            let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
            ui.painter().rect_filled(r, Rounding::same(3.0), *c);
            ui.label(RichText::new(t.as_str()).color(p.muted).size(12.5));
            ui.add_space(8.0);
        }
    });
}

pub fn stacked_bars(
    ui: &mut Ui,
    id: &str,
    data: &[(String, f64, f64, f64)],
    height: f32,
    money: &dyn Fn(f64) -> String,
) {
    let p = pal();
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, height), Sense::hover());
    let painter = ui.painter_at(rect);
    let plot = Rect::from_min_max(
        pos2(rect.left() + 48.0, rect.top() + 8.0),
        pos2(rect.right() - 6.0, rect.bottom() - 26.0),
    );
    let max = data.iter().map(|d| d.1 + d.2 + d.3).fold(0.0, f64::max);
    let top = nice_top(max);
    for k in 0..=4 {
        let y = plot.bottom() - plot.height() * k as f32 / 4.0;
        painter.line_segment(
            [pos2(plot.left(), y), pos2(plot.right(), y)],
            Stroke::new(1.0, tint(p.muted, if k == 0 { 90 } else { 38 })),
        );
        painter.text(
            pos2(plot.left() - 8.0, y),
            Align2::RIGHT_CENTER,
            compact(top * k as f64 / 4.0),
            FontId::proportional(11.5),
            p.muted,
        );
    }
    let n = data.len().max(1);
    let slot = plot.width() / n as f32;
    let bw = (slot * 0.48).min(56.0);
    let t = anim::progress(ui.ctx(), (id, "bars"), 0.85);
    for (i, d) in data.iter().enumerate() {
        let cx = plot.left() + slot * (i as f32 + 0.5);
        let mut y = plot.bottom();
        for (val, col) in [(d.1, p.accent), (d.2, p.warn), (d.3, p.purple)] {
            let h = (val / top) as f32 * plot.height() * t;
            if h > 0.5 {
                let r = Rect::from_min_max(pos2(cx - bw / 2.0, y - h), pos2(cx + bw / 2.0, y));
                painter.rect_filled(r, Rounding::same(2.0), col);
                y -= h;
            }
        }
        painter.text(
            pos2(cx, plot.bottom() + 7.0),
            Align2::CENTER_TOP,
            tr(&d.0),
            FontId::proportional(12.0),
            p.muted,
        );
        let hr = Rect::from_min_max(
            pos2(cx - slot / 2.0, plot.top()),
            pos2(cx + slot / 2.0, rect.bottom()),
        );
        let resp = ui.interact(hr, Id::new((id, "hover", i)), Sense::hover());
        let _ = resp.on_hover_text(format!(
            "{}\n{}: {}\n{}: {}\n{}: {}\n{}: {}",
            tr(&d.0),
            tr("Fuel"),
            money(d.1),
            tr("Maintenance"),
            money(d.2),
            tr("Other expenses"),
            money(d.3),
            tr("Total"),
            money(d.1 + d.2 + d.3)
        ));
    }
}

pub fn donut(
    ui: &mut Ui,
    id: &str,
    size: f32,
    segs: &[(String, f64, Color32)],
    big: &str,
    small: &str,
) {
    let p = pal();
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let painter = ui.painter_at(rect);
    let c = rect.center();
    let th = size * 0.14;
    let r_mid = size / 2.0 - th / 2.0 - 3.0;
    painter.circle_stroke(c, r_mid, Stroke::new(th, tint(p.muted, 34)));
    let total: f64 = segs.iter().map(|s| s.1).sum();
    let t = anim::progress(ui.ctx(), (id, "donut"), 0.9);
    if total > 0.0 {
        let mut cum = 0.0f32;
        for (_, v, col) in segs {
            let f0 = cum;
            let f1 = cum + (*v / total) as f32;
            cum = f1;
            let s0 = f0.min(t);
            let s1 = f1.min(t);
            if s1 - s0 < 0.0004 {
                continue;
            }
            let gap = if segs.len() > 1 && (f1 - f0) > 0.04 {
                0.006
            } else {
                0.0
            };
            let a0 = -FRAC_PI_2 + (s0 + gap) * TAU;
            let a1 = -FRAC_PI_2 + (s1 - gap) * TAU;
            if a1 <= a0 {
                continue;
            }
            let steps = (((a1 - a0) / 0.04) as usize).max(2);
            let pts: Vec<_> = (0..=steps)
                .map(|k| {
                    let a = a0 + (a1 - a0) * k as f32 / steps as f32;
                    pos2(c.x + r_mid * a.cos(), c.y + r_mid * a.sin())
                })
                .collect();
            painter.add(Shape::line(pts, Stroke::new(th, *col)));
        }
        if let Some(pos) = resp.hover_pos() {
            let d = pos - c;
            let dist = d.length();
            if dist >= r_mid - th / 2.0 - 2.0 && dist <= r_mid + th / 2.0 + 2.0 {
                let mut ang = d.y.atan2(d.x) + FRAC_PI_2;
                if ang < 0.0 {
                    ang += TAU;
                }
                let frac = ang / TAU;
                let mut acc = 0.0f32;
                for (name, v, _) in segs {
                    let share = (*v / total) as f32;
                    if frac >= acc && frac < acc + share {
                        let _ =
                            resp.clone()
                                .on_hover_text(format!("{}: {:.1}%", name, share * 100.0));
                        break;
                    }
                    acc += share;
                }
            }
        }
    }
    painter.text(
        pos2(c.x, c.y - size * 0.045),
        Align2::CENTER_CENTER,
        big,
        FontId::proportional(size * 0.19),
        p.text,
    );
    painter.text(
        pos2(c.x, c.y + size * 0.12),
        Align2::CENTER_CENTER,
        small,
        FontId::proportional(size * 0.07 + 4.0),
        p.muted,
    );
}

pub fn hbars(ui: &mut Ui, id: &str, items: &[(String, f64)], unit: &str, color: Color32) {
    let p = pal();
    let max = items.iter().map(|i| i.1).fold(0.0, f64::max).max(0.0001);
    let t = anim::progress(ui.ctx(), (id, "hbars"), 0.8);
    for (name, v) in items {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
        let painter = ui.painter_at(rect);
        let cy = rect.center().y;
        painter.text(
            pos2(rect.left(), cy),
            Align2::LEFT_CENTER,
            super::widgets::cut(name, 12),
            FontId::proportional(13.0),
            p.text,
        );
        let lx = rect.left() + 100.0;
        let rx = (rect.right() - 92.0).max(lx + 20.0);
        let track = Rect::from_min_max(pos2(lx, cy - 5.0), pos2(rx, cy + 5.0));
        painter.rect_filled(track, Rounding::same(5.0), tint(p.muted, 34));
        let bw = (rx - lx) * (*v / max) as f32 * t;
        if bw > 1.0 {
            painter.rect_filled(
                Rect::from_min_max(pos2(lx, cy - 5.0), pos2(lx + bw, cy + 5.0)),
                Rounding::same(5.0),
                color,
            );
        }
        painter.text(
            pos2(rect.right(), cy),
            Align2::RIGHT_CENTER,
            format!("{:.1} {}", v, unit),
            FontId::proportional(12.5),
            p.muted,
        );
    }
}
