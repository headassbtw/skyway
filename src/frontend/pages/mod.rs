use egui::{pos2, vec2, Align2, Color32, FontId, Rect, Ui, UiBuilder};
use media::FrontendMediaViewVariant;
use profile::FrontendProfileView;
use thread::FrontendThreadView;
use timeline::FrontendTimelineView;

use crate::{bridge::Bridge, defs::bsky::actor::defs::ProfileViewDetailed, image::ImageCache, BSKY_BLUE};

use super::main::{ClientFrontendFlyout, ClientFrontendModal};

pub mod landing;
pub mod media;
pub mod profile;
pub mod thread;
pub mod timeline;

pub enum FrontendMainView {
    Login(),
    Timeline(FrontendTimelineView),
    Thread(FrontendThreadView),
    Profile(FrontendProfileView),
    Media(FrontendMediaViewVariant),
}

pub struct FrontendMainViewStack {
    ctx: egui::Context,
    stack: Vec<FrontendMainView>,
    propose: MainViewProposition, // add animaiton state and whatnot
}

pub struct MainViewProposition(Option<FrontendMainView>, bool);

impl MainViewProposition {
    pub fn set(&mut self, to: FrontendMainView) {
        self.0 = Some(to);
    }

    pub fn new() -> Self {
        Self(None, false)
    }
}

fn ease_out_cubic(x: f32) -> f32 {
    return 1.0 - f32::powf(1.0 - x, 3.0);
}

impl FrontendMainViewStack {
    pub fn new(ctx: egui::Context, initial: FrontendMainView) -> Self {
        Self {
            ctx: ctx.clone(),
            stack: {
                let mut stack = Vec::new();
                stack.push(initial);
                stack
            },
            propose: MainViewProposition::new(),
        }
    }

    pub fn set(&mut self, to: FrontendMainView) {
        // fuck with propose later but for now just do this lmao
        //self.propose = Some(to);
        self.stack.clear();
        self.stack.push(to);
    }

    pub fn pop(&mut self) {
        if self.stack.len() < 2 {
            return;
        }
        self.ctx.animate_bool_with_time("FrontendMainViewStackSlide".into(), false, 0.0);
        self.ctx.animate_bool_with_time("FrontendMainViewStackTitleSlide".into(), false, 0.0);
        self.stack.pop();
    }

    pub fn render(&mut self, ui: &mut Ui, you: &Option<ProfileViewDetailed>, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, modal: &mut ClientFrontendModal) {
        if let Some(guh) = self.propose.0.take() {
            self.ctx.animate_bool_with_time("FrontendMainViewStackSlide".into(), false, 0.0);
            self.ctx.animate_bool_with_time("FrontendMainViewStackTitleSlide".into(), false, 0.0);
            self.stack.push(guh);
        } else if self.propose.1 {
            self.pop();
            self.propose.1 = false;
        }

        let offset = self.ctx.animate_bool_with_time_and_easing("FrontendMainViewStackTitleSlide".into(), true, 0.5, ease_out_cubic);
        let pos = pos2(ui.cursor().left() + (100.0 - (offset * 100.0)), ui.cursor().top() - 40.0);

        let guh = self.stack.last_mut().unwrap();
        let _ = modal; // shut the fuck up
        let offset = self.ctx.animate_bool_with_time_and_easing("FrontendMainViewStackSlide".into(), true, 0.7, ease_out_cubic);
        let mut view = ui.new_child(UiBuilder::new().max_rect(ui.cursor().with_max_y(self.ctx.screen_rect().bottom()).translate(vec2(100.0 - (offset * 100.0), 0.0))));
        let (title, render_header) = match guh {
            FrontendMainView::Login() => {
                FrontendMainView::landing(&mut view, modal);
                ("", false)
            }
            FrontendMainView::Timeline(ref mut data) => data.render(&mut view, you, backend, image, flyout, &mut self.propose),
            FrontendMainView::Thread(ref mut data) => data.render(&mut view, backend, image, flyout, &mut self.propose),
            FrontendMainView::Profile(ref mut data) => data.render(&mut view, backend, image, flyout, &mut self.propose),
            FrontendMainView::Media(ref mut data) => data.render(&mut view, image, &mut self.propose),
        };
        
        if !render_header { return; }

        ui.painter().text(pos, Align2::LEFT_BOTTOM, title, FontId::new(40.0, egui::FontFamily::Name("Segoe Light".into())), BSKY_BLUE);

        if self.stack.len() > 1 {
            let back_button_rect = Rect { min: pos2(pos.x - 60.0, pos.y - 44.0), max: pos2(pos.x - 20.0, pos.y - 4.0) };

            let back_button = ui.allocate_rect(back_button_rect, egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
            ui.painter().text(back_button_rect.center(), Align2::CENTER_CENTER, "\u{E0BA}", FontId::new(40.0, egui::FontFamily::Name("Segoe Symbols".into())), BSKY_BLUE);
            if back_button.clicked() || ui.input(|r| r.key_pressed(egui::Key::Escape)) {
                self.pop();
            }
        }
    }

    /// TO BE REMOVED, stopgap before i remove the callback architecture
    pub fn top(&mut self) -> Option<&mut FrontendMainView> {
        return self.stack.last_mut();
    }
}
