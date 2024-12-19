use egui::{pos2, vec2, Color32, Id, Layout, Rect, Rounding, Stroke, UiBuilder};

use crate::{defs::bsky::embed, frontend::pages::{thread::FrontendThreadView, FrontendMainView, MainViewProposition}, image::ImageCache};

use super::images::view_images;

pub fn view_record(ui: &mut egui::Ui, record: &crate::defs::bsky::embed::record::Variant, media_size: f32, img_cache: &ImageCache, new_view: &mut MainViewProposition) -> egui::Response {
	let resp = ui.allocate_new_ui(UiBuilder::default().max_rect(ui.cursor().shrink(8.0)), |quote| {
        quote.with_layout(Layout::left_to_right(egui::Align::Min), |embed| {
            embed.horizontal_wrapped(|embed| match record {
                embed::record::Variant::Record(record) => {
                    embed.with_layout(Layout::top_down(egui::Align::Min), |embed| {
                        embed.label(format!("{:?} ({})", record.author.display_name, record.author.handle));
                        embed.separator();
                        match &record.value {
                            crate::backend::record::BlueskyApiRecord::Post(post) => {
                                embed.add(egui::Label::new(format!("{:?}", post.text)).selectable(false));
                                if let Some(_) = &post.embed {
                                	embed.weak("There's an embed here, but I'm not rendering it, because rust is being a bitch (also it's a completely different struct for no reason at all)");
                                }
                            },
                            _ => {},
                        }
                    });
                }
                embed::record::Variant::NotFound(_) => {
                    embed.weak("Not Found");
                }
                embed::record::Variant::Blocked(_) => {
                    embed.weak("Blocked");
                }
                embed::record::Variant::Detached(_) => {
                    embed.weak("Detached Record");
                }
                embed::record::Variant::FeedGenerator(_) => {
                    embed.weak("Feed Generator");
                }
                embed::record::Variant::List(_) => {
                    embed.weak("List");
                }
                embed::record::Variant::Labeler(_) => {
                    embed.weak("Labeler");
                }
                embed::record::Variant::PackView(_) => {
                    embed.weak("PackView");
                }
            });
        });
    });

    if resp.response.interact(egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        match record {
            embed::record::Variant::Record(record) => {
                new_view.set(FrontendMainView::Thread(FrontendThreadView::new(record.uri.clone())));
            },
            embed::record::Variant::NotFound(_) |
            embed::record::Variant::Blocked(_) |
            embed::record::Variant::Detached(_) |
            embed::record::Variant::FeedGenerator(_) |
            embed::record::Variant::List(_) |
            embed::record::Variant::Labeler(_) |
            embed::record::Variant::PackView(_) => {},
        }
    }

    ui.painter().rect(resp.response.rect.expand(4.0), Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(2.0, ui.style().visuals.text_color()));
    resp.response
}