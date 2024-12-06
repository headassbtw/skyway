use egui::{pos2, vec2, Color32, Rect, Response, Rounding};

use crate::{frontend::pages::BSKY_BLUE, image::{ImageCache, LoadableImage}, widgets::spinner::SegoeBootSpinner};

pub struct FrontendMediaImageView {
	uri: String,
}

impl FrontendMediaImageView {
	pub fn new(uri: String) -> Self {
		Self {
			uri
		}
	}

	pub fn render(&mut self, ui: &mut egui::Ui, image: &ImageCache) {
		let fuck = ui.cursor().with_max_y(ui.ctx().screen_rect().bottom());
		match image.get_image(&self.uri) {
		    LoadableImage::Unloaded |
		    LoadableImage::Loading => {
		    	SegoeBootSpinner::new().color(BSKY_BLUE).size(200.0).paint_at(ui, fuck);
		    },
		    LoadableImage::Loaded(texture_id, size) => {
		    	let img_ratio = size.x / size.y;
		    	let view_ratio = fuck.width() / fuck.height();

		    	if img_ratio > view_ratio {
		    		// wider than the view
		    		let rect = Rect::from_center_size(fuck.center(), vec2(fuck.width(), (1.0 / img_ratio) * fuck.width()));
		    		ui.painter().image(texture_id, rect, Rect { min: pos2(0.0, 0.0), max: pos2(1.0, 1.0) }, Color32::WHITE);
		    	} else {
		    		// taller than the view
		    		let rect = Rect::from_center_size(fuck.center(), vec2(img_ratio * fuck.height(), fuck.height()));
		    		ui.painter().image(texture_id, rect, Rect { min: pos2(0.0, 0.0), max: pos2(1.0, 1.0) }, Color32::WHITE);
		    	}
		    },
		}
	}
}