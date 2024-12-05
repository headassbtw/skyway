use egui::{pos2, vec2, Align2, Color32, FontId, ImageSize, ImageSource, Layout, Rect, Rounding, ScrollArea, Ui, UiBuilder};

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
			let funny_rect = ui.cursor().with_max_x(ui.ctx().screen_rect().right()).with_max_y(ui.ctx().screen_rect().bottom());
			let height = funny_rect.height() - funny_rect.top();
			let mut who_gaf = ui.child_ui(funny_rect, Layout::left_to_right(egui::Align::Min), None);
			who_gaf.style_mut().spacing.item_spacing = vec2(4.0, 4.0);
			ScrollArea::horizontal().vscroll(false)
			.max_width(funny_rect.width()).max_height(funny_rect.height())
			.scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible).show(&mut who_gaf, |ui| {
				ui.allocate_space(vec2(0.0, funny_rect.height()));
					ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
						let (_, rect0) = ui.allocate_space(vec2(height * 1.5, (height - ui.style().spacing.item_spacing.y) * (2.0 / 3.0)));
						let (_, rect1) = ui.allocate_space(vec2(height * 1.5, (height - ui.style().spacing.item_spacing.y) * (1.0 / 3.0)));

						if let Some(pfp) = &profile.banner {
							match image.get_image(pfp) {
						        LoadableImage::Unloaded |
						        LoadableImage::Loading => {
						        	ui.painter().rect_filled(rect0, Rounding::ZERO, Color32::GRAY);
						        	SegoeBootSpinner::new().size(200.0).color(Color32::from_black_alpha(64)).paint_at(ui, rect0);
						        },
						        LoadableImage::Loaded(texture_id, size) => {
						        	let ratio_src = size.x / size.y;
						        	let ratio_dst = rect0.width() / rect0.height();
						        	if ratio_dst < 1.0 { todo!() }

						        	let offset = (ratio_dst / ratio_src) * 0.5;

						        	let uv_rect = Rect { min: pos2(0.5 - offset, 0.0), max: pos2(0.5 + offset, 1.0) };
						        	ui.painter().image(texture_id, rect0, uv_rect, Color32::GRAY);
						        },
						    }
						} else {
							ui.painter().rect_filled(rect0, Rounding::ZERO, Color32::GRAY);
						}

						let upper_center = rect0.center() - vec2(0.0, rect0.height() * (0.5 / 3.0));
						let lower_center = rect0.center() + vec2(0.0, rect0.height() * (0.5 / 3.0));

						if let Some(dn) = profile.display_name() {
							ui.painter().text(lower_center - vec2(0.0, 20.0), Align2::CENTER_BOTTOM, dn, FontId::proportional(20.0), Color32::WHITE);
							ui.painter().text(lower_center + vec2(0.0, 20.0), Align2::CENTER_BOTTOM, format!("@{}", profile.handle), FontId::proportional(20.0), Color32::WHITE);
						} else {
							ui.painter().text(lower_center, Align2::CENTER_BOTTOM, format!("@{}", profile.handle), FontId::proportional(20.0), Color32::WHITE);
						}

						if let Some(avatar) = &profile.avatar {
							let pfp_rect = Rect::from_center_size(upper_center, vec2(100.0, 100.0));

							match image.get_image(avatar) {
						        LoadableImage::Unloaded |
						        LoadableImage::Loading => {
						        	ui.painter().rect_filled(pfp_rect, Rounding::ZERO, BSKY_BLUE);
						        	SegoeBootSpinner::new().size(80.0).color(Color32::WHITE).paint_at(ui, pfp_rect);
						        },
						        LoadableImage::Loaded(texture_id, vec2) => {
						        	ui.painter().image(texture_id, pfp_rect, Rect { min: pos2(0.0, 0.0), max: pos2(1.0, 1.0) }, Color32::WHITE);
						        },
						    }
						}


						if let Some(bio) = &profile.description {
							ui.painter().rect_filled(rect1, Rounding::ZERO, ui.style().visuals.extreme_bg_color);
							ui.new_child(UiBuilder::new().max_rect(rect1.shrink(4.0))).label(bio);
						}
					});
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