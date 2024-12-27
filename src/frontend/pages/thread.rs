use egui::{pos2, vec2, Align2, Color32, FontId, Rect, Response, Rounding, ScrollArea, Stroke, Ui};

use super::profile::FrontendProfileView;
use super::{MainViewProposition, ViewStackReturnInfo};
use crate::bridge::FrontToBackMsg;
use crate::defs::bsky::feed::defs::{BlockedPost, ThreadPostVariant};
use crate::frontend::main::ClientFrontendFlyout;
use crate::BSKY_BLUE;
use crate::frontend::viewers;
use crate::widgets::spinner::SegoeBootSpinner;
use crate::{bridge::Bridge, image::ImageCache};
#[derive(Debug)]
pub struct FrontendThreadView {
    pub data: Option<ThreadPostVariant>,
    pub id_cmp: String,
    pub loading: bool,
}

impl FrontendThreadView {
    pub fn new(did: String) -> Self {
        Self { data: None, id_cmp: did, loading: false }
    }
}

impl FrontendThreadView {
    fn render_blocked(ui: &mut Ui, info: &BlockedPost, new_view: &mut MainViewProposition) -> Response {
        ui.painter().text(ui.cursor().min + vec2(18.0, 16.0), Align2::CENTER_CENTER, "\u{E181}", FontId::new(20.0, egui::FontFamily::Name("Segoe Symbols".into())), ui.visuals().weak_text_color());
        let rect = ui.painter().text(ui.cursor().min + vec2(36.0, 28.0), Align2::LEFT_BOTTOM, "Blocked", FontId::proportional(20.0), ui.visuals().weak_text_color());

        let res = ui.allocate_response(vec2(rect.width() + 42.0, 36.0), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        ui.painter().rect(res.rect, Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(2.0, ui.visuals().weak_text_color()));
        ui.painter().line_segment([pos2(res.rect.left() + 30.0, res.rect.bottom() + 1.0), pos2(res.rect.left() + 30.0, res.rect.bottom() + ui.spacing().item_spacing.y / 2.0)], Stroke::new(2.0, ui.visuals().weak_text_color()));

        if res.clicked() { new_view.set(super::FrontendMainView::Profile(FrontendProfileView::new(info.author.did.clone()))); }

        res
    }

    fn render_not_found(ui: &mut Ui) -> Response {
        ui.painter().text(ui.cursor().min + vec2(18.0, 14.0), Align2::CENTER_CENTER, "\u{E283}", FontId::new(20.0, egui::FontFamily::Name("Segoe Symbols".into())), ui.visuals().weak_text_color());
        let rect = ui.painter().text(ui.cursor().min + vec2(36.0, 28.0), Align2::LEFT_BOTTOM, "Deleted Post", FontId::proportional(20.0), ui.visuals().weak_text_color());

        let res = ui.allocate_response(vec2(rect.width() + 42.0, 36.0), egui::Sense::click());
        ui.painter().rect(res.rect, Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(2.0, ui.visuals().weak_text_color()));
        ui.painter().line_segment([pos2(res.rect.left() + 30.0, res.rect.bottom() + 1.0), pos2(res.rect.left() + 30.0, res.rect.bottom() + ui.spacing().item_spacing.y / 2.0)], Stroke::new(2.0, ui.visuals().weak_text_color()));

        res
    }

    fn render_reply(ui: &mut Ui, reply: &ThreadPostVariant, depth: u32, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> Response {
        match reply {
            ThreadPostVariant::NotFound(_) => { Self::render_not_found(ui) },
            ThreadPostVariant::Blocked(info) => { Self::render_blocked(ui, info, new_view) },
            ThreadPostVariant::ThreadView(post) => {
                let rtn = viewers::post::post_viewer(ui, post.post.clone(), false, backend, image, flyout, new_view);
                if depth <= 0 { return rtn; }
                if let Some(replies) = &post.replies {
                    if let Some(first) = replies.first() {
                        ui.painter().line_segment([pos2(rtn.rect.left() + 30.0, rtn.rect.top() + 70.0), pos2(rtn.rect.left() + 30.0, rtn.rect.bottom() + (ui.style().spacing.item_spacing.y) + 10.0)], ui.style().visuals.widgets.inactive.fg_stroke);
                        Self::render_reply(ui, first, depth - 1, backend, image, flyout, new_view);
                    }
                }
                rtn
            }
        }
    }

    fn render_recursive(ui: &mut Ui, thread: &ThreadPostVariant, first: bool, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> Response {
        match thread {
            ThreadPostVariant::Blocked(info) => {
                Self::render_blocked(ui, info, new_view)
            },
            ThreadPostVariant::NotFound(_) => {
                Self::render_not_found(ui)
            },
            ThreadPostVariant::ThreadView(thread) => {
                if let Some(parent) = &thread.parent {
                    let res = Self::render_recursive(ui, &parent.lock().unwrap(), false, backend, image, flyout, new_view);
                    ui.painter().line_segment([pos2(res.rect.left() + 30.0, res.rect.top() + 70.0), pos2(res.rect.left() + 30.0, res.rect.bottom() + (ui.style().spacing.item_spacing.y) + 10.0)], ui.style().visuals.widgets.inactive.fg_stroke);
                }
                let rtn = viewers::post::post_viewer(ui, thread.post.clone(), first, backend, image, flyout, new_view);

                if first {

                    if let Some(replies) = &thread.replies {
                        // this is kind of jank but it works for now :)
                        let reply_depth = match replies.len() {
                             1..5  => 80, // functionally infinite (i hope. for your mental health. also performance, but still.)
                             5..10 => 4,
                            10..50 => 2,
                            _      => 0,
                        };
                        let (_, line_rect) = ui.allocate_space(vec2(rtn.rect.width(), ui.style().visuals.widgets.inactive.fg_stroke.width * 2.0));
                        ui.painter().rect_filled(line_rect.with_max_x(ui.cursor().right()), Rounding::ZERO, ui.style().visuals.widgets.inactive.fg_stroke.color);
                        for reply in replies {
                            Self::render_reply(ui, reply, reply_depth, backend, image, flyout, new_view);
                        }
                    }
                }

                rtn
            }
        }
    }

    pub fn render(&mut self, ui: &mut Ui, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> ViewStackReturnInfo {
        puffin::profile_function!();

        if let Some(thread) = &self.data {
            ScrollArea::vertical().hscroll(false).show(ui, |scroll| {
                Self::render_recursive(scroll, &thread, true, backend, image, flyout, new_view);
                scroll.allocate_space(vec2(scroll.cursor().width(), 0.0));
            });
        } else {
            SegoeBootSpinner::new().size(200.0).color(BSKY_BLUE).paint_at(ui, ui.ctx().screen_rect());
            if !self.loading {
                backend.backend_commander.send(FrontToBackMsg::GetThreadRequest(self.id_cmp.clone())).unwrap();
                self.loading = true;
            }
        }

        ViewStackReturnInfo {
            title: Some("Thread".into()),
            render_back_button: true,
            handle_back_logic: true,
            force_back: false,
        }
    }
}