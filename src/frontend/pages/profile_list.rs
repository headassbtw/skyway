use std::sync::{Arc, Mutex};
use eframe::emath::vec2;
use egui::{Layout, ScrollArea, Ui};

use crate::{defs::bsky::actor::defs::ProfileView, bridge::Bridge, image::ImageCache, BSKY_BLUE};
use crate::bridge::FrontToBackMsg;
use crate::defs::bsky::feed::defs::FeedCursorPair;
use crate::frontend::CursorListPair;
use crate::frontend::viewers::profile::profile_viewer;
use crate::widgets::spinner::SegoeBootSpinner;
use super::{MainViewProposition, ViewStackReturnInfo};

pub struct ListData {
	pub did: String,
	pub profiles: Arc<Mutex<CursorListPair<ProfileView>>>,
}

pub enum FrontendProfileListVariant {
	Followers(ListData),
	Following(ListData),
	LikedBy(ListData),
	RepostedBy(ListData),
}

impl FrontendProfileListVariant {
	pub fn render(&mut self, ui: &mut Ui, backend: &Bridge, image: &ImageCache, new_view: &mut MainViewProposition) -> ViewStackReturnInfo {
		let (title, data, variant) = match self {
			FrontendProfileListVariant::Followers(data) =>  ("Followers", data, 0),
			FrontendProfileListVariant::Following(data) =>  ("Following", data, 1),
			FrontendProfileListVariant::LikedBy(data) =>    ("Likes",     data, 2),
			FrontendProfileListVariant::RepostedBy(data) => ("Reposts",   data, 3),
		};

		ScrollArea::vertical().hscroll(false).max_width(ui.cursor().width()).id_salt(format!("{}_ProfileList", &data.did)).show(ui, |ui| {
			let profiles = data.profiles.lock().unwrap();
			for profile in profiles.items.iter() {
				profile_viewer(ui, profile, image, new_view);
			}

			let loader_response = ui.add(SegoeBootSpinner::new().size(50.0).color(BSKY_BLUE));

			if ui.is_rect_visible(loader_response.rect) && profiles.cursor.is_some() {
				drop(profiles);
				match variant {
					0 => {
						backend.backend_commander.send(FrontToBackMsg::GetFollowersRequest {
							did: data.did.clone(),
							profiles: data.profiles.clone(),
						}).unwrap();
					}
					_ => {}
				}
			}
		});

		ViewStackReturnInfo {
			title: Some(title.into()),
			render_back_button: true,
			handle_back_logic: true,
			force_back: false
		}
	}
}