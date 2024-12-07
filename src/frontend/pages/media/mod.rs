pub mod image;
pub mod video;

use egui::{pos2, Align2, FontId};
use image::FrontendMediaImageView;
use video::FrontendMediaVideoView;

use crate::image::ImageCache;

pub enum FrontendMediaViewVariant {
    Image(FrontendMediaImageView),
    Video(FrontendMediaVideoView),
}

impl FrontendMediaViewVariant {
    pub fn render(&mut self, ui: &mut egui::Ui, image: &ImageCache) -> &str {
        match self {
            FrontendMediaViewVariant::Image(data) => {
                data.render(ui, image);
            }
            FrontendMediaViewVariant::Video(_) => {
                ui.painter().text(pos2(ui.cursor().center().x, ui.ctx().screen_rect().center().y), Align2::CENTER_CENTER, "Videos are not supported yet", FontId::proportional(50.0), ui.visuals().text_color());
            }
        }
        "Media"
    }
}
