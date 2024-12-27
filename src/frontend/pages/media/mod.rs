pub mod image;
pub mod video;

use egui::{pos2, vec2, Align2, Color32, FontId, Rounding, UiBuilder};
use image::FrontendMediaImageView;
use video::FrontendMediaVideoView;

use crate::image::ImageCache;

use super::{MainViewProposition, ViewStackReturnInfo};

pub enum FrontendMediaViewVariant {
    Image(FrontendMediaImageView),
    Video(FrontendMediaVideoView),
}

impl FrontendMediaViewVariant {
    pub fn render(&mut self, ui: &mut egui::Ui, image: &ImageCache, new_view: &mut MainViewProposition) -> ViewStackReturnInfo {
        ui.painter().rect_filled(ui.ctx().screen_rect(), Rounding::ZERO, Color32::BLACK);

        match self {
            FrontendMediaViewVariant::Image(data) => {
                data.render(ui, image);
            }
            FrontendMediaViewVariant::Video(_) => {
                ui.painter().text(pos2(ui.cursor().center().x, ui.ctx().screen_rect().center().y), Align2::CENTER_CENTER, "Videos are not supported yet", FontId::proportional(50.0), ui.visuals().text_color());
            }
        }
        // this is down here so it's always above the image or video
        //TODO: find actual dimentions, i have refs on my main computer but i'm comfy rn
        let back_rect = ui.allocate_rect(egui::Rect::from_center_size(ui.ctx().screen_rect().left_top() + vec2(60.0, 60.0), vec2(40.0, 40.0)), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);        
        ui.painter().text(back_rect.rect.center(), Align2::CENTER_CENTER, "\u{E0BA}", FontId::new(40.0, egui::FontFamily::Name("Segoe Symbols".into())), Color32::WHITE);
        ViewStackReturnInfo {
            title: None,
            render_back_button: false,
            handle_back_logic: true,
        }
    }
}
