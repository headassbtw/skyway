use egui::{vec2, Color32, FontId, Layout};

use crate::{defs::atproto::label, frontend::main::ClientFrontend, open_in_browser, BSKY_BLUE};

pub struct DeceptiveLinkModal {
    pub label: String,
    pub target: String,
}

impl DeceptiveLinkModal {
    pub fn new(label: String, target: String) -> Self {
        Self { label, target }
    }
}

impl ClientFrontend {
    pub fn deceptive_link_modal(&mut self, ui: &mut egui::Ui) {
        puffin::profile_function!();
        let data = if let Some(data) = &mut self.modal.main {
            match data {
                crate::frontend::main::ClientFrontendModalVariant::DeceptiveLink(data) => data,
                _ => panic!("Wrong modal!"),
            }
        } else {
            return;
        };

        ui.heading(egui::RichText::new("Potentially misleading link").size(20.0).color(Color32::WHITE).font(FontId::new(20.0, egui::FontFamily::Name("Segoe Light".into()))));

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("This link displays itself as ");
            ui.label(egui::RichText::new(&data.label).color(BSKY_BLUE));
            ui.label(", but actually leads to ");
            ui.label(egui::RichText::new(&data.target).color(BSKY_BLUE));
            ui.label(". Would you like to visit it?");
        });

        let mut close: bool = false;
        ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
            ui.with_layout(Layout::right_to_left(egui::Align::Max), |ui| {
                if ui.add_sized(vec2(90.0, 32.0), egui::Button::new("Cancel")).clicked() {
                    close = true;
                }
                if ui.add_sized(vec2(90.0, 32.0), egui::Button::new("Continue")).clicked() {
                    open_in_browser(&data.target);
                    close = true;
                }
            });
        });
        if close {
            self.modal.close();
        }
    }
}
