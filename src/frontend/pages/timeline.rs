use std::sync::{Arc, Mutex};

use egui::{pos2, vec2, Align2, Color32, FontId, Layout, Rect, Rounding, ScrollArea, Stroke, Vec2};

use crate::{
    backend::responses::timeline::BlueskyApiTimelineResponseObject,
    bridge::Bridge,
    frontend::{flyouts::composer::ComposerFlyout, main::ClientFrontendFlyout, viewers},
    image::ImageCache,
    widgets::spinner::SegoeBootSpinner,
};

const BSKY_BLUE: Color32 = Color32::from_rgb(32, 139, 254);

pub struct FrontendTimelineView {
    pub timeline: Vec<Arc<Mutex<BlueskyApiTimelineResponseObject>>>,
    pub timeline_cursor: Option<String>,
    pub post_highlight: (usize, f32, bool),
}

impl FrontendTimelineView {
    pub fn new() -> Self {
        Self { timeline: Vec::new(), timeline_cursor: Some("".to_owned()), post_highlight: (0, 999.999, false) }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout) {
        puffin::profile_function!();
        let pos = pos2(ui.cursor().left(), ui.cursor().top() - 40.0);

        ui.painter().text(pos, Align2::LEFT_BOTTOM, "Timeline", FontId::new(40.0, egui::FontFamily::Name("Segoe Light".into())), BSKY_BLUE);
        let top = ui.cursor().top(); // the top of the scroll rect, used to compare post positions for keyboard nav
        ScrollArea::vertical().hscroll(false).max_width(ui.cursor().width()).max_height(ui.cursor().height()).show(ui, |tl| {
            let length = if self.timeline.len() <= 0 { 0 } else { self.timeline.len() - 1 };

            // keyboard nav polling
            let (scrolling, scroll_to) = {
                puffin::profile_scope!("Keyboard nav part A");

                let scroll_to: Option<usize> = if !tl.is_enabled() {
                    None
                } else {
                    tl.input(|r| {
                        puffin::profile_scope!("Key polling");
                        if r.key_pressed(egui::Key::K) {
                            self.post_highlight.0 += 1;
                            self.post_highlight.2 = true;
                            Some(self.post_highlight.0)
                        } else if r.key_pressed(egui::Key::J) && self.post_highlight.0 > 0 {
                            self.post_highlight.0 -= 1;
                            self.post_highlight.2 = true;
                            Some(self.post_highlight.0)
                        } else {
                            None
                        }
                    })
                };
                let scrolling = tl.input(|r| r.smooth_scroll_delta != Vec2::new(0.0, 0.0));
                if scrolling {
                    self.post_highlight.2 = false;
                }
                self.post_highlight.1 = 9999.9999;
                (scrolling, scroll_to)
            };
            for i in 0..length {
                puffin::profile_scope!("Post");
                let res = viewers::post::post_viewer(tl, self.timeline[i].clone(), backend, image, flyout);
                // keyboard nav comparison, checks if we're scrolling (no need to update if not), and if we are, sets the closest post to the top as the active one
                {
                    puffin::profile_scope!("Keyboard nav part B");
                    if scrolling
                    /* do some max height check here*/
                    {
                        let comp = f32::abs(top - res.rect.top());
                        if comp < self.post_highlight.1 {
                            self.post_highlight.1 = comp;
                            self.post_highlight.0 = i;
                        }
                    }
                    if self.post_highlight.2 && i == self.post_highlight.0 {
                        tl.painter().rect(res.rect, Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(4.0, BSKY_BLUE));
                    }
                    if let Some(to) = scroll_to {
                        if i == to {
                            res.scroll_to_me(Some(egui::Align::Min));
                        }
                    }
                }
            }
            tl.with_layout(Layout::top_down(egui::Align::Center), |spinner| {
                let spinner_rect = spinner.add_sized(vec2(40.0, 40.0), SegoeBootSpinner::new().size(40.0).color(BSKY_BLUE)).rect;
                if spinner.is_rect_visible(spinner_rect) && self.timeline_cursor.is_some() {
                    backend.backend_commander.send(crate::bridge::FrontToBackMsg::GetTimelineRequest(self.timeline_cursor.clone(), None)).unwrap();
                    self.timeline_cursor = None;
                }
            })
        });

        let search_pos = ui.ctx().screen_rect().right_top() + vec2(-80.0, 80.0);
        let compose_pos = search_pos - vec2(50.0, 0.0);
        let refresh_pos = compose_pos - vec2(50.0, 0.0);

        let _search_button = ui.allocate_rect(Rect::from_center_size(search_pos, vec2(30.0, 30.0)), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        let compose_button = ui.allocate_rect(Rect::from_center_size(compose_pos, vec2(30.0, 30.0)), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        let refresh_button = ui.allocate_rect(Rect::from_center_size(refresh_pos, vec2(30.0, 30.0)), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);

        ui.painter().text(search_pos, Align2::CENTER_CENTER, "\u{E11A}", FontId::new(30.0, egui::FontFamily::Name("Segoe Symbols".into())), BSKY_BLUE);
        ui.painter().text(compose_pos, Align2::CENTER_CENTER, "\u{E104}", FontId::new(30.0, egui::FontFamily::Name("Segoe Symbols".into())), BSKY_BLUE);
        ui.painter().text(refresh_pos, Align2::CENTER_CENTER, "\u{E0F2}", FontId::new(30.0, egui::FontFamily::Name("Segoe Symbols".into())), BSKY_BLUE);

        if compose_button.clicked() {
            flyout.set(crate::frontend::main::ClientFrontendFlyoutVariant::PostComposerFlyout(ComposerFlyout::new()));
        }

        if refresh_button.clicked() {
            self.timeline_cursor = Some(String::new());
            self.timeline.clear();
        }

        /*
        ui.label("LANDING PAGE");
        if ui.button("").clicked() {
            let data = LoginModal::new();
            self.modal = Some(crate::frontend::main::ClientFrontendModal::LoginModal(data));
        }
        */
    }
}
