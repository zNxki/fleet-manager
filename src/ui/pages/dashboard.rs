use super::{Ctx, Page};
use crate::i18n::tr;
use crate::ui::charts;
use crate::ui::icons::Icon;
use crate::ui::theme::{level_color, pal, status_color, tint};
use crate::ui::widgets::{self, card_wh, metric, muted, pair, section_title, stat_card};
use crate::util::format as f;
use eframe::egui::{self, vec2, Align, Color32, Layout, RichText, ScrollArea, Sense, Ui};

pub fn show(ui: &mut Ui, cx: &mut Ctx) {
    let cache = cx.cache;
    let s = &cache.stats;
    let ctx = ui.ctx().clone();
    let p = pal();
    widgets::page_header(ui, "Dashboard", "Overview of your fleet", |ui| {
        muted(ui, &f::date(&f::today_iso()));
    });
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let avail = ui.available_width() - 4.0;
            let gap = 16.0;
            let per = if avail >= 1180.0 {
                6
            } else if avail >= 760.0 {
                3
            } else {
                2
            };
            let w = ((avail - gap * (per as f32 - 1.0)) / per as f32).floor();
            let cnt = |k: &str, v: usize| -> String {
                format!("{}", widgets::count_up(&ctx, k, v as f64).round() as i64)
            };
            let alert_color = if s.expired > 0 {
                p.danger
            } else if s.expiring > 0 {
                p.warn
            } else {
                p.success
            };
            let active_vehicles = s.vehicles.saturating_sub(s.retired);
            let items: Vec<(Icon, &str, String, String, Color32)> = vec![
                (
                    Icon::Vehicles,
                    "Total vehicles",
                    cnt("d_veh", s.vehicles),
                    format!("{} {}", active_vehicles, tr("active")),
                    p.accent,
                ),
                (
                    Icon::Drivers,
                    "Total drivers",
                    cnt("d_drv", s.drivers),
                    format!("{} {}", s.active_drivers, tr("active")),
                    Color32::from_rgb(20, 184, 166),
                ),
                (
                    Icon::Vehicles,
                    "Available",
                    cnt("d_av", s.available),
                    tr("Ready for assignment"),
                    p.success,
                ),
                (
                    Icon::Assignments,
                    "Assigned",
                    cnt("d_as", s.assigned),
                    tr("Currently in use"),
                    p.purple,
                ),
                (
                    Icon::Maintenance,
                    "In maintenance",
                    cnt("d_mn", s.maintenance),
                    tr("Out of service"),
                    p.warn,
                ),
                (
                    Icon::Alert,
                    "Expired deadlines",
                    cnt("d_ex", s.expired),
                    format!("{} {}", s.expiring, tr("expiring soon")),
                    alert_color,
                ),
            ];
            for chunk in items.chunks(per) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for (icon, label, value, sub, color) in chunk {
                        stat_card(ui, w, *icon, label, value.clone(), sub, *color);
                    }
                });
                ui.add_space(gap);
            }

            pair(
                ui,
                0.62,
                |ui, w| {
                    card_wh(ui, w, 340.0, |ui| {
                        section_title(ui, "Monthly expenses");
                        ui.add_space(2.0);
                        let total = widgets::count_up(&ctx, "d_month", s.month_total);
                        ui.label(RichText::new(f::money(total)).size(28.0).strong());
                        muted(ui, &tr("Spent this month"));
                        ui.add_space(4.0);
                        charts::legend(
                            ui,
                            &[
                                (
                                    p.accent,
                                    format!("{} {}", tr("Fuel"), f::money(s.month_fuel)),
                                ),
                                (
                                    p.warn,
                                    format!("{} {}", tr("Maintenance"), f::money(s.month_maint)),
                                ),
                                (
                                    p.purple,
                                    format!("{} {}", tr("Other expenses"), f::money(s.month_other)),
                                ),
                            ],
                        );
                        ui.add_space(6.0);
                        charts::stacked_bars(ui, "d_monthly", &s.monthly, 170.0, &|v| f::money(v));
                    });
                },
                |ui, w| {
                    card_wh(ui, w, 340.0, |ui| {
                        section_title(ui, "Fleet status");
                        ui.add_space(6.0);
                        let rows = [
                            ("Available", s.available),
                            ("Assigned", s.assigned),
                            ("Maintenance", s.maintenance),
                            ("Retired", s.retired),
                        ];
                        let segs: Vec<(String, f64, Color32)> = rows
                            .iter()
                            .map(|(n, v)| (tr(n), *v as f64, status_color(n)))
                            .collect();
                        ui.vertical_centered(|ui| {
                            charts::donut(
                                ui,
                                "d_status",
                                158.0,
                                &segs,
                                &s.vehicles.to_string(),
                                &tr("vehicles"),
                            );
                        });
                        ui.add_space(8.0);
                        for (n, v) in rows.iter() {
                            ui.horizontal(|ui| {
                                let (r, _) =
                                    ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                                ui.painter().rect_filled(
                                    r,
                                    egui::Rounding::same(3.0),
                                    status_color(n),
                                );
                                ui.label(tr(n));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(v.to_string()).strong());
                                });
                            });
                        }
                    });
                },
            );
            ui.add_space(gap);

            pair(
                ui,
                0.5,
                |ui, w| {
                    card_wh(ui, w, 330.0, |ui| {
                        section_title(ui, "Fuel consumption");
                        ui.add_space(6.0);
                        let mw = ((w - 38.0) / 2.0).floor();
                        ui.horizontal(|ui| {
                            metric(ui, "Fleet average", &fmt_l100(s.fleet_l100), mw);
                            metric(
                                ui,
                                "Average price per liter",
                                &money_or_dash(s.avg_price),
                                mw,
                            );
                        });
                        ui.horizontal(|ui| {
                            metric(
                                ui,
                                "Liters this month",
                                &format!("{} L", f::num(s.month_liters, 1)),
                                mw,
                            );
                            metric(ui, "Cost per km", &money_or_dash(s.fleet_cost_km), mw);
                        });
                        ui.add_space(6.0);
                        let mut list: Vec<(String, f64)> = s.by_vehicle.clone();
                        list.sort_by(|a, b| {
                            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        list.truncate(5);
                        if list.is_empty() {
                            muted(
                                ui,
                                &tr("Record at least two full-tank refuelings to see consumption."),
                            );
                        } else {
                            charts::hbars(ui, "d_fuel", &list, "L/100km", p.accent);
                        }
                    });
                },
                |ui, w| {
                    card_wh(ui, w, 330.0, |ui| {
                        ui.horizontal(|ui| {
                            section_title(ui, "Upcoming deadlines");
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if widgets::small(ui, "View all").clicked() {
                                    cx.go(Page::Documents);
                                }
                            });
                        });
                        ui.add_space(4.0);
                        let list: Vec<_> = cache
                            .deadlines
                            .iter()
                            .filter(|d| d.level > 0 || d.days <= 90)
                            .take(6)
                            .collect();
                        if list.is_empty() {
                            muted(ui, &tr("No upcoming deadlines"));
                        }
                        for d in list {
                            ui.horizontal(|ui| {
                                let c = level_color(d.level);
                                let (r, _) =
                                    ui.allocate_exact_size(vec2(8.0, 30.0), Sense::hover());
                                ui.painter().rect_filled(r, egui::Rounding::same(4.0), c);
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    ui.label(RichText::new(widgets::cut(&d.subject, 26)).strong());
                                    ui.label(
                                        RichText::new(widgets::cut(&tr(&d.label), 34))
                                            .size(12.0)
                                            .color(p.muted),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let t = if d.days == i64::MAX {
                                        d.detail.clone()
                                    } else {
                                        widgets::due_label(d.days)
                                    };
                                    ui.label(RichText::new(t).color(c).size(12.5).strong());
                                });
                            });
                        }
                    });
                },
            );
            ui.add_space(gap);

            card_wh(ui, avail, 120.0, |ui| {
                section_title(ui, "Recent activity");
                ui.add_space(4.0);
                if cache.activities.is_empty() {
                    muted(ui, &tr("No activity recorded yet"));
                }
                for a in cache.activities.iter().take(8) {
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                        ui.painter()
                            .circle_filled(r.center(), 4.0, tint(p.accent, 200));
                        let when = if a.ts.len() >= 16 {
                            format!("{} {}", f::date(&a.ts[..10]), &a.ts[11..16])
                        } else {
                            a.ts.clone()
                        };
                        ui.label(RichText::new(when).color(p.muted).size(12.5));
                        ui.label(tr(&a.message));
                    });
                }
            });
            ui.add_space(10.0);
        });
}

fn fmt_l100(v: f64) -> String {
    if v > 0.0 {
        format!("{} L/100 km", f::num(v, 2))
    } else {
        "-".to_string()
    }
}

fn money_or_dash(v: f64) -> String {
    if v > 0.0 {
        f::money(v)
    } else {
        "-".to_string()
    }
}
