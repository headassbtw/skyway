use std::sync::{Arc, Mutex};

use crate::{
    BSKY_BLUE,
    backend::record::BlueskyApiRecordLike, bridge::Bridge, defs::bsky::{embed, feed::{defs::PostView, ReplyRef, StrongRef}}, frontend::{
        circle_button,
        flyouts::composer::ComposerFlyout,
        main::ClientFrontendFlyout,
        pages::{
            media::{video::FrontendMediaVideoView, FrontendMediaViewVariant},
            profile::FrontendProfileView,
            thread::FrontendThreadView,
            FrontendMainView, MainViewProposition,
        }, viewers::{embeds::{images::view_images, record::view_record}, offset_time},
    }, image::{ImageCache, LoadableImage}, open_in_browser, widgets::{click_context_menu, spinner::SegoeBootSpinner}
};

use chrono::Utc;
use egui::{
    pos2, text::{LayoutJob, LayoutSection, TextWrapping}, vec2, Align2, Button, Color32, FontId, Id, Layout, Rect, Response, Rounding, Stroke, TextFormat, Ui, UiBuilder
};

pub fn post_viewer(ui: &mut Ui, post: Arc<Mutex<PostView>>, main: bool, backend: &Bridge, img_cache: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> Response {
    puffin::profile_function!();
    let post_og = post.clone();
    let mut like: Option<bool> = None;
    let mut repost: Option<bool> = None;
    let mut view_thread = false;
    let mut view_profile = false;
    let post = {
        puffin::profile_scope!("Mutex Lock");
        &post_og.lock().unwrap()
    };
    ui.style_mut().spacing.item_spacing.y = 40.0;

    let ffs = ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
        puffin::profile_scope!("Main Container");
        ui.style_mut().spacing.item_spacing = vec2(10.0, 10.0);
        let pfp_response = ui.allocate_response(vec2(60.0, 60.0), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        let pfp_rect = pfp_response.rect;
        if ui.is_rect_visible(pfp_rect) {
            if let Some(avatar) = &post.author.avatar {
                match img_cache.get_image(avatar) {
                    LoadableImage::Unloaded => {
                        ui.painter().rect_filled(pfp_rect, Rounding::ZERO, Color32::RED);
                        SegoeBootSpinner::new().size(40.0).color(Color32::WHITE).paint_at(ui, pfp_rect);
                    }
                    LoadableImage::Loading => {
                        ui.painter().rect_filled(pfp_rect, Rounding::ZERO, BSKY_BLUE);
                        SegoeBootSpinner::new().size(40.0).color(Color32::WHITE).paint_at(ui, pfp_rect);
                    }
                    LoadableImage::Loaded(texture_id, _) => {
                        ui.painter().image(texture_id, pfp_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                    }
                }
            } else {
                ui.painter().rect_filled(pfp_rect, Rounding::ZERO, BSKY_BLUE);
                ui.painter().text(pfp_rect.center(), Align2::CENTER_CENTER, "", FontId::new(50.0, egui::FontFamily::Name("Segoe Symbols".into())), Color32::WHITE);
            }
        }
        ui.with_layout(Layout::top_down(egui::Align::Min), |post_contents| {
            let the_width_you_care_about = post_contents.cursor().width();
            post_contents.set_max_width(the_width_you_care_about);
            post_contents.allocate_new_ui(UiBuilder::new().layout(Layout::left_to_right(egui::Align::TOP)), |name| 'render_name: {
                puffin::profile_scope!("Name");
                if !name.is_visible() {
                    break 'render_name;
                }
                let guh_fr = name.painter().layout_no_wrap(offset_time(post.record.created_at), FontId::new(16.0, egui::FontFamily::Name("Segoe Light".into())), Color32::DARK_GRAY);
                name.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                name.set_width(the_width_you_care_about - (20.0 + guh_fr.mesh_bounds.width()));
                let display_name = if let Some(display_name) = &post.author.display_name && display_name.len() > 0{
                    Some(name.add(egui::Label::new(egui::RichText::new(display_name).size(20.0).color(name.style().visuals.text_color())).selectable(false).sense(egui::Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand))
                } else {
                    None
                };
                let name_res = if display_name.is_none() { name.add(egui::Label::new(egui::RichText::new(&post.author.handle).size(20.0)).selectable(false).sense(egui::Sense::click())) } else { name.add(egui::Label::new(egui::RichText::new(&post.author.handle).weak().font(FontId::new(20.0, egui::FontFamily::Name("Segoe Light".into()))).size(20.0)).selectable(false).sense(egui::Sense::click())) }.on_hover_cursor(egui::CursorIcon::PointingHand);

                let click_response = if let Some(dn) = display_name {
                    if dn.hovered() {
                        dn.clicked()
                    } else {
                        name_res.clicked()
                    }
                } else {
                    name_res.clicked()
                } || pfp_response.clicked();

                if click_response {
                    view_profile = true;
                }

                name.painter().galley(pos2(name_res.rect.right() + 20.0, name_res.rect.right_center().y - guh_fr.mesh_bounds.height() / 2.0), guh_fr, Color32::GREEN);
                name.painter().circle_filled(pos2(name_res.rect.right() + 10.0, name_res.rect.right_center().y + 5.0), 2.0, Color32::DARK_GRAY);
            });
            if post.record.text.len() > (0 as usize) {
                puffin::profile_scope!("Text");
                let font_id = FontId::proportional(if main { 20.0 } else { 14.0 });

                // This kinda sucks but it works!
                if let Some(facets) = &post.record.facets {
                    post_contents.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        ui.style_mut().visuals.override_text_color = Some(ui.visuals().noninteractive().fg_stroke.color);

                        let mut prev: usize = 0;
                        for (idx, facet) in facets.iter().enumerate() {

                            if prev < facet.index.byte_start {
                                ui.add(egui::Label::new(egui::RichText::new(&post.record.text[prev..facet.index.byte_start-1]).font(font_id.clone())));
                            }

                            if ui.link(egui::RichText::new(&post.record.text[facet.index.byte_start..facet.index.byte_end]).color(BSKY_BLUE).font(font_id.clone())).clicked() {
                                for feature in &facet.features {
                                    match feature {
                                        crate::defs::bsky::richtext::Feature::Mention(mention) => { new_view.set(FrontendMainView::Profile(FrontendProfileView::new(mention.did.clone()))); },
                                        crate::defs::bsky::richtext::Feature::Link(link) => { open_in_browser(&link.uri); },
                                        crate::defs::bsky::richtext::Feature::Tag(_) => {},
                                    }
                                }
                            }

                            if idx == facets.len() - 1 { // last facet, write the rest of the text...
                                if &post.record.text.len() >= &facet.index.byte_end { // ...if applicable
                                    ui.add(egui::Label::new(egui::RichText::new(&post.record.text[facet.index.byte_end..post.record.text.len()]).font(font_id.clone())));
                                }
                            }

                            prev = facet.index.byte_end;
                        }
                    });
                } else {
                    post_contents.add(egui::Label::new(egui::RichText::new(&post.record.text).color(post_contents.visuals().noninteractive().fg_stroke.color).font(font_id)));
                }

            }

            let media_size: f32 = if main { 240.0 } else { 180.0 };

            let embed_enabled = if let Some(opt) = &post.viewer {
                if let Some(en) = opt.embedding_disabled {
                    if en {
                        &None
                    } else { &post.embed }
                } else { &post.embed }
            } else { &post.embed };

            if let Some(embed) = embed_enabled {
                puffin::profile_scope!("Embed");
                match embed {
                    embed::Variant::Images { images } => {
                        view_images(post_contents, Id::new(&post.cid), images, media_size, img_cache, new_view);
                    }
                    embed::Variant::Video(video) => 'render_video: {
                        puffin::profile_scope!("Video");
                        let video_rect = post_contents.cursor().with_max_y(post_contents.cursor().top() + media_size);
                        if !post_contents.is_rect_visible(video_rect) {
                            puffin::profile_scope!("Video Short-Circuit");
                            post_contents.allocate_rect(video_rect, egui::Sense::click());
                            break 'render_video;
                        }
                        let ratio = if let Some(real_ratio) = &video.aspect_ratio { real_ratio.width as f32 / real_ratio.height as f32 } else { 16.0 / 9.0 };

                        let video_rect = video_rect.with_max_x(video_rect.left() + media_size * ratio);

                        let tex = if let Some(thumb) = &video.thumbnail {
                            match img_cache.get_image(thumb) {
                                LoadableImage::Unloaded | LoadableImage::Loading => None,
                                LoadableImage::Loaded(texture_id, _) => Some(texture_id),
                            }
                        } else {
                            None
                        };

                        if let Some(id) = tex {
                            post_contents.painter().image(id, video_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                        } else {
                            post_contents.painter().rect_filled(video_rect, Rounding::ZERO, Color32::BLACK);
                        }
                        post_contents.allocate_rect(video_rect, egui::Sense::click());
                        post_contents.painter().rect_filled(video_rect, Rounding::ZERO, Color32::from_white_alpha(64));
                        post_contents.painter().circle_filled(video_rect.center(), 15.0, Color32::from_black_alpha(128));
                        post_contents.painter().text(video_rect.center(), Align2::CENTER_CENTER, "\u{25B6}", FontId::new(20.0, egui::FontFamily::Name("Segoe Symbols".into())), BSKY_BLUE);

                        if post_contents.allocate_rect(video_rect, egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            new_view.set(FrontendMainView::Media(FrontendMediaViewVariant::Video(FrontendMediaVideoView {})));
                            println!("image clicked!");
                        }
                    }
                    embed::Variant::External { external } => {
                        let resp = post_contents.allocate_new_ui(UiBuilder::default().max_rect(post_contents.cursor().shrink(8.0)), |link| {
                            link.add(egui::Label::new(&external.title).selectable(false));
                            if external.description.len() > 0 {
                                link.add(egui::Label::new(&external.description).selectable(false));
                            }
                            let rtn = link.allocate_space(vec2(2.0, 2.0)).1;
                            
                            link.add(egui::Label::new(&external.uri).selectable(false));
                            rtn
                        });

                        post_contents.painter().rect(resp.response.rect.expand(4.0), Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(2.0, post_contents.visuals().weak_text_color()));

                        post_contents.painter().rect_filled(resp.inner.with_max_x(resp.response.rect.max.x), Rounding::ZERO, post_contents.visuals().weak_text_color());

                        if resp.response.interact(egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            open_in_browser(&external.uri);
                        }
                    }
                    embed::Variant::Record { record } => {
                        puffin::profile_scope!("Record");
                        view_record(post_contents, record, media_size, img_cache, new_view);
                        
                    }
                    embed::Variant::RecordWithMedia(aforementioned) => {
                        match &aforementioned.media {
                            embed::record_with_media::MediaVariant::Images { images } => {
                                view_images(post_contents, Id::new(&post.cid), images, media_size, img_cache, new_view);
                            },
                            embed::record_with_media::MediaVariant::Video(value) => {
                                post_contents.weak("Video");
                                post_contents.weak(format!("{:?}", value));
                            },
                            embed::record_with_media::MediaVariant::External(value) => {
                                post_contents.weak("Link");
                                post_contents.weak(format!("{:?}", value));
                            },
                        }
                        view_record(post_contents, &aforementioned.record.record, media_size * 0.8, img_cache, new_view);
                    }
                    _ => {
                        post_contents.weak("Unhandled embed");
                    }
                }
            }
            post_contents.style_mut().spacing.item_spacing.y = 10.0;
            post_contents.allocate_space(vec2(0.0, 0.0));
            post_contents.with_layout(Layout::left_to_right(egui::Align::Min), |action_buttons| 'render_action_buttons: {
                puffin::profile_scope!("Action Buttons");
                if post.viewer.is_none() {
                    break 'render_action_buttons; // if there's no viewer, you can't interact with it (for the most part) so don't bother
                }
                if !action_buttons.is_rect_visible(action_buttons.cursor().with_max_y(action_buttons.cursor().top() + 30.0)) {
                    action_buttons.allocate_space(vec2(0.0, 30.0));
                    break 'render_action_buttons;
                }

                action_buttons.style_mut().spacing.item_spacing.x = 26.0 * 3.0;

                let reply_enabled = if let Some(dis) = post.viewer.as_ref().unwrap().reply_disabled { dis } else { true };
                action_buttons.add_enabled_ui(reply_enabled, |action_buttons| {
                    let button = circle_button(action_buttons, "\u{E206}", 20.0, 15.0, None);
                    if button.clicked() {
                        let reply = ReplyRef {
                            root: if let Some(reply) = &post.record.reply { reply.root.clone() } else { StrongRef { uri: post.uri.clone(), cid: post.cid.clone() } },
                            parent: StrongRef { uri: post.uri.clone(), cid: post.cid.clone() },
                        };
                        flyout.set(crate::frontend::main::ClientFrontendFlyoutVariant::PostComposerFlyout(ComposerFlyout::with_reply(reply)));
                    }
                    if let Some(reply_count) = &post.reply_count {
                        if reply_count > &0u32 {
                            action_buttons.painter().text(button.rect.right_center() + vec2(12.0, -2.0), Align2::LEFT_CENTER, format!("{}", reply_count), FontId::proportional(15.0), action_buttons.style().interact(&button).fg_stroke.color);
                        }
                    }
                });

                let rt_override = if post.viewer.as_ref().unwrap().repost.is_some() { Some(Color32::from_rgb(92, 239, 170)) } else { None };
                let repost_button = circle_button(action_buttons, "\u{E207}", 20.0, 15.0, rt_override);
                if let Some(repost_count) = &post.repost_count {
                    if repost_count > &0u32 {
                        action_buttons.painter().text(repost_button.rect.right_center() + vec2(12.0, -2.0), Align2::LEFT_CENTER, format!("{}", repost_count), FontId::proportional(15.0), if let Some(col) = rt_override { col } else { action_buttons.style().interact(&repost_button).fg_stroke.color });
                    }
                }
                click_context_menu::click_context_menu(repost_button, |guh| {
                    guh.spacing_mut().item_spacing.y = 0.0;
                    if guh.add(Button::new(if rt_override.is_some() { "Un-Repost" } else { "Repost" }).min_size(vec2(280.0, 40.0))).clicked() {
                        repost = Some(rt_override.is_none());
                    }
                    if guh.add_enabled(false, Button::new("Quote Repost").min_size(vec2(280.0, 40.0))).clicked() { }
                });

                let like_override = if post.viewer.as_ref().unwrap().like.is_some() { Some(Color32::from_rgb(236, 72, 153)) } else { None };
                let like_button = circle_button(action_buttons, "\u{E209}", 20.0, 15.0, like_override);
                if like_button.clicked() {
                    like = Some(like_override.is_none());
                }
                if let Some(like_count) = &post.like_count {
                    if like_count > &0u32 {
                        action_buttons.painter().text(like_button.rect.right_center() + vec2(12.0, -2.0), Align2::LEFT_CENTER, format!("{}", like_count), FontId::proportional(15.0), if let Some(col) = like_override { col } else { action_buttons.style().interact(&like_button).fg_stroke.color });
                    }
                }

                click_context_menu::click_context_menu(circle_button(action_buttons, "\u{E0C2}", 15.0, 15.0, None), |guh| {
                    guh.spacing_mut().item_spacing.y = 0.0;
                    if guh.add(Button::new("Open in browser").min_size(vec2(280.0, 40.0))).clicked() {
                        let id = post.uri.split("/").last().unwrap();
                        let handle = if post.author.handle.eq("handle.invalid") { &post.author.did } else { &post.author.handle };
                        let url = format!("https://bsky.app/profile/{}/post/{}", handle, id);

                        open_in_browser(&url);
                    }

                    if guh.add_enabled(false, Button::new("Copy link").min_size(vec2(280.0, 40.0))).clicked() {
                    }
                });
            });
        }) //post contents container
    }); // main container including pfp
    ui.style_mut().spacing.item_spacing.y = 20.0;

    //ui.allocate_space(vec2(0.0, 0.0)); // weird hack because spacing doesn't apply i guess?

    if let Some(repost) = repost {
        if repost {
            let record = crate::backend::record::BlueskyApiRecord::Repost(BlueskyApiRecordLike { subject: StrongRef { uri: post.uri.clone(), cid: post.cid.clone() }, created_at: Utc::now() });
            backend.backend_commander.send(crate::bridge::FrontToBackMsg::CreateRecordUnderPostRequest(record, post_og.clone())).unwrap();
        } else {
            if let Some(viewer) = &post.viewer {
                if let Some(viewer_repost) = &viewer.repost {
                    backend.backend_commander.send(crate::bridge::FrontToBackMsg::DeleteRecordUnderPostRequest(viewer_repost.split("/").last().unwrap().to_owned(), "app.bsky.feed.repost".to_owned(), post_og.clone())).unwrap();
                }
            }
        }
    }

    if let Some(like) = like {
        if like {
            let record = crate::backend::record::BlueskyApiRecord::Like(BlueskyApiRecordLike { subject: StrongRef { uri: post.uri.clone(), cid: post.cid.clone() }, created_at: Utc::now() });
            backend.backend_commander.send(crate::bridge::FrontToBackMsg::CreateRecordUnderPostRequest(record, post_og.clone())).unwrap();
        } else {
            if let Some(viewer) = &post.viewer {
                if let Some(viewer_like) = &viewer.like {
                    backend.backend_commander.send(crate::bridge::FrontToBackMsg::DeleteRecordUnderPostRequest(viewer_like.split("/").last().unwrap().to_owned(), "app.bsky.feed.like".to_owned(), post_og.clone())).unwrap();
                }
            }
        }
    }

    if ffs.response.clicked() {
        println!("guh!");
    }

    if view_thread || ffs.response.interact(egui::Sense::click()).clicked() {
        new_view.set(FrontendMainView::Thread(FrontendThreadView::new(post.uri.clone())));
    }
    if view_profile {
        new_view.set(FrontendMainView::Profile(FrontendProfileView::new(post.author.did.clone())));
    }

    ffs.response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
