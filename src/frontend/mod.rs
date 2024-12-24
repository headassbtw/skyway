use egui::{vec2, Align2, Color32, FontId, Stroke};

pub mod flyouts;
pub mod main;
pub mod modals;
pub mod pages;
pub mod viewers;

fn circle_button(ui: &mut egui::Ui, icon: &str, icon_size: f32, radius: f32) -> egui::Response {
    puffin::profile_function!();

    let info = {
        puffin::profile_scope!("Button alloc");
        ui.allocate_response(vec2(radius * 2.0, radius * 2.0), egui::Sense::click())
        //ui.add_sized(vec2(radius * 2.0,radius * 2.0), egui::Button::new("").frame(false))
    };
    if !ui.is_rect_visible(info.rect) {
        return info;
    }
    let color = ui.style().interact(&info).fg_stroke.color;
    {
        puffin::profile_scope!("Circle");
        ui.painter().circle(info.rect.center(), radius - 1.0, Color32::TRANSPARENT, Stroke::new(2.0, color));
    }
    {
        puffin::profile_scope!("Icon");
        ui.painter().text(info.rect.center() - vec2(0.0, 1.2), Align2::CENTER_CENTER, icon, FontId::proportional(icon_size), color);
    }
    info.on_hover_cursor(egui::CursorIcon::PointingHand)
}
