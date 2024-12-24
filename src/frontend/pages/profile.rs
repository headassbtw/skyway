use std::sync::{Arc, Mutex};

use egui::{pos2, vec2, Align, Align2, Color32, FontId, Id, Layout, Rect, Rounding, ScrollArea, Ui, UiBuilder};
use puffin::profile_scope;

use crate::bridge::FrontToBackMsg;
use crate::defs::bsky::actor::defs::ProfileViewDetailed;
use crate::defs::bsky::feed::defs::{FeedCursorPair, FeedViewPost};
use crate::frontend::main::ClientFrontendFlyout;
use crate::BSKY_BLUE;
use crate::frontend::viewers::feed_post::feed_post_viewer;
use crate::image::LoadableImage;
use crate::widgets::spinner::SegoeBootSpinner;
use crate::{bridge::Bridge, image::ImageCache};

use super::MainViewProposition;
#[derive(Debug)]
pub struct FrontendProfileView {
    pub profile_data: Option<ProfileViewDetailed>,
    pub id_cmp: String,
    pub loading: bool,
    pub posts: Option<Arc<Mutex<FeedCursorPair>>>,
    ctx: Option<egui::Context>,
    id: Id,
}

// E13D NONE
// E1E2 PLUS
// E181 STOP
// E1E0 BLOCK

impl FrontendProfileView {
    pub fn new(did: String) -> Self {
        Self {
            profile_data: None,
            id_cmp: did.clone(),
            loading: false,
            posts: None,
            ctx: None,
            id: Id::new(format!("{}_profile_scrollview", did)),
        }
    }
    pub fn render(&mut self, ui: &mut Ui, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> (&str, bool) {
        puffin::profile_function!();
        ui.style_mut().spacing.scroll.floating = false;
        ui.style_mut().spacing.scroll.bar_width = 18.0;

        if self.ctx.is_none() {
            self.ctx = Some(ui.ctx().clone());
        }
        if let Some(profile) = &self.profile_data {
            let right_pad = ui.ctx().screen_rect().width() - ui.cursor().right();
            let title_pos = pos2(ui.cursor().left(), ui.cursor().top() - 40.0);
            let funny_rect = ui.cursor().with_max_x(ui.ctx().screen_rect().right()).with_min_x(ui.ctx().screen_rect().left()).with_max_y(ui.ctx().screen_rect().bottom());
            let height = funny_rect.height() - funny_rect.top();
            let panel_height = (height - (8.0)) * (1.0 / 3.0);
            let offset_left = ui.cursor().left() - 4.0;
            let mut who_gaf = ui.child_ui(funny_rect, Layout::left_to_right(egui::Align::Min), None);
            who_gaf.style_mut().spacing.item_spacing = vec2(4.0, 4.0);

            let title_fontid = FontId::new(40.0, egui::FontFamily::Name("Segoe Light".into()));
            ui.painter().text(title_pos, Align2::LEFT_BOTTOM, "Profile", title_fontid.clone(), BSKY_BLUE);
            self.id = ScrollArea::horizontal().id_salt(format!("{}_profile_scrollview", profile.did)).vscroll(false).max_width(funny_rect.width()).max_height(funny_rect.height()).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible).show(&mut who_gaf, |ui| {
                ui.allocate_space(vec2(offset_left, funny_rect.height()));

                ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                    let (_, rect0) = ui.allocate_space(vec2(height * 1.5, (panel_height * 2.0) + ui.style().spacing.item_spacing.y));
                    let (_, rect1) = ui.allocate_space(vec2(height * 1.5, panel_height));

                    if let Some(pfp) = &profile.banner {
                        match image.get_image(pfp) {
                            LoadableImage::Unloaded | LoadableImage::Loading => {
                                ui.painter().rect_filled(rect0, Rounding::ZERO, Color32::GRAY);
                                SegoeBootSpinner::new().size(200.0).color(Color32::from_black_alpha(64)).paint_at(ui, rect0);
                            }
                            LoadableImage::Loaded(texture_id, size) => {
                                let ratio_src = size.x / size.y;
                                let ratio_dst = rect0.width() / rect0.height();
                                if ratio_dst < 1.0 {
                                    todo!()
                                }

                                let offset = (ratio_dst / ratio_src) * 0.5;

                                let uv_rect = Rect { min: pos2(0.5 - offset, 0.0), max: pos2(0.5 + offset, 1.0) };
                                ui.painter().image(texture_id, rect0, uv_rect, Color32::GRAY);
                            }
                        }
                    } else {
                        ui.painter().rect_filled(rect0, Rounding::ZERO, ui.style().visuals.extreme_bg_color);
                    }

                    let upper_center = rect0.center() - vec2(0.0, rect0.height() * (0.5 / 3.0));
                    let lower_center = rect0.center() + vec2(0.0, rect0.height() * (0.5 / 3.0));

                    let text_size = rect0.height() * (0.25 / 4.5);
                    let text_color = if profile.banner.is_some() {
                        Color32::WHITE
                    } else {
                        if ui.visuals().dark_mode {
                            Color32::WHITE
                        } else {
                            ui.visuals().text_color()
                        }
                    };

                    if let Some(dn) = profile.display_name() {
                        ui.painter().text(lower_center - vec2(0.0, text_size), Align2::CENTER_BOTTOM, dn, FontId::proportional(text_size), text_color);
                        ui.painter().text(lower_center + vec2(0.0, text_size), Align2::CENTER_BOTTOM, format!("@{}", profile.handle), FontId::proportional(text_size), text_color);
                    } else {
                        ui.painter().text(lower_center, Align2::CENTER_BOTTOM, format!("@{}", profile.handle), FontId::proportional(text_size), text_color);
                    }

                    let pfp_rect = Rect::from_center_size(upper_center, vec2(rect0.height() * 0.25, rect0.height() * 0.25));
                    if let Some(avatar) = &profile.avatar {
                        match image.get_image(avatar) {
                            LoadableImage::Unloaded | LoadableImage::Loading => {
                                ui.painter().rect_filled(pfp_rect, Rounding::ZERO, BSKY_BLUE);
                                SegoeBootSpinner::new().size(40.0).color(Color32::WHITE).paint_at(ui, pfp_rect);
                            }
                            LoadableImage::Loaded(texture_id, _) => {
                                ui.painter().image(texture_id, pfp_rect, Rect { min: pos2(0.0, 0.0), max: pos2(1.0, 1.0) }, Color32::WHITE);
                            }
                        }
                    } else {
                        ui.painter().rect_filled(pfp_rect, Rounding::ZERO, BSKY_BLUE);
                        ui.painter().text(pfp_rect.center(), Align2::CENTER_CENTER, "", FontId::new(rect0.height() * 0.2, egui::FontFamily::Name("Segoe Symbols".into())), Color32::WHITE);
                    }

                    if let Some(bio) = &profile.description {
                        ui.painter().rect_filled(rect1, Rounding::ZERO, ui.style().visuals.extreme_bg_color);
                        let mut bio_ui = ui.new_child(UiBuilder::new().layout(Layout::left_to_right(egui::Align::Max)).max_rect(rect1.shrink(8.0)));
                        bio_ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        bio_ui.label(bio);
                    }
                });
                let big_text_size = panel_height / 5.0;
                let small_text_size = big_text_size / 2.5;
                let mut scroll_to_posts = false;
                ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                    if let Some(posts_count) = &profile.posts_count {
                        let button = ui.allocate_response(vec2(height * 0.5, panel_height), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
                        ui.painter().rect_filled(button.rect, Rounding::ZERO, ui.style().visuals.extreme_bg_color);

                        ui.painter().text(button.rect.center() - vec2(0.0, big_text_size / 4.0), Align2::CENTER_BOTTOM, posts_count, FontId::proportional(big_text_size), ui.style().visuals.text_color());
                        ui.painter().text(button.rect.center() + vec2(0.0, big_text_size * 1.2), Align2::CENTER_TOP, "Posts", FontId::proportional(small_text_size), ui.style().visuals.text_color());

                        if button.clicked() { scroll_to_posts = true; println!("goes hard"); }
                    }

                    ui.disable();
                    if let Some(follows_count) = &profile.follows_count {
                        let button = ui.allocate_response(vec2(height * 0.5, panel_height), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
                        ui.painter().rect_filled(button.rect, Rounding::ZERO, ui.style().visuals.extreme_bg_color);

                        let text = if let Some(viewer) = &profile.viewer {
                            if viewer.followed_by.is_some() {
                                "Following (inc. You!)"
                            } else { "Following" }
                        } else { "Following" };

                        ui.painter().text(button.rect.center() - vec2(0.0, big_text_size / 4.0), Align2::CENTER_BOTTOM, follows_count, FontId::proportional(big_text_size), ui.style().visuals.text_color());
                        ui.painter().text(button.rect.center() + vec2(0.0, big_text_size * 1.2), Align2::CENTER_TOP, text, FontId::proportional(small_text_size), ui.style().visuals.text_color());
                    }

                    if let Some(followers_count) = &profile.followers_count {
                        let button = ui.allocate_response(vec2(height * 0.5, panel_height), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
                        ui.painter().rect_filled(button.rect, Rounding::ZERO, ui.style().visuals.extreme_bg_color);

                        let text = if let Some(viewer) = &profile.viewer {
                            if viewer.following.is_some() {
                                "Followers (inc. You)"
                            } else { "Followers" }
                        } else { "Followers" };

                        ui.painter().text(button.rect.center() - vec2(0.0, big_text_size / 4.0), Align2::CENTER_BOTTOM, followers_count, FontId::proportional(big_text_size), ui.style().visuals.text_color());
                        ui.painter().text(button.rect.center() + vec2(0.0, big_text_size * 1.2), Align2::CENTER_TOP, text, FontId::proportional(small_text_size), ui.style().visuals.text_color());
                    }
                });
                ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                    let did_lookup_button = ui.allocate_response(vec2(height * 0.5, panel_height), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
                    ui.painter().rect_filled(did_lookup_button.rect, Rounding::ZERO, ui.style().visuals.extreme_bg_color);

                    ui.painter().text(did_lookup_button.rect.center() - vec2(0.0, big_text_size / 4.0), Align2::CENTER_BOTTOM, "\u{E11A}", FontId::new(big_text_size, egui::FontFamily::Name("Segoe Symbols".into())), ui.style().visuals.text_color());
                    ui.painter().text(did_lookup_button.rect.center() + vec2(0.0, big_text_size * 1.2), Align2::CENTER_TOP, "Lookup DID", FontId::proportional(small_text_size), ui.style().visuals.text_color());
                    if did_lookup_button.clicked() {
                        let url = format!("https://web.plc.directory/did/{}", &profile.did);

                        #[cfg(target_os = "linux")]
                        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("cmd.exe").arg("/C").arg("start").arg(url).spawn();
                    }
                    ui.disable();
                    if let Some(viewer) = &profile.viewer { profile_scope!("Follow Button");
                        let button = ui.allocate_response(vec2(height * 0.5, panel_height), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
                        ui.painter().rect_filled(button.rect, Rounding::ZERO, ui.style().visuals.extreme_bg_color);

                        let (icon, text) = if viewer.following.is_some() {
                            if button.hovered() {
                                ("\u{E181}", "Unfollow")
                            } else {
                                ("\u{E13D}", "Following")
                            }
                        } else {
                            ("\u{E1E2}", "Follow")
                        };

                        ui.painter().text(button.rect.center() - vec2(0.0, big_text_size / 4.0), Align2::CENTER_BOTTOM, icon, FontId::new(big_text_size, egui::FontFamily::Name("Segoe Symbols".into())), ui.style().visuals.text_color());
                        ui.painter().text(button.rect.center() + vec2(0.0, big_text_size * 1.2), Align2::CENTER_TOP, text, FontId::proportional(small_text_size), ui.style().visuals.text_color());

                    }
                });
                ui.allocate_space(vec2(120.0, funny_rect.height()));
                
                let posts_res = ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
                    let pos = if ui.cursor().left() <= title_pos.x { title_pos } else { pos2(ui.cursor().left(), title_pos.y) };
                    ui.allocate_space(vec2(ui.ctx().screen_rect().width() - 240.0, 0.0));
                    ui.painter().rect_filled(Rect::from_two_pos(pos2(pos.x, pos.y-60.0), pos2(pos.x + 500.0, pos.y + 10.0)), Rounding::ZERO, ui.style().visuals.panel_fill);
                    ui.painter().text(pos, Align2::LEFT_BOTTOM, "Posts", title_fontid, BSKY_BLUE);
                    ui.style_mut().always_scroll_the_only_direction = false;
                    ScrollArea::vertical().max_width(ui.ctx().screen_rect().width() - offset_left).show(ui, |ui| {
                        if let Some(posts) = &self.posts {
                            let posts = posts.lock().unwrap();
                            for post in posts.feed.iter() {
                                feed_post_viewer(ui, post, &backend, image, flyout, new_view);
                            }
                        }

                        let loader_response = ui.add(SegoeBootSpinner::new().size(50.0).color(BSKY_BLUE));

                        if self.posts.is_none() && ui.is_rect_visible(loader_response.rect) {
                            let post_vec = Arc::new(Mutex::new( FeedCursorPair { cursor: None, feed: Vec::new() }));
                            self.posts = Some(post_vec.clone());
                            backend.backend_commander.send(FrontToBackMsg::GetAuthorFeedRequest(profile.did.clone(), "".into(), post_vec)).unwrap();
                        }
                    });
                });
                if scroll_to_posts { posts_res.response.scroll_to_me(Some(Align::TOP)); }
                ui.allocate_space(vec2(right_pad, funny_rect.height()));
            }).id;

        } else {
            SegoeBootSpinner::new().size(200.0).color(BSKY_BLUE).paint_at(ui, ui.ctx().screen_rect());
            if !self.loading {
                backend.backend_commander.send(FrontToBackMsg::GetProfileRequest(self.id_cmp.clone())).unwrap();
                self.loading = true;
            }
        }
        ("", true)
    }
}

impl Drop for FrontendProfileView {
    fn drop(&mut self) {
        if self.ctx.is_none() { return; }
        let ctx = self.ctx.clone().unwrap();
        ctx.data_mut(|map| {
            map.remove::<egui::scroll_area::State>(self.id);
        });
    }
}