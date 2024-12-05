use egui::{pos2, Align2, Color32, FontId, Rect, Rounding, Ui};
use profile::FrontendProfileView;
use thread::FrontendThreadView;
use timeline::FrontendTimelineView;

use crate::{bridge::Bridge, image::ImageCache};

use super::main::{ClientFrontendFlyout, ClientFrontendModal};

const BSKY_BLUE: Color32 = Color32::from_rgb(32, 139, 254);

pub mod landing;
pub mod profile;
pub mod thread;
pub mod timeline;

pub enum FrontendMainView {
    Login(),
    Timeline(FrontendTimelineView),
    Thread(FrontendThreadView),
    Profile(FrontendProfileView),
}

pub struct FrontendMainViewStack {
    ctx: egui::Context,
    stack: Vec<FrontendMainView>,
    propose: MainViewProposition, // add animaiton state and whatnot
}

pub struct MainViewProposition(Option<FrontendMainView>);

impl MainViewProposition {
	pub fn set(&mut self,to: FrontendMainView) {
		self.0 = Some(to);
	}

	pub fn new() -> Self {
		Self(None)
	}
}

impl FrontendMainViewStack {
    pub fn new(ctx: egui::Context, initial: FrontendMainView) -> Self {
        Self {
            ctx,
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
        // do animation shit later
        self.stack.pop();
    }

    pub fn render(&mut self, ui: &mut Ui, backend: &Bridge, image: &ImageCache, flyout: &mut ClientFrontendFlyout, modal: &mut ClientFrontendModal) {
    	if let Some(guh) = self.propose.0.take() {
    		self.stack.push(guh);
    	}

    	let pos = pos2(ui.cursor().left(), ui.cursor().top() - 40.0);
        
        let guh = self.stack.last_mut().unwrap();
        let _ = modal; // shut the fuck up
        let title = match guh {
            FrontendMainView::Login() => {
            	ui.label("Login placeholder");
                //FrontendMainView::landing(self, ui);
                ""
            }
            FrontendMainView::Timeline(ref mut data) => data.render(ui, backend, image, flyout, &mut self.propose),
            FrontendMainView::Thread(ref mut data) => data.render(ui, backend, image, flyout, &mut self.propose),
            FrontendMainView::Profile(ref mut data) => data.render(ui, backend, image),
        };
        ui.painter().text(pos, Align2::LEFT_BOTTOM, title, FontId::new(40.0, egui::FontFamily::Name("Segoe Light".into())), BSKY_BLUE);

        if self.stack.len() > 1 {
    		let back_button_rect = Rect { min: pos2(pos.x - 60.0, pos.y - 44.0), max: pos2(pos.x - 20.0, pos.y - 4.0) };

    		let back_button = ui.allocate_rect(back_button_rect, egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
    		let col = if ui.is_enabled() { BSKY_BLUE } else { Color32::RED };
            ui.painter().text(back_button_rect.center(), Align2::CENTER_CENTER, "\u{E0BA}", FontId::new(40.0, egui::FontFamily::Name("Segoe Symbols".into())), col);
            if back_button.clicked() {
            	self.pop();
            }
    	}
    }

    /// TO BE REMOVED, stopgap before i remove the callback architecture
    pub fn top(&mut self) -> Option<&mut FrontendMainView> {
        return self.stack.last_mut();
    }
}
