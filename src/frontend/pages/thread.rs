
use egui::{pos2, Align2, Color32, Ui, FontId};

const BSKY_BLUE: Color32 = Color32::from_rgb(32, 139, 254);

pub struct FrontendThreadView {

}

impl FrontendThreadView {
	pub fn new(uri: String) -> Self {
		Self {
			
		}
	}
}

impl FrontendThreadView {
	pub fn render(&mut self, ui: &mut Ui) -> &str {
		puffin::profile_function!();
		ui.heading("This one's gonna be fun to do :)");
		"Thread"
	}
}