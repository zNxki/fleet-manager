use eframe::egui::{self, Color32, FontFamily, FontId, Rounding, Stroke, TextStyle, Visuals};
use std::sync::atomic::{AtomicBool, Ordering};

static DARK: AtomicBool = AtomicBool::new(true);

pub fn set_dark(v: bool) {
    DARK.store(v, Ordering::Relaxed);
}

static ANIM: AtomicBool = AtomicBool::new(true);

pub fn set_animations(v: bool) {
    ANIM.store(v, Ordering::Relaxed);
}

pub fn animations() -> bool {
    ANIM.load(Ordering::Relaxed)
}

pub fn is_dark() -> bool {
    DARK.load(Ordering::Relaxed)
}

#[derive(Clone, Copy)]
pub struct Pal {
    pub bg: Color32,
    pub sidebar: Color32,
    pub card: Color32,
    pub panel: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub success: Color32,
    pub warn: Color32,
    pub danger: Color32,
    pub purple: Color32,
}

pub fn pal() -> Pal {
    if DARK.load(Ordering::Relaxed) {
        Pal {
            bg: Color32::from_rgb(11, 15, 22),
            sidebar: Color32::from_rgb(15, 20, 29),
            card: Color32::from_rgb(21, 27, 38),
            panel: Color32::from_rgb(29, 37, 51),
            border: Color32::from_rgb(40, 50, 68),
            text: Color32::from_rgb(230, 236, 245),
            muted: Color32::from_rgb(139, 151, 171),
            accent: Color32::from_rgb(59, 130, 246),
            success: Color32::from_rgb(34, 197, 94),
            warn: Color32::from_rgb(245, 158, 11),
            danger: Color32::from_rgb(239, 68, 68),
            purple: Color32::from_rgb(168, 85, 247),
        }
    } else {
        Pal {
            bg: Color32::from_rgb(243, 245, 249),
            sidebar: Color32::from_rgb(255, 255, 255),
            card: Color32::from_rgb(255, 255, 255),
            panel: Color32::from_rgb(238, 242, 248),
            border: Color32::from_rgb(222, 227, 236),
            text: Color32::from_rgb(22, 28, 40),
            muted: Color32::from_rgb(100, 112, 130),
            accent: Color32::from_rgb(37, 99, 235),
            success: Color32::from_rgb(22, 163, 74),
            warn: Color32::from_rgb(217, 119, 6),
            danger: Color32::from_rgb(220, 38, 38),
            purple: Color32::from_rgb(147, 51, 234),
        }
    }
}

pub fn tint(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

pub fn status_color(s: &str) -> Color32 {
    let p = pal();
    match s {
        "Available" | "Completed" | "Active" | "Valid" => p.success,
        "Assigned" | "Scheduled" | "Open" => p.accent,
        "Maintenance" | "In progress" => p.warn,
        "Expired" => p.danger,
        _ => p.muted,
    }
}

pub fn level_color(l: u8) -> Color32 {
    let p = pal();
    match l {
        2 => p.danger,
        1 => p.warn,
        _ => p.success,
    }
}

pub fn apply(ctx: &egui::Context, dark: bool) {
    set_dark(dark);
    let p = pal();
    let mut v = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    v.panel_fill = p.bg;
    v.window_fill = p.card;
    v.extreme_bg_color = p.panel;
    v.faint_bg_color = tint(p.text, 8);
    v.window_stroke = Stroke::new(1.0, p.border);
    v.window_rounding = Rounding::same(12.0);
    v.override_text_color = Some(p.text);
    v.hyperlink_color = p.accent;
    v.selection.bg_fill = tint(p.accent, 90);
    v.selection.stroke = Stroke::new(1.0, p.accent);
    v.widgets.noninteractive.bg_fill = p.card;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.border);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.inactive.bg_fill = p.panel;
    v.widgets.inactive.weak_bg_fill = p.panel;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, p.border);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.hovered.bg_fill = tint(p.accent, 50);
    v.widgets.hovered.weak_bg_fill = tint(p.accent, 50);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, p.accent);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.active.bg_fill = tint(p.accent, 90);
    v.widgets.active.weak_bg_fill = tint(p.accent, 90);
    v.widgets.active.bg_stroke = Stroke::new(1.0, p.accent);
    v.widgets.active.fg_stroke = Stroke::new(1.0, p.text);
    v.widgets.open.bg_fill = p.panel;
    v.widgets.open.weak_bg_fill = p.panel;
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.rounding = Rounding::same(7.0);
    }
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(10.0, 8.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
        s.spacing.interact_size.y = 28.0;
        s.text_styles.insert(
            TextStyle::Heading,
            FontId::new(24.0, FontFamily::Proportional),
        );
        s.text_styles
            .insert(TextStyle::Body, FontId::new(14.5, FontFamily::Proportional));
        s.text_styles.insert(
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        );
        s.text_styles.insert(
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        );
    });
}