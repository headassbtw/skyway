use std::sync::{Arc, Mutex};

use crate::{
    backend::record::{BlueskyApiRecordLike}, bridge::Bridge, defs::bsky::feed::defs::{FeedViewPost, Reason, RelatedPostVariant}, frontend::{
        main::ClientFrontendFlyout,
        pages::{profile::FrontendProfileView, FrontendMainView, MainViewProposition},
    }, image::{ImageCache, LoadableImage}, open_in_browser, widgets::{click_context_menu, spinner::SegoeBootSpinner}
};


use egui::{
    pos2,
    text::{LayoutJob, TextWrapping},
    vec2, Align2, Color32, FontId, Layout, Rect, Response, Rounding, ScrollArea, Stroke, TextFormat, Ui, UiBuilder, Widget,
};

use super::post::post_viewer;

pub fn feed_post_viewer(ui: &mut Ui, post: &FeedViewPost, backend: &Bridge, img_cache: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> Response {
    if post.reason.is_some() || post.reply.is_some() {
        puffin::profile_scope!("Reason");
        ui.style_mut().spacing.item_spacing = vec2(10.0, 2.0);
        ui.with_layout(Layout::left_to_right(egui::Align::TOP), |name| {
            name.allocate_space(vec2(60.0, 2.0));
            name.style_mut().spacing.item_spacing.x = 0.0;
            if let Some(reason) = &post.reason {
                match reason {
                    Reason::Repost(repost) => {
                        egui::Label::new(egui::RichText::new("\u{E201} Reposted by ").weak()).selectable(false).ui(name);
                        if name.link(egui::RichText::new(repost.by.easy_name()).color(name.visuals().weak_text_color())).clicked() {
                            new_view.set(FrontendMainView::Profile(FrontendProfileView::new(repost.by.did.clone())));
                        }
                    }
                    Reason::Pin => {
                        egui::Label::new(egui::RichText::new("\u{E141} Pinned").weak()).selectable(false).ui(name);
                    }
                }
            } else if let Some(reply) = &post.reply {
                match &reply.parent {
                    RelatedPostVariant::Post(post) => {
                        egui::Label::new(egui::RichText::new("\u{E200} Replying to ").weak()).selectable(false).ui(name);
                        if name.link(egui::RichText::new(post.author.easy_name()).color(name.visuals().weak_text_color())).clicked() {
                            new_view.set(FrontendMainView::Profile(FrontendProfileView::new(post.author.did.clone())));
                        }
                    }
                    RelatedPostVariant::NotFound(_) => {
                        egui::Label::new(egui::RichText::new("\u{E200} Replying to an unknown post").weak()).selectable(false).ui(name);
                    }
                    RelatedPostVariant::Blocked(_) => {
                        egui::Label::new(egui::RichText::new("\u{E200} Replying to a blocked post").weak()).selectable(false).ui(name);
                    }
                }
            }
        });
        ui.style_mut().spacing.item_spacing.y = 10.0;
    }
    post_viewer(ui, post.post.clone(), false, backend, img_cache, flyout, new_view)
}