use egui::{emath::TSTransform, pos2, vec2, Color32, Rect};

use crate::{
    frontend::pages::BSKY_BLUE,
    image::{ImageCache, LoadableImage},
    widgets::spinner::SegoeBootSpinner,
};

pub struct FrontendMediaImageView {
    uri: String,

    transform: TSTransform,
}

impl FrontendMediaImageView {
    pub fn new(uri: String) -> Self {
        Self {
            uri,
            transform: TSTransform::default(),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, image: &ImageCache) {
        let fuck = ui.ctx().screen_rect();
        match image.get_image(&self.uri) {
            LoadableImage::Unloaded | LoadableImage::Loading => {
                SegoeBootSpinner::new().color(BSKY_BLUE).size(200.0).paint_at(ui, fuck);
            }
            LoadableImage::Loaded(texture_id, size) => {
                let response = ui.allocate_rect(ui.ctx().screen_rect(), egui::Sense::click_and_drag());

                if response.dragged() {
                    self.transform.translation += response.drag_delta();
                }

                if response.double_clicked() {
                    self.transform = TSTransform::default();
                }

                let transform = TSTransform::from_translation(ui.ctx().screen_rect().left_top().to_vec2()) * self.transform;

                if let Some(pointer) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                    // Note: doesn't catch zooming / panning if a button in this PanZoom container is hovered.
                    if response.hovered() {
                        let pointer_in_layer = transform.inverse() * pointer;
                        let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
                        let pan_delta = ui.ctx().input(|i| i.smooth_scroll_delta);

                        // Zoom in on pointer:
                        self.transform = self.transform
                            * TSTransform::from_translation(pointer_in_layer.to_vec2())
                            * TSTransform::from_scaling(zoom_delta)
                            * TSTransform::from_translation(-pointer_in_layer.to_vec2());

                        // Pan:
                        self.transform = TSTransform::from_translation(pan_delta) * self.transform;
                    }
                }


                
                let img_ratio = size.x / size.y;
                let view_ratio = fuck.width() / fuck.height();

                let rect = if img_ratio > view_ratio {
                    // wider than the view
                    Rect::from_center_size(fuck.center(), vec2(fuck.width(), (1.0 / img_ratio) * fuck.width()))
                } else {
                    // taller than the view
                    Rect::from_center_size(fuck.center(), vec2(img_ratio * fuck.height(), fuck.height()))
                };

                let rect = rect.translate(self.transform.translation);
                let rect = Rect::from_two_pos(rect.left_top(), rect.left_top() + rect.size() * self.transform.scaling);

                ui.painter().image(texture_id, rect, Rect { min: pos2(0.0, 0.0), max: pos2(1.0, 1.0) }, Color32::WHITE);
            }
        }
    }
}
