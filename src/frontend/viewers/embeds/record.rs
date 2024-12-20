use egui::{pos2, vec2, Align2, Color32, FontId, Id, Layout, Rect, Rounding, Stroke, UiBuilder};

use crate::{defs::bsky::embed, frontend::{pages::{thread::FrontendThreadView, FrontendMainView, MainViewProposition}, viewers::offset_time}, image::{ImageCache, LoadableImage}, widgets::spinner::SegoeBootSpinner, BSKY_BLUE};

use super::images::view_images;

pub fn view_record(ui: &mut egui::Ui, record: &crate::defs::bsky::embed::record::Variant, media_size: f32, img_cache: &ImageCache, new_view: &mut MainViewProposition) -> egui::Response {
    let content_rect = ui.cursor().shrink(8.0);
	let resp = ui.allocate_new_ui(UiBuilder::default().max_rect(content_rect), |quote| {
        quote.with_layout(Layout::left_to_right(egui::Align::Min), |embed| {
            embed.spacing_mut().item_spacing.x = 8.0;
            embed.spacing_mut().item_spacing.y = 4.0;
            embed.horizontal_wrapped(|embed| match record {
                embed::record::Variant::Record(record) => {
                    embed.with_layout(Layout::top_down(egui::Align::Min), |embed| {
                        embed.with_layout(Layout::left_to_right(egui::Align::Min), |name| {
                            let pfp_rect = name.allocate_space(vec2(30.0, 30.0)).1;
                            if let Some(avatar) = &record.author.avatar {
                                match img_cache.get_image(avatar) {
                                    LoadableImage::Unloaded | LoadableImage::Loading => {
                                        name.painter().rect_filled(pfp_rect, Rounding::ZERO, BSKY_BLUE);
                                        SegoeBootSpinner::new().size(25.0).color(Color32::WHITE).paint_at(name, pfp_rect);
                                    }
                                    LoadableImage::Loaded(texture_id, _) => {
                                        name.painter().image(texture_id, pfp_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                                    }
                                }
                            } else {
                                name.painter().rect_filled(pfp_rect, Rounding::ZERO, BSKY_BLUE);
                                name.painter().text(pfp_rect.center(), Align2::CENTER_CENTER, "", FontId::new(50.0, egui::FontFamily::Name("Segoe Symbols".into())), Color32::WHITE);
                            }

                            
                            let time_galley = name.painter().layout_no_wrap(offset_time(record.indexed_at), FontId::new(12.0, egui::FontFamily::Name("Segoe Light".into())), Color32::DARK_GRAY);
                            let seglight = FontId::new(15.0, egui::FontFamily::Name("Segoe Light".into()));
                            name.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                            // just gonna manually lay out the text here. RichText-ing just for size is kind of messy.
                            // this is different from how i do it elsewhere, but i really don't care.
                            if let Some(dn) = &record.author.display_name && dn.len() > 0 {
                                let dn_galley = name.painter().layout(dn.to_string(), FontId::proportional(15.0), name.style().visuals.text_color(), content_rect.width() - (30.0 + time_galley.rect.width() + (name.spacing().item_spacing.x * 2.0)));
                                let handle_galley = name.painter().layout(record.author.handle.clone(), seglight, name.style().visuals.weak_text_color(), content_rect.width() - (30.0 + dn_galley.rect.width() + time_galley.rect.width() + (name.spacing().item_spacing.x * 3.0)));

                                // allocations to increase the sub-ui width
                                name.allocate_space(vec2(dn_galley.rect.width() + handle_galley.rect.width() + time_galley.rect.width() + name.spacing().item_spacing.x * 3.0, 0.0));

                                name.painter().circle_filled(pos2(pfp_rect.max.x + dn_galley.rect.width() + handle_galley.rect.width() + name.spacing().item_spacing.x * 3.0, pfp_rect.center().y), 2.0, Color32::DARK_GRAY);
                                name.painter().galley(pos2(pfp_rect.max.x + dn_galley.rect.width() + handle_galley.rect.width() + name.spacing().item_spacing.x * 4.0, pfp_rect.max.y - 25.0), time_galley, Color32::DARK_GRAY);
                                
                                name.painter().galley(pfp_rect.max + vec2((name.spacing().item_spacing.x * 2.0) + dn_galley.rect.width(), -27.0), handle_galley, name.style().visuals.weak_text_color());
                                name.painter().galley(pfp_rect.max + vec2(name.spacing().item_spacing.x, -27.0), dn_galley, name.style().visuals.text_color());
                            } else {
                                let handle_galley = name.painter().layout(record.author.handle.clone(), seglight, name.style().visuals.text_color(), content_rect.width() - (30.0 + time_galley.rect.width() + (name.spacing().item_spacing.x * 2.0)));

                                // allocations to increase the sub-ui width
                                name.allocate_space(vec2(handle_galley.rect.width() + time_galley.rect.width() + name.spacing().item_spacing.x * 2.0, 0.0));

                                name.painter().circle_filled(pos2(pfp_rect.max.x + handle_galley.rect.width() + name.spacing().item_spacing.x * 2.0, pfp_rect.center().y), 2.0, Color32::DARK_GRAY);
                                name.painter().galley(pos2(pfp_rect.max.x + handle_galley.rect.width() + name.spacing().item_spacing.x * 3.0, pfp_rect.max.y - 25.0), time_galley, Color32::DARK_GRAY);
                                
                                name.painter().galley(pfp_rect.max + vec2((name.spacing().item_spacing.x * 2.0), -27.0), handle_galley, name.style().visuals.text_color());
                            }

                        });

                        match &record.value {
                            crate::backend::record::BlueskyApiRecord::Post(post) => {
                                if post.text.len() > 0 {
                                    embed.add(egui::Label::new(format!("{}", post.text)).selectable(false));    
                                }

                                if let Some(_) = &post.embed {
                                	embed.weak("Post has an embed");
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

    ui.painter().rect(resp.response.rect.expand(4.0), Rounding::ZERO, Color32::TRANSPARENT, Stroke::new(2.0, ui.style().visuals.weak_text_color()));
    resp.response
}