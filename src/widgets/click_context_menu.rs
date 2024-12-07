use egui::{
    menu::{BarState, MenuRoot},
    Id, InnerResponse,
};

pub fn click_context_menu(response: egui::Response, add_contents: impl FnOnce(&mut egui::Ui)) -> Option<InnerResponse<()>> {
    let id = Id::new(format!("{:?}_CONTEXT_MENU", response.id));
    let mut state = BarState::load(&response.ctx, id);

    MenuRoot::stationary_click_interaction(&response, &mut state);
    let inner_response = state.show(&response, add_contents);

    state.store(&response.ctx, id);
    inner_response
}
