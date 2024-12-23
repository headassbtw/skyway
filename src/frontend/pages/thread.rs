use egui::{pos2, vec2, Response, Rounding, ScrollArea, Ui};

use super::MainViewProposition;
use crate::bridge::FrontToBackMsg;
use crate::defs::bsky::feed::defs::ThreadPostVariant;
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
    fn render_recursive(ui: &mut Ui, thread: &ThreadPostVariant, first: bool, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> Response {
        match thread {
            ThreadPostVariant::NotFound(_) => ui.heading("Not Found"),
            ThreadPostVariant::Blocked(_) => ui.heading("Blocked"),
            ThreadPostVariant::ThreadView(thread) => {
                if let Some(parent) = &thread.parent {
                    let res = Self::render_recursive(ui, &parent.lock().unwrap(), false, backend, image, flyout, new_view);
                    ui.painter().line_segment([pos2(res.rect.left() + 30.0, res.rect.top() + 70.0), pos2(res.rect.left() + 30.0, res.rect.bottom() + (ui.style().spacing.item_spacing.y * 2.0) + 10.0)], ui.style().visuals.widgets.inactive.fg_stroke);
                }
                let rtn = viewers::post::post_viewer(ui, thread.post.clone(), first, backend, image, flyout, new_view);

                if first {
                    if let Some(replies) = &thread.replies {
                        let (_, line_rect) = ui.allocate_space(vec2(rtn.rect.width(), ui.style().visuals.widgets.inactive.fg_stroke.width * 2.0));
                        ui.painter().rect_filled(line_rect.with_max_x(ui.cursor().right()), Rounding::ZERO, ui.style().visuals.widgets.inactive.fg_stroke.color);
                        for reply in replies {
                            match reply {
                                ThreadPostVariant::NotFound(_) | ThreadPostVariant::Blocked(_) => {}
                                ThreadPostVariant::ThreadView(post) => {
                                    viewers::post::post_viewer(ui, post.post.clone(), false, backend, image, flyout, new_view);
                                }
                            }
                        }
                    }
                }

                rtn
            }
        }
    }

    pub fn render(&mut self, ui: &mut Ui, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, new_view: &mut MainViewProposition) -> (&str, bool) {
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

        ("Thread", true)
    }
}