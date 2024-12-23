use egui::{pos2, vec2, Color32, Rect, Rounding, Stroke, UiBuilder};

use crate::{defs::bsky::embed::external, image::ImageCache, open_in_browser};

pub fn view_external(ui: &mut egui::Ui, external: &external::View, _media_size: f32, img_cache: &ImageCache) -> egui::Response {
	puffin::profile_function!();
	let resp = ui.allocate_new_ui(UiBuilder::default().max_rect(ui.cursor().shrink(5.0)), |link| {

		let a = egui::Label::new(&external.title).selectable(false).layout_in_ui(link);
        let b = if external.description.len() > 0 {
            Some(egui::Label::new(&external.description).selectable(false).layout_in_ui(link))
        } else { None };
        let rtn = link.allocate_space(vec2(2.0, 2.0)).1;
        link.style_mut().spacing.item_spacing.y = 0.0;
        let c = egui::Label::new(&external.uri).selectable(false).layout_in_ui(link);
        link.allocate_space(vec2(0.0, 4.0));
        (rtn, a, b, c)
    });

	// we've already done layout, don't bother rendering what we can't see
	if !ui.is_rect_visible(resp.response.rect) { return resp.response; }

	if let Some(thumb) = &external.thumb {
		match img_cache.get_image(thumb) {
		    crate::image::LoadableImage::Loaded(texture_id, vec2) => {
		    	let ratio = (resp.response.rect.height() / resp.response.rect.width()) / (vec2.y / vec2.x);
		    	let rect = Rect::from_min_max(pos2(0.0, 0.5 - 0.5 * ratio), pos2(1.0, 0.5 + 0.5 * ratio));
		    	ui.painter().image(texture_id, resp.response.rect.expand(4.0), rect, if ui.style().visuals.dark_mode { Color32::from_white_alpha(32) } else { Color32::from_white_alpha(128) });
		    },
		    _ => {},
		}
	}

	ui.painter().galley(resp.inner.1.0, resp.inner.1.1, ui.style().visuals.text_color());
	if let Some(thing) = resp.inner.2 {
		ui.painter().galley(thing.0, thing.1, ui.style().visuals.text_color());	
	}
	ui.painter().galley(resp.inner.3.0, resp.inner.3.1, ui.style().visuals.text_color());

    ui.painter().rect(resp.response.rect.expand(4.0), Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(2.0, ui.visuals().weak_text_color()));

    ui.painter().rect_filled(resp.inner.0.with_max_x(resp.response.rect.max.x), Rounding::ZERO, ui.visuals().weak_text_color());

    if resp.response.interact(egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        open_in_browser(&external.uri);
    }

    resp.response
}