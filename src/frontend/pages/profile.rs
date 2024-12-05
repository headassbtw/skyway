use egui::{pos2, vec2, Color32, FontId, Layout, Rect, Rounding, Ui};

use crate::frontend::pages::BSKY_BLUE;
use crate::widgets::spinner::SegoeBootSpinner;
use crate::{backend::profile::BlueskyApiProfile, bridge::Bridge, image::ImageCache};
use crate::bridge::FrontToBackMsg;
use crate::image::LoadableImage;
#[derive(Debug)]
pub struct FrontendProfileView {
	pub profile_data: Option<BlueskyApiProfile>,
	pub id_cmp: String,
	pub loading: bool,
}

impl FrontendProfileView {
	pub fn new(did: String) -> Self {
		Self {
			profile_data: None,
			id_cmp: did,
			loading: false,
		}
	}
	pub fn render(&mut self, ui: &mut Ui, backend: &Bridge, image: &ImageCache) -> &str {
		puffin::profile_function!();
		if let Some(profile) = &self.profile_data {
			ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
				if let Some(pfp) = &profile.avatar {
					let (_, paint_rect) = ui.allocate_space(vec2(38.0, 38.0));
					match image.get_image(pfp) {
				        LoadableImage::Unloaded |
				        LoadableImage::Loading => {
				        	ui.painter().rect_filled(paint_rect, Rounding::ZERO, Color32::GRAY);
				        },
				        LoadableImage::Loaded(texture_id) => {
				        	ui.painter().image(texture_id, paint_rect, Rect { min: pos2(0.0, 0.0), max: pos2(1.0, 1.0) }, Color32::WHITE);
				        },
		    		}
				}
				let weak = if let Some(dn) = &profile.display_name {
					if dn.len() > 0 {
						ui.add(egui::Label::new(egui::RichText::new(dn).size(38.0)).selectable(false));
						ui.allocate_space(vec2(10.0, 10.0));
						true
					} else { false }
				} else { false };
				let mut handle_text = egui::RichText::new(&profile.handle).size(38.0);
				if weak { handle_text = handle_text.weak().font(FontId::new(38.0, egui::FontFamily::Name("Segoe Light".into()))); }

				ui.add(egui::Label::new(handle_text).selectable(false));
				
			});
		} else {
			SegoeBootSpinner::new().size(200.0).color(BSKY_BLUE).paint_at(ui, ui.ctx().screen_rect());
			if !self.loading {
				backend.backend_commander.send(FrontToBackMsg::GetProfileRequest(self.id_cmp.clone())).unwrap();
				self.loading = true;
			}
		}
		"Profile"
	}
}