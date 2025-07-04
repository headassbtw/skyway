use eframe::emath::{pos2, vec2, Align2, Rect};
use eframe::epaint::{Color32, FontId, Rounding};
use egui::{Response, Ui};
use crate::BSKY_BLUE;
use crate::image::{ImageCache, LoadableImage};
use crate::widgets::spinner::SegoeBootSpinner;

pub fn profile_picture_viewer(ui: &mut Ui, avatar: &Option<String>, size: [f32; 2], img_cache: &ImageCache) -> Response {
    let response = ui.allocate_response(vec2(size[0], size[1]), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
    let rect = response.rect;
    if !ui.is_rect_visible(rect) {
        return response;
    }
    match avatar {
        Some(avatar) => {
            match img_cache.get_image(avatar) {
                LoadableImage::Unloaded => {
                    ui.painter().rect_filled(rect, Rounding::ZERO, Color32::RED);
                    SegoeBootSpinner::new().size(f32::min(size[0], size[1] * 0.6)).color(Color32::WHITE).paint_at(ui, rect);
                }
                LoadableImage::Loading => {
                    ui.painter().rect_filled(rect, Rounding::ZERO, BSKY_BLUE);
                    SegoeBootSpinner::new().size(f32::min(size[0], size[1] * 0.6)).color(Color32::WHITE).paint_at(ui, rect);
                }
                LoadableImage::Loaded(texture_id, _) => {
                    ui.painter().image(texture_id, rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                }
            }
        }
        None => {
            ui.painter().rect_filled(rect, Rounding::ZERO, BSKY_BLUE);
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, "", FontId::new(f32::min(size[0], size[1] * 0.8), egui::FontFamily::Name("Segoe Symbols".into())), Color32::WHITE);
        }
    }

    response
}