use chrono::Utc;
use egui::{pos2, vec2, Align2, Color32, FontId, Layout, Rect, Rounding, TextEdit, TextStyle, Ui};

use crate::{
    backend::record::{BlueskyApiRecord, BlueskyApiRecordPost, BlueskyApiReplyRef},
    bridge::Bridge,
    defs::bsky::actor::defs::ProfileViewDetailed,
    frontend::{circle_button, main::ClientFrontendFlyoutVariant},
    image::ImageCache,
    widgets::spinner::SegoeBootSpinner,
};

const BSKY_BLUE: Color32 = Color32::from_rgb(32, 139, 254);

pub struct ComposerFlyout {
    pub draft: String,
    pub sending: bool,
    pub reply: Option<BlueskyApiReplyRef>,
}

impl ComposerFlyout {
    pub fn new() -> Self {
        Self { draft: String::new(), sending: false, reply: None }
    }

    pub fn with_reply(reply: BlueskyApiReplyRef) -> Self {
        Self { draft: String::new(), sending: false, reply: Some(reply) }
    }
}

fn render_mini_profile(ui: &mut Ui, image: &ImageCache, avatar: &Option<String>, display_name: &Option<String>, handle: &String) -> egui::Response {
    ui.with_layout(Layout::left_to_right(egui::Align::Min), |user| {
        let space = user.allocate_space(vec2(40.0, 40.0));
        let tex = if let Some(avatar) = &avatar {
            match image.get_image(&avatar) {
                crate::image::LoadableImage::Unloaded | crate::image::LoadableImage::Loading => None,
                crate::image::LoadableImage::Loaded(texture_id, _) => Some(texture_id),
            }
        } else {
            None
        };

        if let Some(id) = tex {
            user.painter().image(id, space.1, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        } else {
            user.painter().rect_filled(space.1, Rounding::ZERO, BSKY_BLUE);
            user.painter().text(space.1.center(), Align2::CENTER_CENTER, "", FontId::new(30.0, egui::FontFamily::Name("Segoe Symbols".into())), Color32::WHITE);
        }

        user.with_layout(Layout::top_down(egui::Align::Min), |name| {
            name.style_mut().spacing.item_spacing.y = 2.0;
            if let Some(dn) = display_name {
                name.label(egui::RichText::new(dn).color(Color32::BLACK).font(FontId::new(16.0, egui::FontFamily::Name("Segoe Light".into()))));
                name.weak(format!("@{}", handle));
            } else {
                name.label(egui::RichText::new(handle).color(Color32::BLACK).font(FontId::new(16.0, egui::FontFamily::Name("Segoe Light".into()))));
            }
        })
        .response
    })
    .response
}

impl ClientFrontendFlyoutVariant {
    pub fn post_composer(ui: &mut Ui, data: &mut ComposerFlyout, profile: &Option<ProfileViewDetailed>, img_cache: &ImageCache, backend: &Bridge) {
        let center = ui.cursor().center();
        ui.add_enabled_ui(!data.sending, |ui| {
            let right_limit = ui.cursor().right();
            ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                let res = if let Some(profile) = &profile { render_mini_profile(ui, img_cache, &profile.avatar, &profile.display_name, &profile.handle) } else { ui.label(egui::RichText::new("Unknown Profile").color(Color32::BLACK).font(FontId::new(16.0, egui::FontFamily::Name("Segoe Light".into())))) };
                ui.painter().text(pos2(right_limit, res.rect.center().y), Align2::RIGHT_CENTER, format!("{}", 300 as i16 - data.draft.len() as i16), FontId::proportional(12.0), if data.draft.len() > 300 { Color32::RED } else { Color32::GRAY });
            });

            let draft = TextEdit::multiline(&mut data.draft);
            draft.desired_width(ui.cursor().width()).text_color(Color32::BLACK).hint_text("Write Here").frame(false).font(TextStyle::Body).show(ui);

            ui.with_layout(Layout::left_to_right(egui::Align::Min), |buttons| {
                circle_button(buttons, "", 20.0, 15.0, None);
                circle_button(buttons, "", 20.0, 15.0, None);
                let send_button_rect = buttons.cursor().with_min_x(right_limit - 90.0).with_max_x(right_limit).with_max_y(buttons.cursor().top() + 30.0);
                let send_button = buttons.add_enabled_ui(data.draft.len() > 0, |buttons| buttons.allocate_rect(send_button_rect, egui::Sense::click())).inner.on_hover_cursor(egui::CursorIcon::PointingHand);
                buttons.painter().rect_filled(send_button_rect, Rounding::ZERO, BSKY_BLUE.gamma_multiply(if data.draft.len() > 0 && data.draft.len() <= 300 { 1.0 } else { 0.5 }));

                buttons.painter().text(send_button_rect.center() - vec2(0.0, 2.0), Align2::CENTER_CENTER, "Post", FontId::proportional(10.0), Color32::WHITE);

                if send_button.clicked() {
                    let mut languages: Vec<String> = Vec::new();
                    languages.push("en".to_owned());
                    let record = BlueskyApiRecord::Post(BlueskyApiRecordPost { text: data.draft.clone(), created_at: Utc::now(), facets: None, reply: data.reply.clone(), embed: None, langs: Some(languages), labels: None, tags: None });
                    backend.backend_commander.send(crate::bridge::FrontToBackMsg::CreateRecordRequest(record)).unwrap();
                }
            });
        });
        if data.sending {
            SegoeBootSpinner::new().size(60.0).color(BSKY_BLUE).paint_at(ui, Rect::from_center_size(center, vec2(60.0, 60.0)));
        }
    }
}
