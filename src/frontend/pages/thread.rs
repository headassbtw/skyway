use std::sync::{Arc, Mutex};

use egui::{pos2, Align2, Color32, FontId, ScrollArea, Ui};

use crate::backend::thread::{BlueskyApiThreadResponse, BlueskyApiThreadViewPost};
use crate::frontend::main::ClientFrontendFlyout;
use crate::frontend::pages::BSKY_BLUE;
use crate::frontend::viewers;
use crate::widgets::spinner::SegoeBootSpinner;
use crate::{backend::thread::BlueskyApiGetThreadResponse, bridge::FrontToBackMsg};
use crate::{bridge::Bridge, image::ImageCache};

use super::MainViewProposition;
#[derive(Debug)]
pub struct FrontendThreadView {
    pub data: Option<BlueskyApiGetThreadResponse>,
    pub id_cmp: String,
    pub loading: bool,
}

impl FrontendThreadView {
    pub fn new(did: String) -> Self {
        Self { data: None, id_cmp: did, loading: false }
    }
}

impl FrontendThreadView {
	fn render_recursive(ui: &mut Ui, thread: &BlueskyApiThreadResponse, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) {
		match thread {
            BlueskyApiThreadResponse::NotFound(_) => {
                ui.heading("Not Found");
            }
            BlueskyApiThreadResponse::Blocked(_) => {
                ui.heading("Blocked");
            }
            BlueskyApiThreadResponse::ThreadView(thread) => {
            	if let Some(parent) = &thread.parent {
            		Self::render_recursive(ui, &parent, backend, image, flyout, new_view);
            	}
                viewers::post::post_viewer(ui, thread.post.clone(), backend, image, flyout, new_view);
            	//ui.label(format!("{}: {}", &thread.post.author.handle, &thread.post.record.text));
            }
        }
	}

    pub fn render(&mut self, ui: &mut Ui, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> &str {
        puffin::profile_function!();
        if let Some(thread) = &self.data {
            ScrollArea::vertical().hscroll(false).show(ui, |scroll| {
                Self::render_recursive(scroll, &thread.thread, backend, image, flyout, new_view);
            });
        	
        } else {
        	SegoeBootSpinner::new().size(200.0).color(BSKY_BLUE).paint_at(ui, ui.ctx().screen_rect());
            if !self.loading {
                backend.backend_commander.send(FrontToBackMsg::GetThreadRequest(self.id_cmp.clone())).unwrap();
                self.loading = true;
            }
        }

        "Thread"
    }
}
