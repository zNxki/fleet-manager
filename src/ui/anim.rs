use eframe::egui::{Context, Id};
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};

static EPOCH: AtomicU64 = AtomicU64::new(0);

pub fn bump() {
    EPOCH.fetch_add(1, Ordering::Relaxed);
}

pub fn epoch() -> u64 {
    EPOCH.load(Ordering::Relaxed)
}

pub fn progress(ctx: &Context, key: impl Hash, dur: f32) -> f32 {
    if !crate::ui::theme::animations() {
        return 1.0;
    }
    let id = Id::new(("anim", epoch(), key));
    let now = ctx.input(|i| i.time);
    let start: f64 = ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(id, || now));
    let t = (((now - start) as f32) / dur).clamp(0.0, 1.0);
    if t < 1.0 {
        ctx.request_repaint();
    }
    1.0 - (1.0 - t).powi(3)
}
