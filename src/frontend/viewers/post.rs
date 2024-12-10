use std::sync::{Arc, Mutex};

use crate::{
    backend::record::{BlueskyApiRecordLike, BlueskyApiReplyRef, BlueskyApiStrongRef}, bridge::Bridge, defs::bsky::{embed, feed::defs::PostView}, frontend::{
        circle_button,
        flyouts::composer::ComposerFlyout,
        main::ClientFrontendFlyout,
        pages::{
            media::{image::FrontendMediaImageView, video::FrontendMediaVideoView, FrontendMediaViewVariant},
            profile::FrontendProfileView,
            thread::FrontendThreadView,
            FrontendMainView, MainViewProposition,
        },
    }, image::{ImageCache, LoadableImage}, open_in_browser, widgets::{click_context_menu, spinner::SegoeBootSpinner}
};

use chrono::{DateTime, Datelike, Timelike, Utc};
use egui::{
    pos2,
    text::{LayoutJob, TextWrapping},
    vec2, Align2, Color32, FontId, Layout, Rect, Response, Rounding, ScrollArea, Stroke, TextFormat, Ui, UiBuilder,
};

const BSKY_BLUE: Color32 = Color32::from_rgb(32, 139, 254);

fn offset_time(time: DateTime<Utc>) -> String {
    puffin::profile_function!();
    let offset = Utc::now() - time;
    if offset.num_days() >= 7 {
        //TODO: OS formatter
        return format!("{}:{} {}/{}/{}", time.hour(), time.minute(), time.month(), time.day(), time.year());
    } else if offset.num_hours() >= 24 {
        return format!("{}d", offset.num_days());
    } else if offset.num_minutes() >= 60 {
        return format!("{}h", offset.num_hours());
    } else if offset.num_seconds() >= 60 {
        return format!("{}m", offset.num_minutes());
    } else {
        return format!("{}s", offset.num_seconds());
    }
}

pub fn post_viewer(ui: &mut Ui, post: Arc<Mutex<PostView>>, _main: bool, backend: &Bridge, img_cache: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> Response {
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
        let tmp = ui.with_layout(Layout::top_down(egui::Align::Min), |post_contents| {
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
                let display_name = if let Some(display_name) = &post.author.display_name {
                    if display_name.len() > (0 as usize) {
                        // WHAT'S THE FUCKING POINT, BLUESKY???? IF YOU HAVE AN OPTIONAL FIELD, USE THAT FACT AND DON'T JUST RETURN BLANK
                        Some(name.add(egui::Label::new(egui::RichText::new(display_name).size(20.0).color(name.style().visuals.text_color())).selectable(false).sense(egui::Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand))
                    } else {
                        None
                    }
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
                let mut job = LayoutJob::default();
                job.wrap = TextWrapping::wrap_at_width(post_contents.cursor().width());
                let font_id = FontId::proportional(14.0);
                job.text = post.record.text.clone();

                /* if let Some(facets) = &post.record.facets {
                    let mut prev: usize = 0;
                    facets.sort_by(|a,b| {
                        a.index.byte_start.cmp(&b.index.byte_start)
                    });
                    for facet in facets {
                        let section = LayoutSection {
                            leading_space: 0.0,
                            byte_range: prev..(facet.index.byte_start),
                            format: TextFormat::simple(FontId::proportional(14.0), post_contents.style().visuals.noninteractive().fg_stroke.color),
                        };
                        job.sections.push(section);

                        let section = LayoutSection {
                            leading_space: 1.0,
                            byte_range: facet.index.byte_start..facet.index.byte_end,
                            format: TextFormat::simple(FontId::proportional(14.0), BSKY_BLUE),
                        };
                        job.sections.push(section);
                        prev = facet.index.byte_end;
                    }
                } else */
                {
                    job.append(&post.record.text, 0.0, TextFormat::simple(font_id, post_contents.style().visuals.noninteractive().fg_stroke.color));
                }

                let galley = post_contents.fonts(|f| f.layout_job(job));
                if post_contents.add(egui::Label::new(galley).sense(egui::Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    view_thread = true;
                }
            }

            //post_contents.painter().rect(post_contents.cursor(), Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(2.0, Color32::ORANGE));

            const MEDIA_SIZE: f32 = 180.0;

            if let Some(embed) = &post.embed {
                puffin::profile_scope!("Embed");
                match embed {
                    embed::Variant::Images { images } => 'render_images: {
                        puffin::profile_scope!("Images");
                        let img_rect = post_contents.cursor().with_max_y(post_contents.cursor().top() + MEDIA_SIZE);
                        if !post_contents.is_rect_visible(img_rect) {
                            puffin::profile_scope!("Image Short-Circuit");
                            post_contents.allocate_rect(img_rect, egui::Sense::click());
                            break 'render_images;
                        }
                        post_contents.allocate_new_ui(UiBuilder::default().max_rect(img_rect), |container| {
                            ScrollArea::horizontal().max_width(img_rect.width()).max_height(img_rect.height()).vscroll(false).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded).id_salt(&post.cid).show(container, |container| {
                                container.with_layout(Layout::left_to_right(egui::Align::Min), |container| {
                                    for img in images {
                                        if !container.is_visible() {
                                            continue;
                                        }
                                        puffin::profile_scope!("Image");
                                        let img_rect = match img_cache.get_image(&img.thumb) {
                                            LoadableImage::Unloaded | LoadableImage::Loading => {
                                                let x_multiplier = if let Some(ratio) = &img.aspect_ratio { ratio.width as f32 / ratio.height as f32 } else { 1.0 };
                                                let rtn = container.allocate_rect(container.cursor().with_max_x(container.cursor().left() + (MEDIA_SIZE * x_multiplier)), egui::Sense::click());
                                                container.painter().rect_filled(rtn.rect, Rounding::ZERO, Color32::GRAY);
                                                rtn
                                            }
                                            LoadableImage::Loaded(id, ratio) => {
                                                // kind of jank because sometimes the ratio won't send but it always does when loaded
                                                let x_multiplier = if let Some(ratio) = &img.aspect_ratio { ratio.width as f32 / ratio.height as f32 } else { ratio.x / ratio.y };
                                                let rtn = container.allocate_rect(container.cursor().with_max_x(container.cursor().left() + (MEDIA_SIZE * x_multiplier)), egui::Sense::click());
                                                container.painter().image(id, rtn.rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                                                rtn
                                            }
                                        };
                                        if img.alt.len() > 0 as usize {
                                            puffin::profile_scope!("Alt Text");
                                            let dim_rect = img_rect.rect.with_min_y(img_rect.rect.bottom() - 20.0);
                                            container.painter().rect_filled(dim_rect, Rounding::ZERO, Color32::from_black_alpha(128));

                                            container.painter().text(dim_rect.left_center() + vec2(10.0, 0.0), Align2::LEFT_CENTER, "ALT", FontId::proportional(12.0), Color32::WHITE);
                                            let alt_guh = container.allocate_rect(dim_rect, egui::Sense::click());
                                            alt_guh.on_hover_text(&img.alt);
                                        }
                                        if img_rect.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                            new_view.set(FrontendMainView::Media(FrontendMediaViewVariant::Image(FrontendMediaImageView::new(img.fullsize.clone()))));
                                            println!("image clicked!");
                                        }
                                    }
                                });
                            });
                        });
                    }
                    embed::Variant::Video(video) => 'render_video: {
                        puffin::profile_scope!("Video");
                        let video_rect = post_contents.cursor().with_max_y(post_contents.cursor().top() + MEDIA_SIZE);
                        if !post_contents.is_rect_visible(video_rect) {
                            puffin::profile_scope!("Video Short-Circuit");
                            post_contents.allocate_rect(video_rect, egui::Sense::click());
                            break 'render_video;
                        }
                        let ratio = if let Some(real_ratio) = &video.aspect_ratio { real_ratio.width as f32 / real_ratio.height as f32 } else { 16.0 / 9.0 };

                        let video_rect = video_rect.with_max_x(video_rect.left() + MEDIA_SIZE * ratio);

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
                            let rect = link.allocate_space(vec2(link.cursor().width(), 2.0)).1;
                            link.painter().rect_filled(rect, Rounding::ZERO, link.visuals().weak_text_color());
                            link.add(egui::Label::new(&external.uri).selectable(false));
                        }).response.interact(egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);

                        post_contents.painter().rect(resp.rect.expand(4.0), Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(2.0, post_contents.visuals().weak_text_color()));

                        if resp.clicked() {
                            open_in_browser(&external.uri);
                        }
                    }
                    embed::Variant::Record { record } => {
                        puffin::profile_scope!("Record");
                        let resp = post_contents.allocate_new_ui(UiBuilder::default().max_rect(post_contents.cursor().shrink(8.0)), |quote| {
                            quote.with_layout(Layout::left_to_right(egui::Align::Min), |embed| {
                                embed.horizontal_wrapped(|embed| match record {
                                    embed::record::View::Record(_) => {
                                        embed.weak("Record");
                                    }
                                    embed::record::View::NotFound(_) => {
                                        embed.weak("Not Found");
                                    }
                                    embed::record::View::Blocked(_) => {
                                        embed.weak("Blocked");
                                    }
                                    embed::record::View::Detached(_) => {
                                        embed.weak("Detached Record");
                                    }
                                    embed::record::View::FeedGenerator(_) => {
                                        embed.weak("Feed Generator");
                                    }
                                    embed::record::View::List(_) => {
                                        embed.weak("List");
                                    }
                                    embed::record::View::Labeler(_) => {
                                        embed.weak("Labeler");
                                    }
                                    embed::record::View::PackView(_) => {
                                        embed.weak("PackView");
                                    }
                                });
                            });
                        });

                        post_contents.painter().rect(resp.response.rect.expand(4.0), Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(2.0, post_contents.style().visuals.text_color()));
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
                        let reply = BlueskyApiReplyRef {
                            root: if let Some(reply) = &post.record.reply { reply.root.clone() } else { BlueskyApiStrongRef { uri: post.uri.clone(), cid: post.cid.clone() } },
                            parent: BlueskyApiStrongRef { uri: post.uri.clone(), cid: post.cid.clone() },
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
                    if guh.button(if rt_override.is_some() { "Un-Repost" } else { "Repost" }).clicked() {
                        repost = Some(rt_override.is_none());
                    }
                    if guh.add_enabled(false, egui::Button::new("Quote Repost")).clicked() {}
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
                    if guh.button("Open in browser").clicked() {
                        let id = post.uri.split("/").last().unwrap();
                        let handle = if post.author.handle.eq("handle.invalid") { &post.author.did } else { &post.author.handle };
                        let url = format!("https://bsky.app/profile/{}/post/{}", handle, id);

                        open_in_browser(&url);
                    }
                });
            });
        }); //post contents container
        tmp
    }); // main container including pfp
    ui.style_mut().spacing.item_spacing.y = 20.0;

    //ui.allocate_space(vec2(0.0, 0.0)); // weird hack because spacing doesn't apply i guess?

    if let Some(repost) = repost {
        if repost {
            let record = crate::backend::record::BlueskyApiRecord::Repost(BlueskyApiRecordLike { subject: BlueskyApiStrongRef { uri: post.uri.clone(), cid: post.cid.clone() }, created_at: Utc::now() });
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
            let record = crate::backend::record::BlueskyApiRecord::Like(BlueskyApiRecordLike { subject: BlueskyApiStrongRef { uri: post.uri.clone(), cid: post.cid.clone() }, created_at: Utc::now() });
            backend.backend_commander.send(crate::bridge::FrontToBackMsg::CreateRecordUnderPostRequest(record, post_og.clone())).unwrap();
        } else {
            if let Some(viewer) = &post.viewer {
                if let Some(viewer_like) = &viewer.like {
                    backend.backend_commander.send(crate::bridge::FrontToBackMsg::DeleteRecordUnderPostRequest(viewer_like.split("/").last().unwrap().to_owned(), "app.bsky.feed.like".to_owned(), post_og.clone())).unwrap();
                }
            }
        }
    }

    //ui.painter().rect_filled(ffs.response.rect, Rounding::ZERO, Color32::from_rgba_premultiplied(255, 0, 0, 128));

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
