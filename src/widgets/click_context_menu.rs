use egui::{
    menu::{BarState, MenuRoot}, pos2, style::{WidgetVisuals, Widgets}, vec2, Color32, Id, InnerResponse, Margin, Rounding, Spacing, Stroke, Vec2, Vec2b, Visuals
};
#[derive(Clone)]
struct MetroContextMenu {
    pub origin: egui::Pos2,
    pub enabled: bool,
    pub visuals: Visuals,
    pub spacing: Spacing,
}

impl MetroContextMenu {
    fn new(res: &egui::Response) -> Self {
        MetroContextMenu {
            origin: res.rect.min,
            enabled: true,
            visuals: Visuals {
                widgets: Widgets {
                    
                    noninteractive: WidgetVisuals {
                        weak_bg_fill: Color32::LIGHT_GRAY, // this is disabled text color for some reason???
                        bg_fill: Color32::from_gray(248),
                        bg_stroke: Stroke::new(1.0, Color32::from_gray(190)), // separators, indentation lines
                        fg_stroke: Stroke::new(1.0, Color32::BLACK),  // normal text color
                        rounding: Rounding::ZERO,
                        expansion: 0.0,
                    },
                    inactive: WidgetVisuals {
                        weak_bg_fill: Color32::TRANSPARENT, // button background
                        bg_fill: Color32::from_gray(230),      // checkbox background
                        bg_stroke: Stroke::NONE,
                        fg_stroke: Stroke::new(1.0, Color32::BLACK), // button text
                        rounding: Rounding::same(2.0),
                        expansion: 0.0,
                    },
                    hovered: WidgetVisuals {
                        weak_bg_fill: Color32::from_gray(229),
                        bg_fill: Color32::from_gray(220),
                        bg_stroke: Stroke::NONE,
                        fg_stroke: Stroke::new(1.5, Color32::BLACK),
                        rounding: Rounding::ZERO,
                        expansion: 0.0,
                    },
                    active: WidgetVisuals {
                        weak_bg_fill: Color32::BLACK,
                        bg_fill: Color32::from_gray(165),
                        bg_stroke: Stroke::NONE,
                        fg_stroke: Stroke::new(2.0, Color32::WHITE),
                        rounding: Rounding::ZERO,
                        expansion: 0.0,
                    },
                    open: WidgetVisuals {
                        weak_bg_fill: Color32::TRANSPARENT,
                        bg_fill: Color32::from_gray(220),
                        bg_stroke: Stroke::NONE,
                        fg_stroke: Stroke::new(1.0, Color32::BLACK),
                        rounding: Rounding::ZERO,
                        expansion: 0.0,
                    }
                },
                ..Visuals::light()
            },
            spacing: Spacing {
                icon_width: 40.0,
                menu_width: 336.0,
                item_spacing: Vec2 { x: 0.0, y: 0.0, },
                button_padding: Vec2 { x: 18.0, y: 10.0 },
                interact_size: Vec2 { x: 336.0, y: 60.0 },
                ..Spacing::default()
            }
        }
    }
}

pub fn click_context_menu(response: egui::Response, add_contents: impl FnOnce(&mut egui::Ui)) -> Option<InnerResponse<()>> {
    let id = Id::new(format!("{:?}_CONTEXT_MENU", response.id));
    let data = response.ctx.data_mut(|writer| {
        writer.get_temp::<MetroContextMenu>(id)
    });

    // doesn't exist, wasn't requested to exist
    if data.is_none() && !response.clicked() { return None; }
    // clicked when exists, close
    if response.clicked() && data.is_some() { response.ctx.data_mut(|writer| { writer.remove::<MetroContextMenu>(id); }); return None; }
    // either already exists or was requested to exist
    let data = if response.clicked() { MetroContextMenu::new(&response) } else { data.unwrap() };

    let frame = egui::containers::Frame::default().fill(Color32::WHITE).rounding(Rounding::ZERO).stroke(Stroke::new(2.0, Color32::BLACK)).inner_margin(Margin::symmetric(0.0, 8.0));

    let window = egui::Window::new("")
    .frame(frame)
    .title_bar(false)
    .movable(false).auto_sized()
    .resizable(Vec2b { x: false, y: false })
    .min_width(336.0)
    .fixed_pos(pos2(response.rect.min.x, response.rect.max.y + 4.0))
    .show(&response.ctx, |jank| {
        *jank.visuals_mut() = data.visuals.clone();
        *jank.spacing_mut() = data.spacing.clone();
        jank.allocate_space(vec2(336.0, 0.0));
        add_contents(jank)
    }).unwrap();

    let clicked_off = response.ctx.input(|reader| {
        if let Some(pos) = reader.pointer.interact_pos() {
            !response.rect.contains(pos) && reader.pointer.primary_clicked()
        } else { false }
    });

    //moved away or clicked, close
    if response.rect.min != data.origin || clicked_off {
        response.ctx.data_mut(|writer| { writer.remove::<MetroContextMenu>(id); });
        return None;
    }

    response.ctx.data_mut(|writer| { writer.insert_temp(id,data); });

    None
}
