use std::{sync::{Arc, Mutex}};

use crate::{
    backend::record::BlueskyApiRecordLike, bridge::Bridge, defs::bsky::{embed, feed::{defs::PostView, ReplyRef, StrongRef}}, frontend::{
        flyouts::composer::ComposerFlyout,
        main::ClientFrontendFlyout,
        pages::{
            profile::FrontendProfileView,
            thread::FrontendThreadView,
            FrontendMainView, MainViewProposition,
        }, viewers::{embeds::{external::view_external, images::view_images, record::view_record, video::view_video}, offset_time},
    }, image::{ImageCache, LoadableImage}, open_in_browser, widgets::{click_context_menu, spinner::SegoeBootSpinner}, BSKY_BLUE
};

use chrono::Utc;
use egui::{
    pos2, vec2, Align2, Button, Color32, FontId, Id, Layout, Rect, Response, Rounding, Stroke, Ui, UiBuilder, WidgetRect
};

fn action_button(ui: &mut Ui, enabled: bool, pre_actioned: bool, size: f32, glyph: &str, count: usize, color: Option<Color32>) -> Response {
    let (id, rtn) = ui.allocate_space(vec2(size * 2.5 + ui.spacing().item_spacing.x, size));

    let color = {
        puffin::profile_scope!("Color logic");
        let highlight = if ui.style().visuals.dark_mode { Color32::WHITE } else { Color32::BLACK };
        let color = if pre_actioned { color.unwrap_or(highlight) } else { highlight };
        if !enabled { ui.visuals().weak_text_color() } else { color }
    };

    // TEXT
    let (galley, text_width) = if count > 0 {
        puffin::profile_scope!("Text");
        let galley = ui.painter().layout(count.to_string(), FontId::proportional(size / 2.0), color, size * 2.0);
        let width = galley.rect.width() + ui.spacing().item_spacing.x / 3.0;
        (Some(galley),
        width)
    } else { (None, 0.0) };

    // like from alan wake
    let clicker =ui.add_enabled_ui(enabled, |ui| {
        ui.interact(rtn.with_max_x(rtn.min.x + size + text_width), Id::new(id), egui::Sense::click())
    }).inner;
    

    {
        puffin::profile_scope!("Animation");

        let anim = ui.ctx().animate_bool(Id::new(id), clicker.hovered());
        let opacity = (anim * 16.0) as u8;
        ui.painter().rect_filled(clicker.rect.expand(4.0 * anim), Rounding::ZERO, if ui.style().visuals.dark_mode { Color32::from_white_alpha(opacity) } else { Color32::from_black_alpha(opacity * 2) });
    }

    // ICON
    let circle_center = {
        puffin::profile_scope!("Icon");
        let circle_center = rtn.min + vec2(size / 2.0, size / 2.0);
        ui.painter().circle(circle_center.clone(), size / 2.0 - 1.0, Color32::TRANSPARENT, Stroke::new(2.0, color));
        ui.painter().text(circle_center.clone() - vec2(0.0, 2.0), Align2::CENTER_CENTER, glyph, FontId::proportional(if glyph.eq("\u{E0C2}") { size / 2.2 } else { size * 2.0 / 3.0 }), color);
        circle_center
    };

    // TEXT (again)
    if let Some(galley) = galley {
        ui.painter().galley(pos2(rtn.min.x + size + ui.spacing().item_spacing.x / 3.0, circle_center.y - (galley.rect.height() / 2.0 + 2.0)), galley, color);
    }

    clicker
}

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
                // the culling here is really late and that annoys me but it's better than nothing
                let seglight = egui::FontFamily::Name("Segoe Light".into());
                let time_galley = name.painter().layout_no_wrap(offset_time(post.indexed_at), FontId::new(16.0, seglight.clone()), Color32::DARK_GRAY);
                name.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                let profile_click = if let Some(dn) = &post.author.display_name && dn.len() > 0 {
                    let dn_galley = name.painter().layout(dn.to_string(), FontId::proportional(20.0), name.style().visuals.text_color(), name.cursor().width() - (30.0 + time_galley.rect.width() + (name.spacing().item_spacing.x * 2.0)));
                    let handle_galley = name.painter().layout(post.author.handle.clone(), FontId::new(20.0, seglight.clone()), name.style().visuals.weak_text_color(), name.cursor().width() - (30.0 + dn_galley.rect.width() + time_galley.rect.width() + (name.spacing().item_spacing.x * 3.0)));

                    let rtn = name.allocate_response(vec2(dn_galley.rect.width() + handle_galley.rect.width() + name.spacing().item_spacing.x, f32::max(handle_galley.rect.height(), dn_galley.rect.height())), egui::Sense::click());
                    if !name.is_rect_visible(rtn.rect) { break 'render_name; }

                    name.painter().galley(rtn.rect.min + vec2(name.spacing().item_spacing.x + dn_galley.rect.width(), 0.0), handle_galley, name.style().visuals.weak_text_color());
                    name.painter().galley(rtn.rect.min, dn_galley, name.style().visuals.text_color());
                    rtn
                } else {
                    let handle_galley = name.painter().layout(post.author.handle.clone(), FontId::proportional(20.0), name.style().visuals.text_color(), name.cursor().width() - (30.0 + time_galley.rect.width() + (name.spacing().item_spacing.x * 2.0)));

                    let rtn = name.allocate_response(handle_galley.rect.size(), egui::Sense::click());
                    if !name.is_rect_visible(rtn.rect) { break 'render_name; }
                    
                    name.painter().galley(rtn.rect.min, handle_galley, name.style().visuals.text_color());
                    rtn
                };

                let dot_pos = profile_click.rect.max + vec2(name.spacing().item_spacing.x, profile_click.rect.height() * -0.3);
                name.painter().circle_filled(dot_pos.clone(), 2.0, Color32::DARK_GRAY);
                name.painter().galley(dot_pos + vec2(name.spacing().item_spacing.x, time_galley.rect.height() * -0.6), time_galley, Color32::DARK_GRAY);  

                if profile_click.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() || pfp_response.clicked() {
                    view_profile = true;
                }
                
            });
            if post.record.text.len() > (0 as usize) {
                puffin::profile_scope!("Text");
                let font_id = FontId::proportional(if main { 20.0 } else { 14.0 });

                // This kinda sucks but it works!
                if let Some(facets) = &post.record.facets && facets.len() > 0 {
                    puffin::profile_scope!("facets/richtext");
                    post_contents.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        ui.style_mut().visuals.override_text_color = Some(ui.visuals().noninteractive().fg_stroke.color);

                        let mut prev: usize = 0;
                        for (idx, facet) in facets.iter().enumerate() {
                            if prev < facet.index.byte_start {
                                if ui.add(egui::Label::new(egui::RichText::new(&post.record.text[prev..facet.index.byte_start-1]).font(font_id.clone()))).clicked() {
                                    view_thread = true;
                                }
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
                                    if ui.add(egui::Label::new(egui::RichText::new(&post.record.text[facet.index.byte_end..post.record.text.len()]).font(font_id.clone()))).clicked() {
                                        view_thread = true;
                                    }
                                }
                            }

                            prev = facet.index.byte_end;
                        }
                    });
                } else {
                    if post_contents.add(egui::Label::new(egui::RichText::new(&post.record.text).color(post_contents.visuals().noninteractive().fg_stroke.color).font(font_id))).clicked() {
                        view_thread = true;
                    }
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
                    embed::Variant::Video(video) => {
                        view_video(post_contents, video, media_size, img_cache, new_view);
                    }
                    embed::Variant::External { external } => {
                        view_external(post_contents, external, media_size, img_cache);
                    }
                    embed::Variant::Record { record } => {
                        view_record(post_contents, record, media_size, img_cache, new_view);
                    }
                    embed::Variant::RecordWithMedia(aforementioned) => {
                        match &aforementioned.media {
                            embed::record_with_media::MediaVariant::Images { images } => {
                                view_images(post_contents, Id::new(&post.cid), images, media_size, img_cache, new_view);
                            },
                            embed::record_with_media::MediaVariant::Video(video) => {
                                view_video(post_contents, video, media_size, img_cache, new_view);
                            },
                            embed::record_with_media::MediaVariant::External { external} => {
                                view_external(post_contents, external, media_size, img_cache);
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
                if post.viewer.is_none() {
                    puffin::profile_scope!("Action Buttons early exit 0");
                    break 'render_action_buttons; // if there's no viewer, you can't interact with it (for the most part) so don't bother
                }
                if !action_buttons.is_rect_visible(action_buttons.cursor().with_max_y(action_buttons.cursor().top() + 30.0)) {
                    puffin::profile_scope!("Action Buttons early exit 1");
                    action_buttons.allocate_space(vec2(0.0, 30.0));
                    break 'render_action_buttons;
                }

                puffin::profile_scope!("Action Buttons");

                action_buttons.style_mut().spacing.item_spacing.x = 30.0;

                {
                    puffin::profile_scope!("Reply Button");

                    let reply_enabled = if let Some(dis) = post.viewer.as_ref().unwrap().reply_disabled { dis } else { true };
                    let reply_count = post.reply_count.unwrap_or(0);
                    let reply_button = action_button(action_buttons, reply_enabled, false, 30.0, "\u{E206}", reply_count as usize, None);
                    if reply_button.clicked() {
                        let reply = ReplyRef {
                            root: if let Some(reply) = &post.record.reply { reply.root.clone() } else { StrongRef { uri: post.uri.clone(), cid: post.cid.clone() } },
                            parent: StrongRef { uri: post.uri.clone(), cid: post.cid.clone() },
                        };
                        flyout.set(crate::frontend::main::ClientFrontendFlyoutVariant::PostComposerFlyout(ComposerFlyout::with_reply(reply)));
                    }
                }
                {
                    puffin::profile_scope!("Repost Button");

                    let repost_count = post.repost_count.unwrap_or(0) + post.quote_count.unwrap_or(0);
                    let self_reposted = post.viewer.as_ref().unwrap().repost.is_some();
                    let repost_button = action_button(action_buttons, true, self_reposted, 30.0, "\u{E207}", repost_count as usize, Some(Color32::from_rgb(92, 239, 170)));

                    click_context_menu::click_context_menu(repost_button, |guh| {
                        guh.spacing_mut().item_spacing.y = 0.0;
                        if guh.add(Button::new(if self_reposted { "Un-Repost" } else { "Repost" }).min_size(guh.spacing().interact_size)).clicked() {
                            repost = Some(!self_reposted);
                        }
                        if guh.add_enabled(false, Button::new("Quote Repost").min_size(guh.spacing().interact_size)).clicked() { }
                    });
                }
                
                {
                    puffin::profile_scope!("Like Button");

                    let like_count = post.like_count.unwrap_or(0);
                    let self_liked = post.viewer.as_ref().unwrap().like.is_some();
                    if action_button(action_buttons, true, self_liked, 30.0, "\u{E209}", like_count as usize, Some(Color32::from_rgb(236, 72, 153))).clicked() {
                        like = Some(!self_liked);
                    }
                }

                {
                    puffin::profile_scope!("More Button");

                    click_context_menu::click_context_menu(action_button(action_buttons, true, false, 30.0, "\u{E0C2}", 0, None), |guh| {
                        guh.spacing_mut().item_spacing.y = 0.0;
                        if guh.add(Button::new("Open in browser").min_size(guh.spacing().interact_size)).clicked() {
                            let id = post.uri.split("/").last().unwrap();
                            let handle = if post.author.handle.eq("handle.invalid") { &post.author.did } else { &post.author.handle };
                            let url = format!("https://bsky.app/profile/{}/post/{}", handle, id);

                            open_in_browser(&url);
                        }

                        if guh.add_enabled(false, Button::new("Copy link").min_size(guh.spacing().interact_size)).clicked() {
                        }
                    });
                }

                // take up the remainder of the space, so that any thing to the right of the post is clickable
                //action_buttons.allocate_space(vec2(action_buttons.cursor().width(), 30.0));
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

    if !main && (view_thread || ffs.response.interact(egui::Sense::click()).clicked()) {
        new_view.set(FrontendMainView::Thread(FrontendThreadView::new(post.uri.clone())));
    }
    if view_profile {
        new_view.set(FrontendMainView::Profile(FrontendProfileView::new(post.author.did.clone())));
    }

    ffs.response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
