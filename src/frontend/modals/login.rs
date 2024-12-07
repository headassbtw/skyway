use egui::{vec2, Color32, FontId, Label, Layout, Margin, RichText, Stroke, TextEdit, Widget};

use crate::frontend::main::ClientFrontend;

pub struct LoginModal {
    pub username: String,
    pub password: String,
    pub password_dots: bool,
    pub error_msg: String,
    pub interactive: bool,
}

impl LoginModal {
    pub fn new() -> Self {
        Self { username: "".to_owned(), password: "".to_owned(), password_dots: true, error_msg: "".to_owned(), interactive: true }
    }
}

impl ClientFrontend {
    pub fn login_modal(&mut self, ui: &mut egui::Ui) {
        puffin::profile_function!();
        let data = if let Some(data) = &mut self.modal.main {
            match data {
                crate::frontend::main::ClientFrontendModalVariant::LoginModal(data) => data,
                _ => panic!("Wrong modal!"),
            }
        } else {
            return;
        };

        Label::new(RichText::new("Add your Bluesky account").size(20.0).color(Color32::WHITE).font(FontId::new(20.0, egui::FontFamily::Name("Segoe Light".into())))).selectable(false).ui(ui);

        ui.add_enabled_ui(true, |ui| {
            ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::WHITE;
            ui.style_mut().visuals.widgets.inactive.fg_stroke = Stroke::new(2.0, Color32::BLACK);
            ui.style_mut().visuals.widgets.active.bg_fill = Color32::RED;
            ui.style_mut().visuals.widgets.active.expansion = 0.0;

            TextEdit::singleline(&mut data.username).min_size(vec2(390.0, 32.0)).font(FontId::proportional(11.0)).vertical_align(egui::Align::Center).margin(Margin::symmetric(10.0, 0.0)).hint_text("Username/Handle").show(ui);
            let pw_res = TextEdit::singleline(&mut data.password).min_size(vec2(390.0, 32.0)).font(FontId::proportional(11.0)).vertical_align(egui::Align::Center).margin(Margin::symmetric(10.0, 0.0)).hint_text("Password").password(data.password_dots).show(ui);

            let show_pw_rect = pw_res.response.rect.clone();
            let show_pw_rect = show_pw_rect.with_min_x(show_pw_rect.right() + 10.0 - show_pw_rect.height());
            let show_pw_rect = show_pw_rect.with_max_x(show_pw_rect.right() + 10.0);

            ui.style_mut().override_font_id = Some(FontId::new(20.0, egui::FontFamily::Name("Segoe Symbols".into())));
            if ui.put(show_pw_rect, egui::Button::new("").frame(false).min_size(vec2(show_pw_rect.height(), show_pw_rect.height()))).clicked() {
                data.password_dots = !data.password_dots;
            }
        });
        let mut close: bool = false;
        ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
            ui.with_layout(Layout::right_to_left(egui::Align::Max), |ui| {
                if ui.add_sized(vec2(90.0, 32.0), egui::Button::new("Cancel")).clicked() {
                    close = true;
                }
                if ui.add_sized(vec2(90.0, 32.0), egui::Button::new("Log In")).clicked() {
                    self.backend.backend_commander.send(crate::bridge::FrontToBackMsg::LoginRequestStandard(data.username.clone(), data.password.clone())).unwrap();
                }
            });
            if data.error_msg.len() > 0 {
                ui.style_mut().override_font_id = Some(FontId::new(11.0, egui::FontFamily::Name("Segoe Light".into())));
                ui.label(egui::RichText::new(&data.error_msg).color(Color32::YELLOW));
            }
        });
        if close {
            self.modal.close();
        }
    }
}
