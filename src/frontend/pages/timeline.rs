use egui::{load::SizedTexture, pos2, vec2, Align2, Color32, FontId, ImageSource, Layout, Rect, Rounding, ScrollArea, Stroke, Vec2};

use crate::{
    bridge::Bridge,
    defs::bsky::{
        actor::defs::ProfileViewDetailed,
        feed::defs::{FeedCursorPair, GeneratorView},
    },
    frontend::{
        flyouts::composer::ComposerFlyout,
        main::ClientFrontendFlyout,
        pages::{profile::FrontendProfileView, FrontendMainView},
        viewers,
    },
    image::ImageCache,
    widgets::{click_context_menu, spinner::SegoeBootSpinner},
    BSKY_BLUE,
};

use super::{MainViewProposition, ViewStackReturnInfo};

pub struct FrontendTimelineView {
    pub timeline: FeedCursorPair,
    pub feed: usize,
    pub feeds: Vec<(crate::defs::bsky::feed::defs::GeneratorView, FeedCursorPair)>,
    pub post_highlight: (usize, f32, bool),
}

fn ease_out_cubic(x: f32) -> f32 {
    return 1.0 - f32::powf(1.0 - x, 3.0);
}

impl FrontendTimelineView {
    pub fn new(feeds: Vec<GeneratorView>) -> Self {
        let mut feeds_dest: Vec<(crate::defs::bsky::feed::defs::GeneratorView, FeedCursorPair)> = Vec::new();
        for feed in feeds {
            feeds_dest.push((feed, FeedCursorPair { cursor: Some(String::new()), feed: Vec::new() }));
        }
        Self { timeline: FeedCursorPair { cursor: Some(String::new()), feed: Vec::new() }, feed: 0, feeds: feeds_dest, post_highlight: (0, 999.999, false) }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, you: &Option<ProfileViewDetailed>, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> ViewStackReturnInfo {
        puffin::profile_function!();
        let top = ui.cursor().top(); // the top of the scroll rect, used to compare post positions for keyboard nav
        let offset = ui.ctx().animate_bool_with_time_and_easing("FrontendMainViewStackTitleSlide".into(), true, 0.5, ease_out_cubic);
        let pos = pos2(120.0 + (100.0 - (offset * 100.0)), ui.cursor().top() - 40.0);

        let name = if self.feed >= 1
            && let Some(feed) = self.feeds.get(self.feed - 1)
        {
            format!("{} \u{E09D}", feed.0.display_name)
        } else {
            "Timeline \u{E09D}".to_owned()
        };

        let galley = ui.painter().layout(name, FontId::new(40.0, egui::FontFamily::Name("Segoe Light".into())), Color32::PLACEHOLDER, ui.cursor().width());
        let feed_swapper = ui.interact(Rect { min: pos2(pos.x, pos.y - galley.rect.height()), max: pos2(pos.x + galley.rect.width(), pos.y) }, egui::Id::new("TitleInteract"), egui::Sense::click());
        let mult = if feed_swapper.hovered() {
            if feed_swapper.is_pointer_button_down_on() {
                BSKY_BLUE.linear_multiply(0.25)
            } else {
                BSKY_BLUE.linear_multiply(0.5)
            }
        } else {
            BSKY_BLUE
        };
        ui.painter().galley(pos - vec2(0.0, galley.rect.height()), galley, mult);

        click_context_menu::click_context_menu(feed_swapper, |ui| {
            let following_response = ui.add(egui::Button::image_and_text(ImageSource::Texture(SizedTexture { id: egui::TextureId::Managed(0), size: vec2(40.0, 40.0) }), "Timeline").min_size(ui.spacing().interact_size));

            let tl_image_rect = Rect { min: following_response.rect.min + ui.spacing().button_padding, max: following_response.rect.min + ui.spacing().button_padding + vec2(40.0, 40.0) };
            ui.painter().rect_filled(tl_image_rect, Rounding::ZERO, BSKY_BLUE);
            ui.painter().text(tl_image_rect.center(), Align2::CENTER_CENTER, "\u{E1A6}", FontId::new(30.0, egui::FontFamily::Name("Segoe Symbols".into())), Color32::WHITE);
            if following_response.clicked() {
                self.feed = 0;
            }

            for (i, feed) in self.feeds.iter().enumerate() {
                let response = ui.add(egui::Button::image_and_text(ImageSource::Texture(SizedTexture { id: egui::TextureId::Managed(0), size: vec2(40.0, 40.0) }), feed.0.display_name.clone()).min_size(ui.spacing().interact_size));
                if response.clicked() {
                    self.feed = i + 1;
                }

                let image_rect = Rect { min: response.rect.min + ui.spacing().button_padding, max: response.rect.min + ui.spacing().button_padding + vec2(40.0, 40.0) };
                ui.painter().rect_filled(image_rect, Rounding::ZERO, BSKY_BLUE);
                if let Some(avatar) = &feed.0.avatar {
                    match image.get_image(&avatar) {
                        crate::image::LoadableImage::Unloaded | crate::image::LoadableImage::Loading => {}
                        crate::image::LoadableImage::Loaded(texture_id, _) => {
                            ui.painter().image(texture_id, image_rect, Rect::from_two_pos(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                        }
                    }
                }
            }
        });

        ScrollArea::vertical().hscroll(false).max_width(ui.cursor().width()).id_salt(self.feed).max_height(ui.cursor().height()).show(ui, |tl| {
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
            let iter = if self.feed == 0 { self.timeline.feed.iter().enumerate() } else { self.feeds.get(self.feed - 1).unwrap().1.feed.iter().enumerate() };
            for (i, post) in iter {
                puffin::profile_scope!("Post");

                let res = viewers::feed_post::feed_post_viewer(tl, &post, backend, image, flyout, new_view);
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
                let mut jank = String::new();
                let tl: &mut FeedCursorPair = if self.feed == 0 {
                    &mut self.timeline
                } else {
                    jank = self.feeds.get(self.feed - 1).unwrap().0.uri.clone();
                    &mut self.feeds.get_mut(self.feed - 1).unwrap().1
                };
                if spinner.is_rect_visible(spinner_rect) && tl.cursor.is_some() {
                    if self.feed == 0 {
                        backend.backend_commander.send(crate::bridge::FrontToBackMsg::GetTimelineRequest(tl.cursor.clone(), None)).unwrap();
                    } else {
                        backend.backend_commander.send(crate::bridge::FrontToBackMsg::GetFeedRequest(jank, tl.cursor.clone(), None)).unwrap();
                    }

                    tl.cursor = None;
                }
            })
        });

        let profile_pos = ui.ctx().screen_rect().right_top() + vec2(-80.0, 80.0);
        let search_pos = profile_pos - vec2(50.0, 0.0);
        let compose_pos = search_pos - vec2(50.0, 0.0);
        let refresh_pos = compose_pos - vec2(50.0, 0.0);

        let profile_button = ui.allocate_rect(Rect::from_center_size(profile_pos, vec2(30.0, 30.0)), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        let _search_button = ui.allocate_rect(Rect::from_center_size(search_pos, vec2(30.0, 30.0)), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        let compose_button = ui.allocate_rect(Rect::from_center_size(compose_pos, vec2(30.0, 30.0)), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        let refresh_button = ui.allocate_rect(Rect::from_center_size(refresh_pos, vec2(30.0, 30.0)), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);

        ui.painter().text(search_pos, Align2::CENTER_CENTER, "\u{E11A}", FontId::new(30.0, egui::FontFamily::Name("Segoe Symbols".into())), BSKY_BLUE);
        ui.painter().text(compose_pos, Align2::CENTER_CENTER, "\u{E104}", FontId::new(30.0, egui::FontFamily::Name("Segoe Symbols".into())), BSKY_BLUE);
        ui.painter().text(refresh_pos, Align2::CENTER_CENTER, "\u{E0F2}", FontId::new(30.0, egui::FontFamily::Name("Segoe Symbols".into())), BSKY_BLUE);

        if let Some(you) = &you {
            if let Some(pfp) = &you.avatar {
                match image.get_image(pfp) {
                    crate::image::LoadableImage::Loaded(texture_id, _) => {
                        ui.painter().image(texture_id, profile_button.rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                    }
                    _ => {
                        ui.painter().rect_filled(profile_button.rect, Rounding::ZERO, BSKY_BLUE);
                        ui.painter().text(profile_pos + vec2(0.0, 4.0), Align2::CENTER_CENTER, "\u{E2AF}", FontId::new(30.0, egui::FontFamily::Name("Segoe Symbols".into())), Color32::WHITE);
                    }
                }
            }

            if profile_button.clicked {
                new_view.set(FrontendMainView::Profile(FrontendProfileView::new(you.did.clone())));
            }
        } else {
            ui.painter().rect_filled(profile_button.rect, Rounding::ZERO, BSKY_BLUE);
            ui.painter().text(profile_pos + vec2(0.0, 4.0), Align2::CENTER_CENTER, "\u{E2AF}", FontId::new(30.0, egui::FontFamily::Name("Segoe Symbols".into())), Color32::WHITE);
        }

        if compose_button.clicked() {
            flyout.set(crate::frontend::main::ClientFrontendFlyoutVariant::PostComposerFlyout(ComposerFlyout::new()));
        }

        if refresh_button.clicked() {
            let feed = if self.feed == 0 { &mut self.timeline } else { &mut self.feeds.get_mut(self.feed - 1).unwrap().1 };

            feed.cursor = Some(String::new());
            feed.feed.clear();
        }

        ViewStackReturnInfo { title: None, render_back_button: true, handle_back_logic: true, force_back: false }
    }
}
