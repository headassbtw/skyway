use egui::{pos2, Align2, Color32, FontId, Rect, Rounding};

use crate::{
    defs::bsky::embed::video,
    frontend::pages::{
        media::{video::FrontendMediaVideoView, FrontendMediaViewVariant},
        FrontendMainView, MainViewProposition,
    },
    image::{ImageCache, LoadableImage},
    BSKY_BLUE,
};

pub fn view_video(ui: &mut egui::Ui, video: &video::View, media_size: f32, img_cache: &ImageCache, new_view: &mut MainViewProposition) -> egui::Response {
    puffin::profile_function!();
    let video_rect = ui.cursor().with_max_y(ui.cursor().top() + media_size);
    if !ui.is_rect_visible(video_rect) {
        puffin::profile_scope!("Video Short-Circuit");
        return ui.allocate_rect(video_rect, egui::Sense::click());
    }
    let ratio = if let Some(real_ratio) = &video.aspect_ratio { real_ratio.width as f32 / real_ratio.height as f32 } else { 16.0 / 9.0 };

    let video_rect = video_rect.with_max_x(video_rect.left() + media_size * ratio);

    let tex = if let Some(thumb) = &video.thumbnail {
        match img_cache.get_image(thumb) {
            LoadableImage::Loaded(texture_id, _) => Some(texture_id),
            _ => None,
        }
    } else {
        None
    };

    if let Some(id) = tex {
        ui.painter().image(id, video_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    } else {
        ui.painter().rect_filled(video_rect, Rounding::ZERO, Color32::BLACK);
    }
    let rtn = ui.allocate_rect(video_rect, egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
    ui.painter().rect_filled(video_rect, Rounding::ZERO, Color32::from_white_alpha(64));
    ui.painter().circle_filled(video_rect.center(), 15.0, Color32::from_black_alpha(128));
    ui.painter().text(video_rect.center(), Align2::CENTER_CENTER, "\u{25B6}", FontId::new(20.0, egui::FontFamily::Name("Segoe Symbols".into())), BSKY_BLUE);

    if rtn.clicked() {
        new_view.set(FrontendMainView::Media(FrontendMediaViewVariant::Video(FrontendMediaVideoView {})));
    }

    rtn
}
