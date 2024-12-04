use egui::{vec2, Color32, FontId, Layout};

use crate::frontend::main::ClientFrontend;

pub struct ImportantErrorModal {
    pub header: String,
    pub description: String,
}

impl ImportantErrorModal {
    pub fn new(header: String, description: String) -> Self {
        Self { header, description }
    }
}

impl ClientFrontend {
    pub fn important_error_modal(&mut self, ui: &mut egui::Ui) {
        puffin::profile_function!();
        let data = if let Some(data) = &mut self.modal.main {
            match data {
                crate::frontend::main::ClientFrontendModalVariant::ImportantErrorModal(data) => data,
                _ => panic!("Wrong modal!"),
            }
        } else {
            return;
        };

        ui.heading(egui::RichText::new(&data.header).size(20.0).color(Color32::WHITE).font(FontId::new(20.0, egui::FontFamily::Name("Segoe Light".into()))));
        if data.description.len() > 0 {
            ui.label(egui::RichText::new(&data.description).color(Color32::WHITE));
        }

        let mut close: bool = false;
        ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
            ui.with_layout(Layout::right_to_left(egui::Align::Max), |ui| {
                if ui.add_sized(vec2(90.0, 32.0), egui::Button::new("Cancel")).clicked() {
                    close = true;
                }
            });
            ui.label(egui::RichText::new("Warning: This application may not remain in a usable state. If you choose to report this, do not report the actions after this page, but rather before and on it.").color(Color32::YELLOW));
        });
        if close {
            self.modal.close();
        }
    }
}
