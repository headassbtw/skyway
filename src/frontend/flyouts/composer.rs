use std::path::PathBuf;

use chrono::Utc;
use egui::{pos2, vec2, Align2, Color32, FontId, ImageSource, Layout, Rect, Rounding, Style, TextEdit, TextStyle, Ui, UiBuilder, Visuals};
use rfd::FileDialog;

use crate::{
    BSKY_BLUE,
    backend::record::{BlueskyApiRecord, BlueskyApiRecordPost},
    bridge::Bridge,
    defs::bsky::{actor::defs::ProfileViewDetailed, feed::ReplyRef},
    frontend::{circle_button, main::ClientFrontendFlyoutVariant},
    image::ImageCache,
    widgets::spinner::SegoeBootSpinner,
};

pub struct ComposerFlyout {
    pub draft: String,
    pub sending: bool,
    emoji_picker: bool,
    emoji_search: String,
    images: Vec<PathBuf>,
    pub reply: Option<ReplyRef>,
}

impl ComposerFlyout {
    pub fn new() -> Self {
        Self { draft: String::new(), sending: false, emoji_picker: false, emoji_search: String::new(), images: Vec::new(), reply: None }
    }

    pub fn with_reply(reply: ReplyRef) -> Self {
        Self { draft: String::new(), sending: false, emoji_picker: false, emoji_search: String::new(), images: Vec::new(), reply: Some(reply) }
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

fn special_char_name(_: char) -> Option<&'static str> {
    None
}

fn char_name(chr: char) -> String {
    special_char_name(chr)
        .map(|s| s.to_owned())
        .or_else(|| unicode_names2::name(chr).map(|name| name.to_string().to_lowercase()))
        .unwrap_or_else(|| "unknown".to_owned())
}

impl ClientFrontendFlyoutVariant {
    pub fn post_composer(ui: &mut Ui, data: &mut ComposerFlyout, profile: &Option<ProfileViewDetailed>, img_cache: &ImageCache, backend: &Bridge) {
        let center = ui.cursor().center();
        *ui.visuals_mut() = Visuals::light();
        ui.add_enabled_ui(!data.sending, |ui| {
            let right_limit = ui.cursor().right();
            ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                let res = if let Some(profile) = &profile { render_mini_profile(ui, img_cache, &profile.avatar, &profile.display_name, &profile.handle) } else { ui.label(egui::RichText::new("Unknown Profile").color(Color32::BLACK).font(FontId::new(16.0, egui::FontFamily::Name("Segoe Light".into())))) };
                ui.painter().text(pos2(right_limit, res.rect.center().y), Align2::RIGHT_CENTER, format!("{}", 300 as i16 - data.draft.len() as i16), FontId::proportional(12.0), if data.draft.len() > 300 { Color32::RED } else { Color32::GRAY });
            });

            let draft = TextEdit::multiline(&mut data.draft);
            draft.desired_width(ui.cursor().width()).text_color(Color32::BLACK).hint_text("Write Here").frame(false).font(TextStyle::Body).show(ui);

            if data.images.len() > 0 {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                        for path in &data.images {
                            let (_, rect) = ui.allocate_space(vec2(160.0, 90.0));
                            ui.painter().rect_filled(rect, Rounding::ZERO, BSKY_BLUE);
                        }
                    });
                });
            }

            if data.emoji_picker {
                let emojis_height = f32::min(500.0, ui.ctx().screen_rect().bottom() - (ui.cursor().top() + 100.0));
                ui.allocate_ui(vec2(ui.cursor().width(), emojis_height), |ui| {
                    let draft = TextEdit::singleline(&mut data.emoji_search);
                    draft.desired_width(ui.cursor().width()).text_color(Color32::BLACK).hint_text("Search...").frame(false).font(TextStyle::Body).show(ui);
                    let guh = ui.fonts(|f| {
                        f.lock()
                            .fonts
                            .font(&egui::FontId::new(10.0, egui::FontFamily::Monospace)) // size is arbitrary for getting the characters
                            .characters()
                            .iter()
                            .filter(|(chr, _fonts)| !chr.is_whitespace() && !chr.is_ascii_control() && !chr.is_alphanumeric() && (**chr as u16) > 0x25FF)
                            .map(|(chr, _)| {
                                (
                                    *chr,
                                    char_name(*chr),
                                )
                            })
                            .collect::<Vec<_>>()
                    });
                    egui::ScrollArea::vertical().max_height(emojis_height).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for chr in guh {
                                if data.emoji_search.len() > 0 && !chr.1.to_lowercase().contains(&data.emoji_search.to_lowercase()) {
                                    continue;
                                }
                                //another fucking custom button impl!
                                let sense = ui.allocate_response(vec2(30.0, 30.0), egui::Sense::click());
                                let col = if sense.hovered() { BSKY_BLUE } else { Color32::BLACK };
                                ui.painter().text(sense.rect.center() - vec2(0.0, 5.0), Align2::CENTER_CENTER, format!("{}", chr.0), FontId::monospace(24.0), col);

                                if sense.on_hover_text(char_name(chr.0)).clicked() {
                                    data.draft.push(chr.0);
                                }
                            }    
                        });
                        ui.allocate_space(vec2(ui.cursor().width(), 0.0));
                    });
                    

                });
            }

            ui.with_layout(Layout::left_to_right(egui::Align::Min), |buttons| {
                buttons.add_enabled_ui(data.images.len() < 4, |buttons| 'picker_logic: {
                    if circle_button(buttons, "\u{E114}", 15.0, 15.0).on_hover_text("Upload Image").clicked() {
                        let files = FileDialog::new()
                        .add_filter("image", &["png", "jpg", "jpeg", "webp"])
                        .add_filter("other", &["*"])
                        //.set_directory("/")
                        .pick_files();
                        if files.is_none() { break 'picker_logic; }
                        let files = files.unwrap();

                        let pre_count = data.images.len().clone(); // this needs to be cached, else it will live update
                        for (idx, file) in files.into_iter().enumerate() {
                            if idx > (3 - pre_count) { break 'picker_logic; }
                            data.images.push(file);
                        }
                    }
                });
                buttons.add_enabled_ui(false, |buttons| {
                    circle_button(buttons, "", 20.0, 15.0);
                });
                
                if circle_button(buttons, "\u{E234}", 20.0, 15.0).on_hover_text("Emoji Picker").clicked() {
                    data.emoji_picker = !data.emoji_picker;
                }
                
                let send_button_rect = buttons.cursor().with_min_x(right_limit - 90.0).with_max_x(right_limit).with_max_y(buttons.cursor().top() + 30.0);
                let send_button = buttons.add_enabled_ui(data.draft.len() > 0, |buttons| buttons.allocate_rect(send_button_rect, egui::Sense::click())).inner.on_hover_cursor(egui::CursorIcon::PointingHand);
                buttons.painter().rect_filled(send_button_rect, Rounding::ZERO, BSKY_BLUE.gamma_multiply(if data.draft.len() > 0 && data.draft.len() <= 300 { 1.0 } else { 0.5 }));

                buttons.painter().text(send_button_rect.center() - vec2(0.0, 2.0), Align2::CENTER_CENTER, "Post", FontId::proportional(10.0), Color32::WHITE);

                if send_button.clicked() {
                    let mut languages: Vec<String> = Vec::new();
                    languages.push("en".to_owned());

                    let record = BlueskyApiRecord::Post(BlueskyApiRecordPost { text: data.draft.clone(), created_at: Utc::now(), facets: None, reply: data.reply.clone(), embed: None, langs: Some(languages), labels: None, tags: None });
                    if data.images.len() > 0 {
                        backend.backend_commander.send(crate::bridge::FrontToBackMsg::CreateRecordWithMediaRequest(record, data.images.clone())).unwrap();
                    } else {
                        backend.backend_commander.send(crate::bridge::FrontToBackMsg::CreateRecordRequest(record)).unwrap();
                    }
                }
            });
        });
        if data.sending {
            SegoeBootSpinner::new().size(60.0).color(BSKY_BLUE).paint_at(ui, Rect::from_center_size(center, vec2(60.0, 60.0)));
        }
    }
}
