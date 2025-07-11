use crate::defs::bsky::actor::defs::ProfileView;
use crate::frontend::pages::{profile::FrontendProfileView, FrontendMainView, MainViewProposition};
use crate::frontend::viewers::profile_picture::profile_picture_viewer;
use crate::image::ImageCache;
use eframe::emath::vec2;
use eframe::epaint::FontId;
use egui::{pos2, Color32, CursorIcon, Layout, Response, Ui, UiBuilder};
use std::sync::Arc;

pub fn profile_viewer(ui: &mut Ui, profile: &Arc<ProfileView>, img_cache: &ImageCache, new_view: &mut MainViewProposition) -> Response {
    ui.allocate_new_ui(UiBuilder::new().layout(Layout::left_to_right(egui::Align::TOP)), |ui| {
        let pfp = profile_picture_viewer(ui, &profile.avatar, [60.0, 60.0], img_cache);
        ui.vertical(|ui| {
            let seglight = egui::FontFamily::Name("Segoe Light".into());
            let dn_galley = match &profile.display_name {
                Some(display_name) => {
                    if display_name.is_empty() {
                        None
                    } else {
                        Some(ui.painter().layout(
                            display_name.clone(),
                            FontId::proportional(20.0),
                            ui.style().visuals.text_color(),
                            ui.cursor().width() - (ui.spacing().item_spacing.x),
                        ))
                    }
                }
                None => None,
            };
            let (handle_font, handle_color) = if dn_galley.is_some() {
                (FontId::new(20.0, seglight), ui.style().visuals.weak_text_color())
            } else {
                (FontId::proportional(20.0), ui.style().visuals.text_color())
            };
            let handle_galley = ui
                .painter()
                .layout(profile.handle.clone(), handle_font, handle_color, ui.cursor().width());

            let rtn = ui.allocate_response(
                vec2(
                    match &dn_galley {
                        None => 0.0,
                        Some(galley) => galley.rect.width() + ui.spacing().item_spacing.x,
                    } + handle_galley.rect.width(),
                    f32::max(
                        handle_galley.rect.height(),
                        match &dn_galley {
                            None => 0.0,
                            Some(galley) => galley.rect.height(),
                        },
                    ),
                ),
                egui::Sense::click(),
            );

            match dn_galley {
                None => ui.painter().galley(rtn.rect.min, handle_galley, Color32::RED),
                Some(dn) => {
                    ui.painter().galley(
                        pos2(rtn.rect.min.x + ui.spacing().item_spacing.x + dn.rect.width(), rtn.rect.min.y),
                        handle_galley,
                        Color32::RED,
                    );
                    ui.painter().galley(rtn.rect.min, dn, Color32::RED);
                }
            }

            if let Some(bio) = &profile.description {
                ui.label(bio);
            }

            if rtn.on_hover_cursor(CursorIcon::PointingHand).clicked() || pfp.clicked() {
                new_view.set(FrontendMainView::Profile(FrontendProfileView::new(profile.did.clone())));
            }
        });
    }).response
}
