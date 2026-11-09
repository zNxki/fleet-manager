use super::icons::{self, Icon};
use super::theme::{level_color, pal, status_color, tint};
use crate::i18n::tr;
use crate::util::format as f;
use eframe::egui::{
    self, vec2, Align, Color32, CursorIcon, Frame, Layout, Margin, Response, RichText, Rounding,
    Sense, Stroke, TextEdit, Ui,
};
use egui_extras::{Column, TableBuilder};

#[derive(Clone, Copy, Debug)]
pub struct Sort {
    pub col: usize,
    pub asc: bool,
}

impl Default for Sort {
    fn default() -> Self {
        Sort { col: 0, asc: true }
    }
}

#[derive(Clone)]
pub struct Col {
    pub title: String,
    pub width: f32,
    pub sort: bool,
}

impl Col {
    pub fn new(title: &str, width: f32) -> Col {
        Col {
            title: title.to_string(),
            width,
            sort: false,
        }
    }

    pub fn sortable(mut self) -> Col {
        self.sort = true;
        self
    }
}

pub fn cut(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut o: String = s.chars().take(max.saturating_sub(1)).collect();
        o.push('…');
        o
    }
}

pub fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    let p = pal();
    Frame::none()
        .fill(p.card)
        .stroke(Stroke::new(1.0, p.border))
        .rounding(Rounding::same(12.0))
        .inner_margin(Margin::same(18.0))
        .show(ui, add)
        .inner
}

pub fn card_w<R>(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    card(ui, |ui| {
        ui.set_width((width - 38.0).max(10.0));
        add(ui)
    })
}

pub fn card_wh<R>(ui: &mut Ui, width: f32, height: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    card(ui, |ui| {
        ui.set_width((width - 38.0).max(10.0));
        ui.set_min_height((height - 38.0).max(10.0));
        add(ui)
    })
}

pub fn section_title(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(tr(text)).size(16.0).strong());
}

pub fn muted(ui: &mut Ui, text: &str) -> Response {
    ui.label(RichText::new(text).color(pal().muted).size(13.0))
}

pub fn page_header(ui: &mut Ui, title: &str, subtitle: &str, actions: impl FnOnce(&mut Ui)) {
    let p = pal();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(tr(title)).size(26.0).strong());
            ui.label(RichText::new(tr(subtitle)).color(p.muted));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| actions(ui));
    });
    ui.add_space(14.0);
}

pub fn primary(ui: &mut Ui, text: &str) -> Response {
    let p = pal();
    ui.add(
        egui::Button::new(RichText::new(tr(text)).color(Color32::WHITE).strong())
            .fill(p.accent)
            .rounding(Rounding::same(8.0))
            .min_size(vec2(0.0, 32.0)),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}

pub fn secondary(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Button::new(RichText::new(tr(text)))
            .rounding(Rounding::same(8.0))
            .min_size(vec2(0.0, 32.0)),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}

pub fn danger(ui: &mut Ui, text: &str) -> Response {
    let p = pal();
    ui.add(
        egui::Button::new(RichText::new(tr(text)).color(Color32::WHITE).strong())
            .fill(p.danger)
            .rounding(Rounding::same(8.0))
            .min_size(vec2(0.0, 32.0)),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}

pub fn small(ui: &mut Ui, text: &str) -> Response {
    ui.add(egui::Button::new(RichText::new(tr(text)).size(12.5)).small())
        .on_hover_cursor(CursorIcon::PointingHand)
}

pub fn small_danger(ui: &mut Ui, text: &str) -> Response {
    let p = pal();
    ui.add(
        egui::Button::new(RichText::new(tr(text)).size(12.5).color(p.danger))
            .small()
            .stroke(Stroke::new(1.0, tint(p.danger, 110))),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}

pub fn badge(ui: &mut Ui, text: &str, color: Color32) {
    Frame::none()
        .fill(tint(color, 38))
        .rounding(Rounding::same(20.0))
        .inner_margin(Margin::symmetric(9.0, 2.0))
        .show(ui, |ui| {
            ui.label(RichText::new(tr(text)).color(color).size(12.0).strong());
        });
}

pub fn status_badge(ui: &mut Ui, status: &str) {
    badge(ui, status, status_color(status));
}

pub fn level_badge(ui: &mut Ui, level: u8, text: &str) {
    badge(ui, text, level_color(level));
}

pub fn due_label(days: i64) -> String {
    if days < 0 {
        format!("{} {} {}", tr("Expired"), -days, tr("days ago"))
    } else if days == 0 {
        tr("Today")
    } else {
        format!("{} {}", days, tr("days left"))
    }
}

pub fn expiry_level(date: &str) -> (u8, String) {
    match f::days_until(date) {
        Some(d) => {
            let l = if d < 0 {
                2
            } else if d <= 30 {
                1
            } else {
                0
            };
            (l, due_label(d))
        }
        None => (0, "-".to_string()),
    }
}

pub fn cell(ui: &mut Ui, text: &str) {
    let n = ((ui.available_width() / 7.2) as usize).max(4);
    let r = ui.label(cut(text, n));
    if text.chars().count() > n {
        r.on_hover_text(text);
    }
}

pub fn cell_strong(ui: &mut Ui, text: &str) {
    let n = ((ui.available_width() / 7.8) as usize).max(4);
    let r = ui.label(RichText::new(cut(text, n)).strong());
    if text.chars().count() > n {
        r.on_hover_text(text);
    }
}

pub fn cell_muted(ui: &mut Ui, text: &str) {
    let n = ((ui.available_width() / 7.2) as usize).max(4);
    ui.label(RichText::new(cut(text, n)).color(pal().muted));
}

pub fn empty(ui: &mut Ui, title: &str, hint: &str) {
    let p = pal();
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(64.0, 64.0), Sense::hover());
        ui.painter()
            .circle_filled(r.center(), 32.0, tint(p.accent, 28));
        icons::draw(ui.painter(), Icon::Documents, r.center(), 28.0, p.accent);
        ui.add_space(8.0);
        ui.label(RichText::new(tr(title)).size(18.0).strong());
        ui.label(RichText::new(tr(hint)).color(p.muted));
    });
}

pub fn search(ui: &mut Ui, text: &mut String) -> Response {
    ui.add(
        TextEdit::singleline(text)
            .hint_text(tr("Search..."))
            .desired_width(250.0)
            .margin(vec2(10.0, 6.0)),
    )
}

pub fn filter_combo(ui: &mut Ui, id: &str, cur: &mut String, opts: &[&str], all: &str) {
    let shown = if cur.is_empty() {
        tr(all)
    } else {
        tr(cur.as_str())
    };
    egui::ComboBox::from_id_salt(id)
        .width(150.0)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            if ui.selectable_label(cur.is_empty(), tr(all)).clicked() {
                cur.clear();
            }
            for o in opts {
                if ui.selectable_label(*cur == *o, tr(o)).clicked() {
                    *cur = (*o).to_string();
                }
            }
        });
}

pub fn filter_combo_id(ui: &mut Ui, id: &str, cur: &mut i64, items: &[(i64, String)], all: &str) {
    let shown = items
        .iter()
        .find(|(i, _)| *i == *cur)
        .map(|(_, n)| cut(n, 28))
        .unwrap_or_else(|| tr(all));
    egui::ComboBox::from_id_salt(id)
        .width(190.0)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            if ui.selectable_label(*cur == 0, tr(all)).clicked() {
                *cur = 0;
            }
            for (i, n) in items {
                if ui.selectable_label(*cur == *i, n.as_str()).clicked() {
                    *cur = *i;
                }
            }
        });
}

pub fn combo(ui: &mut Ui, id: &str, cur: &mut String, opts: &[&str], width: f32) {
    egui::ComboBox::from_id_salt(id)
        .width(width)
        .selected_text(tr(cur.as_str()))
        .show_ui(ui, |ui| {
            for o in opts {
                if ui.selectable_label(*cur == *o, tr(o)).clicked() {
                    *cur = (*o).to_string();
                }
            }
        });
}

pub fn combo_id(
    ui: &mut Ui,
    id: &str,
    cur: &mut i64,
    items: &[(i64, String)],
    none: Option<&str>,
    width: f32,
) -> bool {
    let mut changed = false;
    let shown = items
        .iter()
        .find(|(i, _)| *i == *cur)
        .map(|(_, n)| cut(n, 36))
        .unwrap_or_else(|| tr(none.unwrap_or("Select...")));
    egui::ComboBox::from_id_salt(id)
        .width(width)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            if let Some(n) = none {
                if ui.selectable_label(*cur == 0, tr(n)).clicked() {
                    *cur = 0;
                    changed = true;
                }
            }
            for (i, n) in items {
                if ui.selectable_label(*cur == *i, n.as_str()).clicked() {
                    *cur = *i;
                    changed = true;
                }
            }
        });
    changed
}

pub fn form_label(ui: &mut Ui, text: &str, required: bool) {
    let t = if required {
        format!("{} *", tr(text))
    } else {
        tr(text)
    };
    ui.label(RichText::new(t).color(pal().muted).size(13.0));
}

pub fn form_text(ui: &mut Ui, label: &str, required: bool, val: &mut String, width: f32) {
    form_label(ui, label, required);
    ui.add(
        TextEdit::singleline(val)
            .desired_width(width)
            .margin(vec2(8.0, 5.0)),
    );
}

pub fn form_password(
    ui: &mut Ui,
    label: &str,
    required: bool,
    val: &mut String,
    width: f32,
) -> Response {
    form_label(ui, label, required);
    ui.add(
        TextEdit::singleline(val)
            .password(true)
            .desired_width(width)
            .margin(vec2(10.0, 7.0)),
    )
}

pub fn form_multiline(ui: &mut Ui, label: &str, val: &mut String, width: f32) {
    form_label(ui, label, false);
    ui.add(
        TextEdit::multiline(val)
            .desired_width(width)
            .desired_rows(2)
            .margin(vec2(8.0, 5.0)),
    );
}

pub fn form_date(ui: &mut Ui, label: &str, required: bool, val: &mut String, width: f32) {
    form_label(ui, label, required);
    ui.horizontal(|ui| {
        ui.add(
            TextEdit::singleline(val)
                .hint_text(f::date_hint())
                .desired_width(width - 66.0)
                .margin(vec2(8.0, 5.0)),
        );
        if small(ui, "Today").clicked() {
            *val = f::today_input();
        }
    });
}

pub fn form_combo(
    ui: &mut Ui,
    label: &str,
    required: bool,
    id: &str,
    cur: &mut String,
    opts: &[&str],
    width: f32,
) {
    form_label(ui, label, required);
    combo(ui, id, cur, opts, width);
}

pub fn error_text(ui: &mut Ui, msg: &str) {
    if !msg.is_empty() {
        let p = pal();
        Frame::none()
            .fill(tint(p.danger, 26))
            .stroke(Stroke::new(1.0, tint(p.danger, 110)))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.label(RichText::new(tr(msg)).color(p.danger));
            });
        ui.add_space(6.0);
    }
}

pub fn kv(ui: &mut Ui, key: &str, value: &str) {
    ui.label(RichText::new(tr(key)).color(pal().muted).size(13.0));
    ui.label(if value.is_empty() { "-" } else { value });
    ui.end_row();
}

pub fn stat_card(
    ui: &mut Ui,
    w: f32,
    icon: Icon,
    label: &str,
    value: String,
    sub: &str,
    color: Color32,
) {
    let p = pal();
    card_w(ui, w, |ui| {
        ui.set_min_height(68.0);
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(48.0, 48.0), Sense::hover());
            ui.painter()
                .rect_filled(r, Rounding::same(13.0), tint(color, 34));
            icons::draw(ui.painter(), icon, r.center(), 24.0, color);
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(RichText::new(tr(label)).color(p.muted).size(12.5));
                ui.label(RichText::new(value).size(26.0).strong());
                ui.label(RichText::new(sub).color(p.muted).size(12.0));
            });
        });
    });
}

pub fn count_up(ctx: &egui::Context, key: &str, target: f64) -> f64 {
    target * super::anim::progress(ctx, key, 0.7) as f64
}

pub fn table(
    ui: &mut Ui,
    id: &str,
    cols: &[Col],
    rows: usize,
    sort: &mut Sort,
    mut cell_fn: impl FnMut(&mut Ui, usize, usize),
) {
    let p = pal();
    ui.push_id(id, |ui| {
        let mut tb = TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(Layout::left_to_right(Align::Center))
            .min_scrolled_height(0.0);
        for c in cols {
            tb = if c.width > 0.0 {
                tb.column(Column::initial(c.width).at_least(48.0).clip(true))
            } else {
                tb.column(Column::remainder().at_least(80.0).clip(true))
            };
        }
        tb.header(32.0, |mut header| {
            for (i, c) in cols.iter().enumerate() {
                header.col(|ui| {
                    let arrow = if c.sort && sort.col == i {
                        if sort.asc {
                            " ^"
                        } else {
                            " v"
                        }
                    } else {
                        ""
                    };
                    let txt = RichText::new(format!("{}{}", tr(&c.title).to_uppercase(), arrow))
                        .strong()
                        .size(11.5)
                        .color(if sort.col == i { p.accent } else { p.muted });
                    if c.sort {
                        let r = ui
                            .add(egui::Label::new(txt).sense(Sense::click()))
                            .on_hover_cursor(CursorIcon::PointingHand);
                        if r.clicked() {
                            if sort.col == i {
                                sort.asc = !sort.asc;
                            } else {
                                sort.col = i;
                                sort.asc = true;
                            }
                        }
                    } else {
                        ui.label(txt);
                    }
                });
            }
        })
        .body(|body| {
            body.rows(38.0, rows, |mut row| {
                let i = row.index();
                for ci in 0..cols.len() {
                    row.col(|ui| cell_fn(ui, i, ci));
                }
            });
        });
    });
}

pub fn apply_sort(
    idx: &mut Vec<usize>,
    sort: Sort,
    mut cmp: impl FnMut(usize, usize, usize) -> std::cmp::Ordering,
) {
    idx.sort_by(|a, b| {
        let o = cmp(sort.col, *a, *b);
        if sort.asc {
            o
        } else {
            o.reverse()
        }
    });
}

pub fn contains_ci(hay: &str, needle: &str) -> bool {
    needle.is_empty() || hay.to_lowercase().contains(&needle.to_lowercase())
}

pub fn row_actions(ui: &mut Ui, view: bool, edit: bool, delete: bool) -> (bool, bool, bool) {
    let mut out = (false, false, false);
    if view && small(ui, "View").clicked() {
        out.0 = true;
    }
    if edit && small(ui, "Edit").clicked() {
        out.1 = true;
    }
    if delete && small_danger(ui, "Delete").clicked() {
        out.2 = true;
    }
    out
}

pub fn pair(
    ui: &mut Ui,
    ratio: f32,
    left: impl FnOnce(&mut Ui, f32),
    right: impl FnOnce(&mut Ui, f32),
) {
    let avail = ui.available_width();
    let gap = 16.0;
    if avail >= 900.0 {
        let w1 = ((avail - gap) * ratio).floor();
        let w2 = avail - gap - w1;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            left(ui, w1);
            right(ui, w2);
        });
    } else {
        left(ui, avail);
        ui.add_space(gap);
        right(ui, avail);
    }
}

pub fn metric(ui: &mut Ui, label: &str, value: &str, w: f32) {
    let p = pal();
    ui.allocate_ui(vec2(w, 46.0), |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(RichText::new(tr(label)).color(p.muted).size(12.0));
            ui.label(RichText::new(value).size(19.0).strong());
        });
    });
}

pub fn tabs(ui: &mut Ui, cur: &mut usize, labels: &[&str]) {
    let p = pal();
    ui.horizontal(|ui| {
        for (i, l) in labels.iter().enumerate() {
            let active = *cur == i;
            let text = RichText::new(tr(l)).size(14.0).strong().color(if active {
                p.accent
            } else {
                p.muted
            });
            let r = ui
                .add(
                    egui::Button::new(text)
                        .frame(false)
                        .min_size(vec2(0.0, 30.0)),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            if active {
                let y = r.rect.bottom() + 1.0;
                ui.painter().line_segment(
                    [
                        egui::pos2(r.rect.left() + 4.0, y),
                        egui::pos2(r.rect.right() - 4.0, y),
                    ],
                    Stroke::new(2.5, p.accent),
                );
            }
            if r.clicked() {
                *cur = i;
            }
            ui.add_space(6.0);
        }
    });
    ui.add_space(6.0);
}
