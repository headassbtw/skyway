
use egui::{pos2, Align2, Color32, Ui, FontId};

use crate::frontend::main::ClientFrontend;

const BSKY_BLUE: Color32 = Color32::from_rgb(32, 139, 254);

impl ClientFrontend {
	pub fn thread_page(&mut self, ui: &mut Ui) {
		puffin::profile_function!();
		let pos = pos2(ui.cursor().left(), ui.cursor().top() - 40.0);
		ui.painter().text(pos, Align2::LEFT_BOTTOM, "Thread", FontId::new(40.0, egui::FontFamily::Name("Segoe Light".into())), BSKY_BLUE);
	}
}