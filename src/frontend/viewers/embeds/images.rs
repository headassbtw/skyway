use egui::{pos2, vec2, Align2, Color32, FontId, Layout, Rect, Rounding, ScrollArea, UiBuilder};

use crate::{defs::bsky::embed::images::ViewImage, frontend::pages::{media::{image::FrontendMediaImageView, FrontendMediaViewVariant}, FrontendMainView, MainViewProposition}, image::{ImageCache, LoadableImage}};


pub fn view_images(ui: &mut egui::Ui, id_salt: egui::Id, images: &Vec<ViewImage>, media_size: f32, img_cache: &ImageCache, new_view: &mut MainViewProposition) -> egui::Response {
	puffin::profile_function!();
    let img_rect = ui.cursor().with_max_y(ui.cursor().top() + media_size);
    if !ui.is_rect_visible(img_rect) {
        puffin::profile_scope!("Image Short-Circuit");
        return ui.allocate_rect(img_rect, egui::Sense::click());
    }
    ui.allocate_new_ui(UiBuilder::default().max_rect(img_rect), |container| {
        ScrollArea::horizontal().max_width(img_rect.width()).max_height(img_rect.height()).vscroll(false).id_salt(id_salt).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded).show(container, |container| {
            container.with_layout(Layout::left_to_right(egui::Align::Min), |container| {
                for img in images {
                    
                    puffin::profile_scope!("Image");
                    let img_rect = match img_cache.get_image(&img.thumb) {
                        LoadableImage::Unloaded | LoadableImage::Loading => {
                            let x_multiplier = if let Some(ratio) = &img.aspect_ratio { ratio.width as f32 / ratio.height as f32 } else { 1.0 };
                            let rtn = container.allocate_rect(container.cursor().with_max_x(container.cursor().left() + (media_size * x_multiplier)), egui::Sense::click());
                            container.painter().rect_filled(rtn.rect, Rounding::ZERO, Color32::GRAY);
                            rtn
                        }
                        LoadableImage::Loaded(id, ratio) => {
                            // kind of jank because sometimes the ratio won't send but it always does when loaded
                            let x_multiplier = if let Some(ratio) = &img.aspect_ratio { ratio.width as f32 / ratio.height as f32 } else { ratio.x / ratio.y };
                            let rtn = container.allocate_rect(container.cursor().with_max_x(container.cursor().left() + (media_size * x_multiplier)), egui::Sense::click());
                            container.painter().image(id, rtn.rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                            rtn
                        }
                    };

                    // probably redundant but i love culling things
                    if !container.is_rect_visible(img_rect.rect) { continue; }

                    if img.alt.len() > 0 as usize {
                        puffin::profile_scope!("Alt Text");
                        let dim_rect = img_rect.rect.with_min_y(img_rect.rect.bottom() - 20.0);
                        container.painter().rect_filled(dim_rect, Rounding::ZERO, Color32::from_black_alpha(128));

                        container.painter().text(dim_rect.left_center() + vec2(10.0, 0.0), Align2::LEFT_CENTER, "ALT", FontId::proportional(12.0), Color32::WHITE);
                        let alt_guh = container.allocate_rect(dim_rect, egui::Sense::click());
                        alt_guh.on_hover_text(&img.alt);
                    }
                    if img_rect.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        new_view.set(FrontendMainView::Media(FrontendMediaViewVariant::Image(FrontendMediaImageView::new(img.fullsize.clone()))));
                    }
                }
            });
        });
    }).response
}