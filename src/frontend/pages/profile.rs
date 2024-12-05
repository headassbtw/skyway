use egui::Ui;

pub struct FrontendProfileView {

}

impl FrontendProfileView {
	pub fn new() -> Self {
		Self {

		}
	}
	pub fn render(&mut self, ui: &mut Ui) -> &str {
		puffin::profile_function!();
		ui.heading("Don't worry about whose it is yet.");
		ui.label("nor the animations, i'll get to those");
		"Profile"
	}
}