use std::sync::{Arc, Mutex};

use crate::{
    backend::{
        record::{BlueskyApiRecordLike, BlueskyApiReplyRef, BlueskyApiStrongRef},
        responses::timeline::{
            embed::{BlueskyApiTimelineEmbedRecordView, BlueskyApiTimelinePostEmbedView},
            reason::BlueskyApiTimelineReason,
            reply::BlueskyApiTimelineReasonReply,
            BlueskyApiTimelineResponseObject,
        },
    },
    bridge::Bridge,
    frontend::{
        circle_button,
        flyouts::composer::ComposerFlyout,
        main::ClientFrontendFlyout,
        pages::{profile::FrontendProfileView, thread::FrontendThreadView, FrontendMainView, FrontendMainViewStack, MainViewProposition},
    },
    image::{ImageCache, LoadableImage},
    widgets::click_context_menu,
};
use chrono::{DateTime, Datelike, Timelike, Utc};
use egui::{
    pos2,
    text::{LayoutJob, TextWrapping},
    vec2, Align2, Color32, FontId, Layout, Rect, Response, Rounding, ScrollArea, Stroke, TextFormat, TextureId, Ui, UiBuilder,
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

pub fn post_viewer(ui: &mut Ui, post: Arc<Mutex<BlueskyApiTimelineResponseObject>>, backend: &Bridge, img_cache: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> Response {
    puffin::profile_function!();
    let post_og = post.clone();
    let mut like: Option<bool> = None;
    let mut repost: Option<bool> = None;
    let post = {
        puffin::profile_scope!("Mutex Lock");
        &post_og.lock().unwrap()
    };
    ui.style_mut().spacing.item_spacing.y = 40.0;
    if post.reason.is_some() || post.reply.is_some() {
        puffin::profile_scope!("Reason");
        ui.style_mut().spacing.item_spacing = vec2(10.0, 2.0);
        ui.with_layout(Layout::left_to_right(egui::Align::TOP), |name| {
            name.allocate_space(vec2(60.0, 2.0));
            name.style_mut().spacing.item_spacing.x = 0.0;
            if let Some(reason) = &post.reason {
                match reason {
                    BlueskyApiTimelineReason::Repost(repost) => {
                        name.weak(format!(
                            "\u{E201} Reposted by {}",
                            if let Some(dn) = &repost.by.display_name {
                                if dn.len() > 0 {
                                    dn
                                } else {
                                    &repost.by.handle
                                }
                            } else {
                                &repost.by.handle
                            }
                        ));
                    }
                    BlueskyApiTimelineReason::Pin => {
                        name.weak("Pinned");
                    }
                }
            } else if let Some(reply) = &post.reply {
                match &reply.parent {
                    BlueskyApiTimelineReasonReply::Post(post) => {
                        name.weak(format!(
                            "\u{E200} Replying to {}",
                            if let Some(name) = &post.author.display_name {
                                if name.len() > 0 {
                                    name
                                } else {
                                    &post.author.handle
                                }
                            } else {
                                &post.author.handle
                            }
                        ));
                    }
                    BlueskyApiTimelineReasonReply::NotFound => {
                        name.weak("\u{E200} Replying to an unknown post");
                    }
                    BlueskyApiTimelineReasonReply::Blocked => {
                        name.weak("\u{E200} Replying to a blocked post");
                    }
                }
            }
        });
        ui.style_mut().spacing.item_spacing.y = 10.0;
    }

    let ffs = ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
        puffin::profile_scope!("Main Container");
        ui.style_mut().spacing.item_spacing = vec2(10.0, 10.0);
        let (_, pfp_rect) = ui.allocate_space(vec2(60.0, 60.0));
        if ui.is_rect_visible(pfp_rect) {
            let tex: Option<TextureId> = if let Some(avatar) = &post.post.author.avatar {
                match img_cache.get_image(avatar) {
                    LoadableImage::Unloaded | LoadableImage::Loading => None,
                    LoadableImage::Loaded(texture_id) => Some(texture_id),
                }
            } else {
                None
            };

            if let Some(id) = tex {
                ui.painter().image(id, pfp_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
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
                let guh_fr = name.painter().layout_no_wrap(offset_time(post.post.record.created_at), FontId::new(16.0, egui::FontFamily::Name("Segoe Light".into())), Color32::DARK_GRAY);
                name.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                name.set_width(the_width_you_care_about - (20.0 + guh_fr.mesh_bounds.width()));
                let display_name = if let Some(display_name) = &post.post.author.display_name {
                    if display_name.len() > (0 as usize) {
                        // WHAT'S THE FUCKING POINT, BLUESKY???? IF YOU HAVE AN OPTIONAL FIELD, USE THAT FACT AND DON'T JUST RETURN BLANK
                        Some(name.add(egui::Label::new(egui::RichText::new(display_name).size(20.0).color(name.style().visuals.text_color())).selectable(false).sense(egui::Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand))
                    } else {
                        None
                    }
                } else {
                    None
                };
                let name_res = if display_name.is_none() { name.add(egui::Label::new(egui::RichText::new(&post.post.author.handle).size(20.0)).selectable(false).sense(egui::Sense::click())) } else { name.add(egui::Label::new(egui::RichText::new(&post.post.author.handle).weak().font(FontId::new(20.0, egui::FontFamily::Name("Segoe Light".into()))).size(20.0)).selectable(false).sense(egui::Sense::click())) }.on_hover_cursor(egui::CursorIcon::PointingHand);

                let click_response = if let Some(dn) = display_name {
                    if dn.hovered() {
                        dn.clicked()
                    } else {
                        name_res.clicked()
                    }
                } else {
                    name_res.clicked()
                };

                if click_response {
                    new_view.set(FrontendMainView::Profile(FrontendProfileView::new(post.post.author.did.clone())));
                }
                
                name.painter().galley(pos2(name_res.rect.right() + 20.0, name_res.rect.right_center().y - guh_fr.mesh_bounds.height() / 2.0), guh_fr, Color32::GREEN);
                name.painter().circle_filled(pos2(name_res.rect.right() + 10.0, name_res.rect.right_center().y + 5.0), 2.0, Color32::DARK_GRAY);
            });
            if post.post.record.text.len() > (0 as usize) {
                puffin::profile_scope!("Text");
                let mut job = LayoutJob::default();
                job.wrap = TextWrapping::wrap_at_width(post_contents.cursor().width());
                let font_id = FontId::proportional(14.0);
                job.text = post.post.record.text.clone();

                /* if let Some(facets) = &post.post.record.facets {
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
                    job.append(&post.post.record.text, 0.0, TextFormat::simple(font_id, post_contents.style().visuals.noninteractive().fg_stroke.color));
                }

                let galley = post_contents.fonts(|f| f.layout_job(job));
                if post_contents.add(egui::Label::new(galley).sense(egui::Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    new_view.set(FrontendMainView::Thread(FrontendThreadView::new(post.post.uri.clone())));
                }
            }

            //post_contents.painter().rect(post_contents.cursor(), Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(2.0, Color32::ORANGE));

            const MEDIA_SIZE: f32 = 180.0;

            if let Some(embed) = &post.post.embed {
                puffin::profile_scope!("Embed");
                match embed {
                    BlueskyApiTimelinePostEmbedView::Images { images } => 'render_images: {
                        puffin::profile_scope!("Images");
                        let img_rect = post_contents.cursor().with_max_y(post_contents.cursor().top() + MEDIA_SIZE);
                        if !post_contents.is_rect_visible(img_rect) {
                            puffin::profile_scope!("Image Short-Circuit");
                            post_contents.allocate_rect(img_rect, egui::Sense::click());
                            break 'render_images;
                        }
                        post_contents.allocate_new_ui(UiBuilder::default().max_rect(img_rect), |container| {
                            ScrollArea::horizontal().max_width(img_rect.width()).max_height(img_rect.height()).vscroll(false).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded).id_salt(&post.post.cid).show(container, |container| {
                                container.with_layout(Layout::left_to_right(egui::Align::Min), |container| {
                                    for img in images {
                                        if !container.is_visible() {
                                            continue;
                                        }
                                        puffin::profile_scope!("Image");
                                        let x_multiplier = if let Some(ratio) = &img.aspect_ratio { ratio.width as f32 / ratio.height as f32 } else { 1.0 };
                                        let img_rect = container.allocate_rect(container.cursor().with_max_x(container.cursor().left() + (MEDIA_SIZE * x_multiplier)), egui::Sense::click());
                                        match img_cache.get_image(&img.thumb) {
                                            LoadableImage::Unloaded | LoadableImage::Loading => {
                                                container.painter().rect_filled(img_rect.rect, Rounding::ZERO, Color32::GRAY);
                                            }
                                            LoadableImage::Loaded(id) => {
                                                container.painter().image(id, img_rect.rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
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
                                    }
                                });
                            });
                        });
                    }
                    BlueskyApiTimelinePostEmbedView::Video(video) => 'render_video: {
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
                                LoadableImage::Loaded(texture_id) => Some(texture_id),
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
                    }
                    BlueskyApiTimelinePostEmbedView::External { external: _ } => {
                        let resp = post_contents.allocate_new_ui(UiBuilder::default().max_rect(post_contents.cursor().shrink(8.0)), |quote| {
                            quote.with_layout(Layout::left_to_right(egui::Align::Min), |name| {
                                name.weak("External Link/Embed");
                            });
                        });

                        post_contents.painter().rect(resp.response.rect.expand(4.0), Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(2.0, post_contents.style().visuals.text_color()));
                    }
                    BlueskyApiTimelinePostEmbedView::Record { record } => {
                        puffin::profile_scope!("Record");
                        let resp = post_contents.allocate_new_ui(UiBuilder::default().max_rect(post_contents.cursor().shrink(8.0)), |quote| {
                            quote.with_layout(Layout::left_to_right(egui::Align::Min), |name| {
                                name.horizontal_wrapped(|name| match record {
                                    BlueskyApiTimelineEmbedRecordView::Record(value) => {
                                        name.weak(format!("{:?}", value));
                                    }
                                    BlueskyApiTimelineEmbedRecordView::NotFound(_) => {
                                        name.weak("Not Found");
                                    }
                                    BlueskyApiTimelineEmbedRecordView::Blocked(_) => {
                                        name.weak("Blocked");
                                    }
                                    BlueskyApiTimelineEmbedRecordView::Detached(_) => {
                                        name.weak("Detached Record");
                                    }
                                    BlueskyApiTimelineEmbedRecordView::FeedGenerator(_) => {
                                        name.weak("Feed Generator");
                                    }
                                    BlueskyApiTimelineEmbedRecordView::List(_) => {
                                        name.weak("List");
                                    }
                                    BlueskyApiTimelineEmbedRecordView::Labeler(_) => {
                                        name.weak("Labeler");
                                    }
                                    BlueskyApiTimelineEmbedRecordView::PackView(_) => {
                                        name.weak("PackView");
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
                if post.post.viewer.is_none() {
                    break 'render_action_buttons; // if there's no viewer, you can't interact with it (for the most part) so don't bother
                }
                if !action_buttons.is_rect_visible(action_buttons.cursor().with_max_y(action_buttons.cursor().top() + 30.0)) {
                    action_buttons.allocate_space(vec2(0.0, 30.0));
                    break 'render_action_buttons;
                }

                action_buttons.style_mut().spacing.item_spacing.x = 26.0;

                let reply_enabled = if let Some(dis) = post.post.viewer.as_ref().unwrap().reply_disabled { dis } else { true };
                action_buttons.add_enabled_ui(reply_enabled, |action_buttons| {
                    if circle_button(action_buttons, "\u{E206}", 20.0, 15.0, None).clicked() {
                        let reply = BlueskyApiReplyRef {
                            root: if let Some(reply) = &post.post.record.reply { reply.root.clone() } else { BlueskyApiStrongRef { uri: post.post.uri.clone(), cid: post.post.cid.clone() } },
                            parent: BlueskyApiStrongRef { uri: post.post.uri.clone(), cid: post.post.cid.clone() },
                        };
                        flyout.set(crate::frontend::main::ClientFrontendFlyoutVariant::PostComposerFlyout(ComposerFlyout::with_reply(reply)));
                    }
                });
                let rt_override = if post.post.viewer.as_ref().unwrap().repost.is_some() { Some(Color32::from_rgb(92, 239, 170)) } else { None };
                click_context_menu::click_context_menu(circle_button(action_buttons, "\u{E207}", 20.0, 15.0, rt_override), |guh| {
                    if guh.button(if rt_override.is_some() { "Un-Repost" } else { "Repost" }).clicked() {
                        repost = Some(rt_override.is_none());
                    }
                    if guh.add_enabled(false, egui::Button::new("Quote Repost")).clicked() {}
                });
                let like_override = if post.post.viewer.as_ref().unwrap().like.is_some() { Some(Color32::from_rgb(236, 72, 153)) } else { None };
                if circle_button(action_buttons, "\u{E209}", 20.0, 15.0, like_override).clicked() {
                    like = Some(like_override.is_none());
                }
                click_context_menu::click_context_menu(circle_button(action_buttons, "\u{E0C2}", 15.0, 15.0, None), |guh| {
                    if guh.button("Open in browser").clicked() {
                        let id = post.post.uri.split("/").last().unwrap();
                        let handle = if post.post.author.handle.eq("handle.invalid") { &post.post.author.did } else { &post.post.author.handle };
                        let url = format!("https://bsky.app/profile/{}/post/{}", handle, id);

                        #[cfg(target_os = "linux")]
                        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("cmd.exe").arg("/C").arg("start").arg(url).spawn();
                    }
                });
            });
        }); //post contents container
    }); // main container including pfp
    ui.style_mut().spacing.item_spacing.y = 20.0;

    ui.allocate_space(vec2(0.0, 0.0)); // weird hack because spacing doesn't apply i guess?

    if let Some(repost) = repost {
        if repost {
            let record = crate::backend::record::BlueskyApiRecord::Repost(BlueskyApiRecordLike { subject: BlueskyApiStrongRef { uri: post.post.uri.clone(), cid: post.post.cid.clone() }, created_at: Utc::now() });
            backend.backend_commander.send(crate::bridge::FrontToBackMsg::CreateRecordUnderPostRequest(record, post_og.clone())).unwrap();
        } else {
            if let Some(viewer) = &post.post.viewer {
                if let Some(viewer_repost) = &viewer.repost {
                    backend.backend_commander.send(crate::bridge::FrontToBackMsg::DeleteRecordUnderPostRequest(viewer_repost.split("/").last().unwrap().to_owned(), "app.bsky.feed.repost".to_owned(), post_og.clone())).unwrap();
                }
            }
        }
    }

    if let Some(like) = like {
        if like {
            let record = crate::backend::record::BlueskyApiRecord::Like(BlueskyApiRecordLike { subject: BlueskyApiStrongRef { uri: post.post.uri.clone(), cid: post.post.cid.clone() }, created_at: Utc::now() });
            backend.backend_commander.send(crate::bridge::FrontToBackMsg::CreateRecordUnderPostRequest(record, post_og.clone())).unwrap();
        } else {
            if let Some(viewer) = &post.post.viewer {
                if let Some(viewer_like) = &viewer.like {
                    backend.backend_commander.send(crate::bridge::FrontToBackMsg::DeleteRecordUnderPostRequest(viewer_like.split("/").last().unwrap().to_owned(), "app.bsky.feed.like".to_owned(), post_og.clone())).unwrap();
                }
            }
        }
    }

    ffs.response
}
