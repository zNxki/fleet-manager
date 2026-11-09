use eframe::egui::{
    self, pos2, Color32, ColorImage, Context, Rect, Sense, TextureHandle, TextureOptions, Ui, Vec2,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct ImageCache {
    map: HashMap<String, Option<TextureHandle>>,
}

impl ImageCache {
    pub fn get(&mut self, ctx: &Context, path: &str) -> Option<TextureHandle> {
        if path.is_empty() {
            return None;
        }
        if let Some(entry) = self.map.get(path) {
            return entry.clone();
        }
        let loaded = load(ctx, path);
        self.map.insert(path.to_string(), loaded.clone());
        loaded
    }

}

fn load(ctx: &Context, path: &str) -> Option<TextureHandle> {
    let img = image::open(path).ok()?;
    let img = img.thumbnail(960, 960).to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture(path.to_string(), color, TextureOptions::LINEAR))
}

pub fn show_cover(ui: &mut Ui, tex: &TextureHandle, size: Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let ts = tex.size_vec2();
    let target = size.x / size.y;
    let src = ts.x / ts.y;
    let uv = if src > target {
        let w = target / src;
        Rect::from_min_max(pos2((1.0 - w) / 2.0, 0.0), pos2((1.0 + w) / 2.0, 1.0))
    } else {
        let h = src / target;
        Rect::from_min_max(pos2(0.0, (1.0 - h) / 2.0), pos2(1.0, (1.0 + h) / 2.0))
    };
    ui.painter().image(tex.id(), rect, uv, Color32::WHITE);
}

pub fn placeholder(ui: &mut Ui, size: Vec2) {
    let p = crate::ui::theme::pal();
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(
        rect,
        egui::Rounding::same(10.0),
        crate::ui::theme::tint(p.accent, 22),
    );
    crate::ui::icons::draw(
        ui.painter(),
        crate::ui::icons::Icon::Vehicles,
        rect.center(),
        (size.y * 0.35).min(size.x * 0.35),
        crate::ui::theme::tint(p.accent, 150),
    );
}
