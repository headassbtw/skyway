use egui::{vec2, FontId, Layout, UiBuilder};

use crate::frontend::{main::ClientFrontend, modals::login::LoginModal};

use super::FrontendMainView;

impl FrontendMainView {
	pub fn landing(host: &mut ClientFrontend, ui: &mut egui::Ui) {
		let ui_rect = egui::Rect::from_center_size(ui.ctx().screen_rect().center(), vec2(200.0, 40.0));

		ui.allocate_new_ui(UiBuilder::new().max_rect(ui_rect), |ui| {
			ui.label(egui::RichText::new("Bluesky").font(FontId::new(20.0, egui::FontFamily::Name("Segoe Light".into()))));
			ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
				ui.style_mut().spacing.item_spacing.x = 0.0;
				ui.label("To use Bluesky, ");
				if ui.link("sign in").clicked() {
					let data = LoginModal::new();
					host.modal.set(crate::frontend::main::ClientFrontendModalVariant::LoginModal(data));
				}
				ui.label(".");
			});
		});

		/*
		ui.painter().text(pos, Align2::LEFT_BOTTOM, "Who Up Blueing They Sky?", FontId::new(40.0, egui::FontFamily::Name("Segoe Light".into())), ui.style().visuals.text_color());
		ui.label("LANDING PAGE");
		if ui.button("").clicked() {
			let data = LoginModal::new();
			self.modal = Some(crate::frontend::main::ClientFrontendModal::LoginModal(data));
		}
		*/
	}

}