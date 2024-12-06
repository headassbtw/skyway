use std::sync::{Arc, Mutex};

use egui::{pos2, vec2, Align2, Color32, FontId, Response, Rounding, ScrollArea, Stroke, Ui};

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
	fn render_recursive(ui: &mut Ui, thread: &BlueskyApiThreadResponse, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> Response{
		match thread {
            BlueskyApiThreadResponse::NotFound(_) => {
                ui.heading("Not Found")
            }
            BlueskyApiThreadResponse::Blocked(_) => {
                ui.heading("Blocked")
            }
            BlueskyApiThreadResponse::ThreadView(thread) => {
            	if let Some(parent) = &thread.parent {
            		let res = Self::render_recursive(ui, &parent, backend, image, flyout, new_view);
                    ui.painter().line_segment([pos2(res.rect.left() + 30.0, res.rect.top() + 70.0), pos2(res.rect.left() + 30.0, res.rect.bottom() + (ui.style().spacing.item_spacing.y * 2.0)+ 10.0)], ui.style().visuals.widgets.inactive.fg_stroke);
            	}
                viewers::post::post_viewer(ui, thread.post.clone(), backend, image, flyout, new_view)
            }
        }
	}

    pub fn render(&mut self, ui: &mut Ui, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> &str {
        puffin::profile_function!();
        if let Some(thread) = &self.data {
            ScrollArea::vertical().hscroll(false).show(ui, |scroll| {
                let guh = Self::render_recursive(scroll, &thread.thread, backend, image, flyout, new_view);
                match &thread.thread {
                    BlueskyApiThreadResponse::NotFound(_) |
                    BlueskyApiThreadResponse::Blocked(_) => { }
                    BlueskyApiThreadResponse::ThreadView(thread) => {
                        let (_, line_rect) = scroll.allocate_space(vec2(guh.rect.width(), scroll.style().visuals.widgets.inactive.fg_stroke.width * 4.0));
                        scroll.painter().rect_filled(line_rect, Rounding::ZERO, scroll.style().visuals.widgets.inactive.fg_stroke.color);
                        if let Some(replies) = &thread.replies {
                            for reply in replies {
                                match reply {
                                    BlueskyApiThreadResponse::NotFound(_) |
                                    BlueskyApiThreadResponse::Blocked(_) => {},
                                    BlueskyApiThreadResponse::ThreadView(post) => {
                                        viewers::post::post_viewer(scroll, post.post.clone(), backend, image, flyout, new_view);
                                    },
                                }
                                
                            }
                        }
                    }
                }
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
