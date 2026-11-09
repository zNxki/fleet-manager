use eframe::egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Rounding, Shape, Stroke};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Dashboard,
    Vehicles,
    Drivers,
    Assignments,
    Maintenance,
    Fuel,
    Documents,
    Reports,
    Settings,
    Alert,
    Sun,
    Moon,
    Logout,
}

pub fn draw(p: &Painter, icon: Icon, c: Pos2, s: f32, col: Color32) {
    let w = (s * 0.09).max(1.4);
    let st = Stroke::new(w, col);
    match icon {
        Icon::Logout => {
            p.add(Shape::line(
                vec![
                    pos2(c.x - s * 0.05, c.y - s * 0.38),
                    pos2(c.x - s * 0.42, c.y - s * 0.38),
                    pos2(c.x - s * 0.42, c.y + s * 0.38),
                    pos2(c.x - s * 0.05, c.y + s * 0.38),
                ],
                st,
            ));
            p.line_segment([pos2(c.x - s * 0.15, c.y), pos2(c.x + s * 0.4, c.y)], st);
            p.line_segment(
                [pos2(c.x + s * 0.4, c.y), pos2(c.x + s * 0.2, c.y - s * 0.18)],
                st,
            );
            p.line_segment(
                [pos2(c.x + s * 0.4, c.y), pos2(c.x + s * 0.2, c.y + s * 0.18)],
                st,
            );
        }
        Icon::Dashboard => {
            let k = s * 0.44;
            let g = s * 0.12;
            for (dx, dy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let cc = pos2(
                    c.x + dx * (k / 2.0 + g / 2.0),
                    c.y + dy * (k / 2.0 + g / 2.0),
                );
                p.rect_filled(
                    Rect::from_center_size(cc, vec2(k, k)),
                    Rounding::same(s * 0.1),
                    col,
                );
            }
        }
        Icon::Vehicles => {
            p.rect_filled(
                Rect::from_center_size(pos2(c.x, c.y + s * 0.1), vec2(s * 0.92, s * 0.32)),
                Rounding::same(s * 0.1),
                col,
            );
            p.add(Shape::convex_polygon(
                vec![
                    pos2(c.x - s * 0.3, c.y - s * 0.04),
                    pos2(c.x - s * 0.17, c.y - s * 0.32),
                    pos2(c.x + s * 0.17, c.y - s * 0.32),
                    pos2(c.x + s * 0.3, c.y - s * 0.04),
                ],
                col,
                Stroke::NONE,
            ));
            p.circle_filled(pos2(c.x - s * 0.28, c.y + s * 0.3), s * 0.13, col);
            p.circle_filled(pos2(c.x + s * 0.28, c.y + s * 0.3), s * 0.13, col);
        }
        Icon::Drivers => {
            p.circle_filled(pos2(c.x, c.y - s * 0.17), s * 0.2, col);
            p.rect_filled(
                Rect::from_center_size(pos2(c.x, c.y + s * 0.28), vec2(s * 0.74, s * 0.34)),
                Rounding {
                    nw: s * 0.17,
                    ne: s * 0.17,
                    sw: s * 0.04,
                    se: s * 0.04,
                },
                col,
            );
        }
        Icon::Assignments => {
            let y1 = c.y - s * 0.17;
            let y2 = c.y + s * 0.2;
            p.line_segment([pos2(c.x - s * 0.42, y1), pos2(c.x + s * 0.3, y1)], st);
            p.add(Shape::convex_polygon(
                vec![
                    pos2(c.x + s * 0.48, y1),
                    pos2(c.x + s * 0.22, y1 - s * 0.17),
                    pos2(c.x + s * 0.22, y1 + s * 0.17),
                ],
                col,
                Stroke::NONE,
            ));
            p.line_segment([pos2(c.x + s * 0.42, y2), pos2(c.x - s * 0.3, y2)], st);
            p.add(Shape::convex_polygon(
                vec![
                    pos2(c.x - s * 0.48, y2),
                    pos2(c.x - s * 0.22, y2 - s * 0.17),
                    pos2(c.x - s * 0.22, y2 + s * 0.17),
                ],
                col,
                Stroke::NONE,
            ));
        }
        Icon::Maintenance => {
            p.line_segment(
                [
                    pos2(c.x - s * 0.38, c.y + s * 0.38),
                    pos2(c.x + s * 0.1, c.y - s * 0.1),
                ],
                Stroke::new(s * 0.16, col),
            );
            p.circle_stroke(
                pos2(c.x + s * 0.2, c.y - s * 0.2),
                s * 0.22,
                Stroke::new(s * 0.14, col),
            );
        }
        Icon::Fuel => {
            p.circle_filled(pos2(c.x, c.y + s * 0.14), s * 0.3, col);
            p.add(Shape::convex_polygon(
                vec![
                    pos2(c.x - s * 0.28, c.y + s * 0.04),
                    pos2(c.x, c.y - s * 0.46),
                    pos2(c.x + s * 0.28, c.y + s * 0.04),
                ],
                col,
                Stroke::NONE,
            ));
        }
        Icon::Documents => {
            let r = Rect::from_center_size(c, vec2(s * 0.7, s * 0.92));
            p.rect_stroke(r, Rounding::same(s * 0.1), Stroke::new(w * 1.2, col));
            for i in 0..3 {
                let y = r.top() + s * (0.3 + 0.2 * i as f32);
                p.line_segment(
                    [pos2(r.left() + s * 0.14, y), pos2(r.right() - s * 0.14, y)],
                    st,
                );
            }
        }
        Icon::Reports => {
            let base = c.y + s * 0.42;
            for (dx, h) in [(-0.3f32, 0.45f32), (0.0, 0.85), (0.3, 0.62)] {
                let r = Rect::from_min_max(
                    pos2(c.x + s * dx - s * 0.11, base - s * h),
                    pos2(c.x + s * dx + s * 0.11, base),
                );
                p.rect_filled(r, Rounding::same(s * 0.04), col);
            }
        }
        Icon::Settings => {
            p.circle_stroke(c, s * 0.2, Stroke::new(w * 1.5, col));
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::FRAC_PI_4;
                let d = vec2(a.cos(), a.sin());
                p.line_segment(
                    [c + d * (s * 0.32), c + d * (s * 0.46)],
                    Stroke::new(w * 1.6, col),
                );
            }
        }
        Icon::Alert => {
            p.add(Shape::closed_line(
                vec![
                    pos2(c.x, c.y - s * 0.42),
                    pos2(c.x + s * 0.46, c.y + s * 0.38),
                    pos2(c.x - s * 0.46, c.y + s * 0.38),
                ],
                st,
            ));
            p.line_segment([pos2(c.x, c.y - s * 0.12), pos2(c.x, c.y + s * 0.12)], st);
            p.circle_filled(pos2(c.x, c.y + s * 0.26), w * 0.7, col);
        }
        Icon::Sun => {
            p.circle_filled(c, s * 0.22, col);
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::FRAC_PI_4;
                let d = vec2(a.cos(), a.sin());
                p.line_segment([c + d * (s * 0.34), c + d * (s * 0.46)], st);
            }
        }
        Icon::Moon => {
            p.circle_filled(c, s * 0.36, col);
            p.circle_filled(
                pos2(c.x + s * 0.2, c.y - s * 0.12),
                s * 0.32,
                crate::ui::theme::pal().sidebar,
            );
        }
    }
}
