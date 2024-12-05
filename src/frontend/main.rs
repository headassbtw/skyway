use egui::{pos2, style::HandleShape, vec2, Align2, Button, Color32, FontFamily, FontId, Id, LayerId, Layout, Rect, Rounding, Stroke, TextStyle, UiBuilder, UiStackInfo};
use std::{collections::BTreeMap, sync::{Arc, Mutex}};

use crate::{
    backend::{
        main::{BlueskyLoginResponseError, BlueskyLoginResponseInfo},
        profile::BlueskyApiProfile,
        BlueskyApiError,
    }, bridge::Bridge, frontend::pages::thread, image::ImageCache, widgets::spinner::SegoeBootSpinner
};

const BSKY_BLUE: Color32 = Color32::from_rgb(32, 139, 254);

use super::{flyouts::composer::ComposerFlyout, modals::important_error::ImportantErrorModal, pages::{timeline::FrontendTimelineView, FrontendMainView, FrontendMainViewStack}};
#[derive(serde::Deserialize, serde::Serialize)]
pub enum ClientFrontendPage {
    LandingPage,
    TimelinePage,
    FeedPage(String),
    ProfilePage(String),
    SettingsPage,
}

pub enum ClientFrontendModalVariant {
    LoginModal(crate::frontend::modals::login::LoginModal),
    ImportantErrorModal(crate::frontend::modals::important_error::ImportantErrorModal),
}

pub enum ClientFrontendFlyoutVariant {
    PostComposerFlyout(ComposerFlyout),
}

pub struct ClientFrontendFlyout {
    /// Keeps track of animation state
    ctx: egui::Context,
    /// Main object, the actual flyout
    pub main: Option<ClientFrontendFlyoutVariant>,
    /// egui keeps track of the animation state, however we still need to know when to render it when closing,
    /// so this bool keeps track of that.
    ///
    ///
    /// main: None means it's not there, don't try AT ALL
    /// main: Some, closing: false means it's opening or opened
    /// Main: Some, closing: true means it's closing
    pub closing: bool,
}

pub struct ClientFrontendModal {
    ctx: egui::Context,
    pub main: Option<ClientFrontendModalVariant>,
}

//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(default)]
pub struct ClientFrontend {
    pub ctx: egui::Context,
    pub modal: ClientFrontendModal,
    pub flyout: ClientFrontendFlyout,
    pub backend: Bridge,
    pub image: ImageCache,
    pub draw_grid: bool,
    pub show_egui_settings: bool,
    pub active: bool,
    pub authenticated: bool,
    pub profile: Option<BlueskyApiProfile>,

    pub view_stack: FrontendMainViewStack,
}

impl ClientFrontend {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        //egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.all_styles_mut(|style| {
            // Global styles
            let text_styles: BTreeMap<_, _> = [(TextStyle::Heading, FontId::new(30.0, FontFamily::Proportional)), (TextStyle::Name("MetroHeading".into()), FontId::new(40.0, FontFamily::Name("Segoe Light".into()))), (TextStyle::Body, FontId::new(11.0, FontFamily::Proportional)), (TextStyle::Monospace, FontId::new(11.0, FontFamily::Proportional)), (TextStyle::Button, FontId::new(11.0, FontFamily::Proportional)), (TextStyle::Small, FontId::new(7.0, FontFamily::Proportional))].into();

            style.visuals.widgets.noninteractive.rounding = Rounding::ZERO;
            style.visuals.widgets.inactive.rounding = Rounding::ZERO;
            style.visuals.widgets.hovered.rounding = Rounding::ZERO;
            style.visuals.widgets.active.rounding = Rounding::ZERO;
            style.visuals.widgets.open.rounding = Rounding::ZERO;
            style.spacing.item_spacing.y = 20.0;
            style.visuals.handle_shape = HandleShape::Rect { aspect_ratio: 1.0 };
            style.text_styles = text_styles;
        });
        cc.egui_ctx.style_mut_of(egui::Theme::Light, |style| {
            style.visuals.widgets.noninteractive.fg_stroke.color = Color32::BLACK;
        });
        cc.egui_ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.visuals.extreme_bg_color = Color32::from_gray(37);
            style.visuals.widgets.noninteractive.fg_stroke.color = Color32::WHITE;
            style.visuals.widgets.noninteractive.bg_fill = Color32::BLACK;
            //style.visuals.window_fill = Color32::BLACK;
            //style.visuals.panel_fill = Color32::BLACK;

            //BUTTON SHIT!
            style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
            style.visuals.widgets.inactive.bg_stroke = Stroke::new(2.0, Color32::WHITE);
            style.visuals.widgets.inactive.expansion = -2.0;

            style.visuals.widgets.hovered.bg_fill = Color32::from_white_alpha(25);
            style.visuals.widgets.hovered.weak_bg_fill = Color32::from_white_alpha(25);
            style.visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, Color32::WHITE);
            style.visuals.widgets.hovered.expansion = -2.0;
        });

        let mut fonts = egui::FontDefinitions::empty();
        fonts.font_data.insert("Segoe UI".to_owned(), {
            let mut font = egui::FontData::from_static(include_bytes!("../../segoeui.ttf"));
            font.tweak.scale = 1.43;
            font.tweak.baseline_offset_factor = 0.04;
            font
        });
        fonts.font_data.insert("Segoe Light".to_owned(), {
            let mut font = egui::FontData::from_static(include_bytes!("../../segoeuil.ttf"));
            font.tweak.scale = 1.43;
            font.tweak.baseline_offset_factor = 0.04;
            font
        });
        fonts.font_data.insert("Segoe Emojis".to_owned(), {
            let mut font = egui::FontData::from_static(include_bytes!("../../seguiemj.ttf"));
            font.tweak.baseline_offset_factor = 0.4;
            font
        });

        fonts.font_data.insert("Droid Sans JPN".to_owned(), {
            let mut font = egui::FontData::from_static(include_bytes!("../../DroidSansJapanese.ttf"));
            font.tweak.y_offset_factor = 0.075;
            font
        });

        fonts.font_data.insert("Segoe Boot".to_owned(), egui::FontData::from_static(include_bytes!("../../segoe_slboot.ttf")));
        fonts.font_data.insert("Segoe Symbols".to_owned(), egui::FontData::from_static(include_bytes!("../../seguisym.ttf")));

        fonts.families.insert(egui::FontFamily::Name("Segoe Symbols".into()), vec!["Segoe Symbols".to_owned()]);
        fonts.families.insert(egui::FontFamily::Name("Segoe Boot".into()), vec!["Segoe Boot".to_owned()]);
        fonts.families.insert(egui::FontFamily::Name("Segoe Light".into()), vec!["Segoe Light".to_owned()]);

        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "Segoe UI".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(1, "Droid Sans JPN".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(2, "Segoe Emojis".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(3, "Segoe Symbols".to_owned());

        fonts.families.get_mut(&egui::FontFamily::Name("Segoe Light".into())).unwrap().insert(1, "Droid Sans JPN".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Name("Segoe Light".into())).unwrap().insert(2, "Segoe Emojis".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Name("Segoe Light".into())).unwrap().insert(3, "Segoe Symbols".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        /*if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }*/
        Self {
            ctx: cc.egui_ctx.clone(),
            modal: ClientFrontendModal { ctx: cc.egui_ctx.clone(), main: None },
            flyout: ClientFrontendFlyout { ctx: cc.egui_ctx.clone(), main: None, closing: false },
            backend: Bridge::new(cc.egui_ctx.clone()),
            image: ImageCache::new(cc.egui_ctx.clone()),
            draw_grid: false,
            show_egui_settings: false,
            active: false,
            authenticated: false,
            profile: None,
            view_stack: FrontendMainViewStack::new(cc.egui_ctx.clone(), FrontendMainView::Login()),
        }
    }
}

fn draw_unit_grid(ctx: &egui::Context) {
    puffin::profile_function!();
    let col = Color32::from_white_alpha(16);
    let width = ctx.screen_rect().width() as i32;
    let height = ctx.screen_rect().height() as i32;
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    'a: loop {
        ctx.layer_painter(LayerId::background()).line_segment([pos2(x as f32, 0.0), pos2(x as f32, height as f32)], Stroke::new(2.0, col));

        if x > width {
            break 'a;
        } else {
            x += 20;
        }
    }
    'b: loop {
        ctx.layer_painter(LayerId::background()).line_segment([pos2(0.0, y as f32), pos2(width as f32, y as f32)], Stroke::new(2.0, col));

        if y > height {
            break 'b;
        } else {
            y += 20;
        }
    }
}

fn ease_out_expo(x: f32) -> f32 {
    if x == 1.0 {
        return 1.0;
    } else {
        return 1.0 - f32::powf(2.0, -10.0 * x);
    }
}

impl ClientFrontend {
    /// Shortcut for `set_modal` with `ClientFrontendModal::ImportantErrorModal`
    pub fn info_modal(&mut self, heading: &str, body: &str) {
        self.modal.set(ClientFrontendModalVariant::ImportantErrorModal(ImportantErrorModal::new(heading.into(), body.into())));
    }
}

impl ClientFrontendFlyout {
    pub fn set(&mut self, to: ClientFrontendFlyoutVariant) {
        self.closing = false;
        self.main = Some(to);
        self.ctx.animate_bool_with_time(Id::new("flyout shift"), false, 0.0);
    }

    pub fn close(&mut self) {
        self.closing = true;
    }

    /// Should everything under the flyout be interactable?
    /// ALSO RUNS ANIMAITON LOGIC!
    /// paramds: should render, should let underneath interact, state
    pub fn get_animation_state(&mut self) -> (bool, bool, f32) {
        if self.main.is_none() {return (false, true, 0.0); }
        let state = self.ctx.animate_bool_with_time_and_easing(Id::new("flyout shift"), self.main.is_some() && !self.closing, 1.2, ease_out_expo);
        if state == 0.0 && self.closing{
            self.main = None;
            self.closing = false;
        }
        return (true, state < 0.2, state);
    }

    pub fn render(&mut self, ui: &mut egui::Ui, profile: &Option<BlueskyApiProfile>, backend: &Bridge, image: &ImageCache) -> &str {
        if let Some(flyout) = &mut self.main {
            match flyout {
                ClientFrontendFlyoutVariant::PostComposerFlyout(data) => {
                    ClientFrontendFlyoutVariant::post_composer(ui, data, profile, image, backend);
                    if data.reply.is_some() {
                        return "Reply"
                    }
                    return "New Post"

                },
            }
        }
        "Unhandled Flyout"
    }
}

impl ClientFrontendModal {
    pub fn set(&mut self, to: ClientFrontendModalVariant) {
        self.ctx.animate_bool_with_time_and_easing(Id::new("modal shift"), false, 0.0, ease_out_expo);
        self.main = Some(to);
    }

    pub fn close(&mut self) {
        self.main = None;
    }

    pub fn get_animation_state(&self) -> (bool, f32) {
        let state = self.ctx.animate_bool_with_time_and_easing(Id::new("modal shift"), self.main.is_some(), 1.2, ease_out_expo); 
        return (state < 0.2, state);
    }
}

impl eframe::App for ClientFrontend {
    fn save(&mut self, _: &mut dyn eframe::Storage) {
        //eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        puffin::profile_function!();

        if let Ok(proc) = self.backend.frontend_listener.try_recv() {
            puffin::profile_scope!("Bridge processing");
            match proc {
                crate::bridge::BackToFrontMsg::LoginResponse(bluesky_login_response, profile) => {
                    self.profile = profile;
                    match bluesky_login_response {
                        crate::backend::main::BlueskyLoginResponse::Success(_, _) => {
                            self.active = true;
                            self.authenticated = true;
                            self.view_stack.set(FrontendMainView::Timeline(FrontendTimelineView::new()));
                            self.modal.close();
                        }
                        crate::backend::main::BlueskyLoginResponse::Info(variant) => match variant {
                            BlueskyLoginResponseInfo::WasntLoggedIn => self.active = true,
                            BlueskyLoginResponseInfo::TwoFactorTokenRequired => self.info_modal("Login Error", "Your account has two-factor authenticaiton enabled. This is currently not supported."),
                        },
                        crate::backend::main::BlueskyLoginResponse::Error(variant) => match variant {
                            BlueskyLoginResponseError::Generic(reason) => self.info_modal("Generic Backend Error", &reason),
                            BlueskyLoginResponseError::Network(reason) => self.info_modal("Network Error", &reason),
                            BlueskyLoginResponseError::InvalidRequest => self.info_modal("Invalid Request", ""),
                            BlueskyLoginResponseError::ExpiredToken => self.info_modal("Token Expired", "Cached login has expired. Please log in again."),
                            BlueskyLoginResponseError::InvalidToken => self.info_modal("Invalid Token", "Cached login was invalid. Please log in again."),
                            BlueskyLoginResponseError::AccountTakenDown => self.info_modal("Account Taken Down", ""),
                            BlueskyLoginResponseError::AccountSuspended => self.info_modal("Account Suspended", ""),
                            BlueskyLoginResponseError::AccountInactive => self.info_modal("Account Inactive", ""),
                            BlueskyLoginResponseError::AccountDeactivated => self.info_modal("Account Deactivated", ""),
                            BlueskyLoginResponseError::Unauthorized => {
                                if let Some(modal) = &mut self.modal.main {
                                    match modal {
                                        ClientFrontendModalVariant::LoginModal(login_modal) => {
                                            login_modal.error_msg = "That password is incorrect.".into();
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        },
                    };
                }
                crate::bridge::BackToFrontMsg::TimelineResponse(tl) => {
                    if let Ok(tl) = tl {
                        //TODO: FIX
                        println!("Recieved {} posts", tl.feed.len());
                        if let Some(page) = self.view_stack.top() {
                            match page {
                                FrontendMainView::Timeline(ref mut data) => {
                                    data.timeline_cursor = tl.cursor;
                                    for post in tl.feed {
                                        data.timeline.push(post);
                                    }
                                },
                                FrontendMainView::Login() |
                                FrontendMainView::Thread(_) |
                                FrontendMainView::Profile(_) => {
                                    println!("fix this :)");
                                    todo!();
                                },
                            }
                        }
                    }
                }
                crate::bridge::BackToFrontMsg::KeyringFailure(reason) => {
                    self.info_modal("OS Keyring Failure", &reason);
                }
                crate::bridge::BackToFrontMsg::RecordCreationResponse(data) => match data {
                    Ok(_) => {
                        if let Some(flyout) = &mut self.flyout.main {
                            match flyout {
                                ClientFrontendFlyoutVariant::PostComposerFlyout(flyout) => {
                                    flyout.draft = String::new();
                                    flyout.sending = false;
                                    self.flyout.close();
                                }
                            }
                        }
                    }
                    Err(err) => {
                        let s = match err {
                            BlueskyApiError::BadRequest(error) => format!("{}: {}", error.error, error.message),
                            BlueskyApiError::Unauthorized(error) => format!("{}: {}", error.error, error.message),
                            BlueskyApiError::NetworkError(error) => format!("Network Error: {}", error),
                            BlueskyApiError::ParseError(error) => format!("Parse Error: {}", error),
                        };
                        self.info_modal("Failed to delete record", &s);
                    },
                },
                crate::bridge::BackToFrontMsg::ProfileResponse(id, profile) => {
                    if let Some(page) = self.view_stack.top() {
                        match page {
                            FrontendMainView::Login() |
                            FrontendMainView::Timeline(_) |
                            FrontendMainView::Thread(_) => todo!(),
                            FrontendMainView::Profile(data) => {
                                if data.id_cmp == id {
                                    if let Ok(profile) = profile {
                                        data.profile_data = Some(profile);
                                        data.loading = false;
                                    } else if let Err(err) = profile {
                                        self.info_modal("Failed to get profile", &format!("{:?}", err));
                                    }
                                }
                            },
                        }
                    }
                },
                crate::bridge::BackToFrontMsg::RecordDeletionResponse(data) => {
                    if let Err(err) = data {
                        let s = match err {
                            BlueskyApiError::BadRequest(error) => format!("{}: {}", error.error, error.message),
                            BlueskyApiError::Unauthorized(error) => format!("{}: {}", error.error, error.message),
                            BlueskyApiError::NetworkError(error) => format!("Network Error: {}", error),
                            BlueskyApiError::ParseError(error) => format!("Parse Error: {}", error),
                        };
                        self.info_modal("Failed to delete record", &s);
                    }
                },
                crate::bridge::BackToFrontMsg::ThreadResponse(uri, res) => {
                    if let Some(page) = self.view_stack.top() {
                        match page {
                            FrontendMainView::Login() |
                            FrontendMainView::Timeline(_) |
                            FrontendMainView::Profile(_) => todo!(),
                            FrontendMainView::Thread(data) => {
                                if data.id_cmp == uri {
                                    match res {
                                        Ok(thread) => {
                                            data.data = Some(thread);
                                            data.loading = false;
                                        },
                                        Err(err) => {
                                            self.info_modal("Failed to get thread", &format!("{:?}", err));
                                        },
                                    }
                                }
                            },
                        }
                    }
                    
                },
            }
        }

        if self.show_egui_settings {
            puffin::profile_scope!("egui settings");
            egui::Window::new("Guh!").show(ctx, |ui| {
                ctx.settings_ui(ui);
            });
        }

        if cfg!(debug_assertions) {
            egui::TopBottomPanel::top("top_panel").show_separator_line(false).max_height(20.0).min_height(20.0).show(ctx, |ui| {
                puffin::profile_scope!("Menu bar");
                egui::menu::bar(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.menu_button("Debug Build", |ui| {
                            ui.style_mut().spacing.item_spacing.y = 0.0;
                            egui::widgets::global_theme_preference_buttons(ui);
                            if ui.button("Toggle Unit Grid").clicked() {
                                self.draw_grid = !self.draw_grid;
                            }
                            if ui.button("Toggle egui settings").clicked() {
                                self.show_egui_settings = !self.show_egui_settings;
                            }
                            if ui.add_enabled(self.modal.main.is_some(), Button::new("Force close modal")).clicked() {
                                self.modal.close();
                            }
                            if ui.add_enabled(true, Button::new("Trigger network error")).clicked() {
                                self.info_modal("Network Error", "Test");
                            }
                            if ui.add_enabled(true, Button::new("Trigger composer flyout")).clicked() {
                                self.flyout.set(ClientFrontendFlyoutVariant::PostComposerFlyout(ComposerFlyout::new()));
                            }
                            if ui.add_enabled(self.authenticated, Button::new("Get Timeline")).clicked() {
                                //self.backend.backend_commander.send(crate::bridge::FrontToBackMsg::GetTimelineRequest(self.timeline_cursor.clone(), None)).unwrap();
                            }
                        });
                    });
                });
            });
        }

        let mut margin = egui::Margin::symmetric(120.0, 140.0);
        //margin.right = 0.0;
        margin.bottom = 0.0;

        let frame = egui::containers::Frame::central_panel(&ctx.style()).inner_margin(margin);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            puffin::profile_scope!("Main panel");
            ui.set_clip_rect(ctx.screen_rect());

            if self.draw_grid {
                draw_unit_grid(&ctx);
            }
            if self.active {
                ui.add_enabled_ui(self.modal.main.is_none() && (self.flyout.get_animation_state().1), |contents| {
                    self.view_stack.render(contents, &self.backend, &self.image, &mut self.flyout, &mut self.modal);
                    
                    /*match self.page {
                        ClientFrontendPage::LandingPage => self.landing_page(contents),
                        ClientFrontendPage::TimelinePage => self.timeline_page(contents),
                        _ => {
                            contents.label("Unimplemented page");
                        }
                    }*/
                });
            } else {
                puffin::profile_scope!("Loading Screen");
                ui.painter().rect_filled(ctx.screen_rect().expand(60.0), Rounding::ZERO, BSKY_BLUE);
                //200px image
                //20px padding
                //40px throbber

                //90px down for throbber
                //30px up for image

                let throbber_center = ctx.screen_rect().center() + vec2(0.0, 110.0);
                let throbber_rect = egui::Rect::from_center_size(throbber_center, vec2(40.0, 40.0));

                let image_center = ctx.screen_rect().center() - vec2(0.0, 30.0);
                let _image_rect = egui::Rect::from_center_size(image_center, vec2(200.0, 200.0));

                SegoeBootSpinner::new().size(40.0).color(Color32::WHITE).paint_at(ui, throbber_rect);
                //ui.painter().rect_filled(image_rect, Rounding::ZERO, self.contrast_a());
                ui.painter().text(image_center, Align2::CENTER_CENTER, "", FontId::new(180.0, egui::FontFamily::Name("Segoe Symbols".into())), Color32::WHITE);
            }

            const FLYOUT_WIDTH: f32 = 500.0;

            let anim_state = self.flyout.get_animation_state();

            if anim_state.0 {
                puffin::profile_scope!("Flyout");
                ui.add_enabled_ui(self.modal.main.is_none(), |ui| {
                    let offset = FLYOUT_WIDTH - anim_state.2 * FLYOUT_WIDTH;
                    let content_rect = ctx.screen_rect().with_min_x(ctx.screen_rect().right() - FLYOUT_WIDTH);
                    let content_rect = content_rect.translate(vec2(offset, 0.0));
                    ui.painter().rect_filled(content_rect, Rounding::ZERO, Color32::WHITE);
                    ui.painter().vline(content_rect.left(), 0.0..=content_rect.bottom(), Stroke::new(2.0, BSKY_BLUE));

                    let content = UiBuilder::new().max_rect(content_rect.with_min_y(100.0).shrink2(vec2(30.0, 10.0))).ui_stack_info(UiStackInfo::new(egui::UiKind::Popup));
                    let content = if anim_state.1 { content.disabled() } else { content };

                    ui.painter().rect_filled(content_rect.with_max_y(content_rect.top() + 100.0), Rounding::ZERO, BSKY_BLUE);

                    let back_button_rect = Rect { min: pos2(content_rect.left() + 40.0, content_rect.top() + 40.0), max: pos2(content_rect.left() + 80.0, content_rect.top() + 80.0) };

                    let back_button = ui.allocate_rect(back_button_rect, egui::Sense::click());
                    ui.painter().text(back_button_rect.center(), Align2::CENTER_CENTER, "\u{E0BA}", FontId::new(40.0, egui::FontFamily::Name("Segoe Symbols".into())), Color32::WHITE);

                    if back_button.clicked() || ctx.input(|state| { state.key_pressed(egui::Key::Escape) }) {
                        self.flyout.close();
                    }

                    if !anim_state.1 {
                        let click_off_rect = ctx.screen_rect().with_max_x(content_rect.left());
                        if ui.allocate_rect(click_off_rect, egui::Sense::click()).clicked() {
                            self.flyout.close();
                        }
                    }

                    ui.style_mut().visuals.selection.bg_fill = Color32::from_rgb(144, 209, 255); // default light mode selection fill

                    ui.allocate_new_ui(content, |flyout_contents| {
                        let title = self.flyout.render(flyout_contents, &self.profile, &self.backend, &self.image);
                        flyout_contents.painter().text(back_button_rect.right_bottom() + vec2(20.0, 0.0), Align2::LEFT_BOTTOM, title, FontId::new(30.0, egui::FontFamily::Name("Segoe Light".into())), Color32::WHITE);
                    });
                    

                    
                });
            }

            if let Some(modal) = &self.modal.main {
                puffin::profile_scope!("Modal");
                let offset = 100.0 - ctx.animate_bool_with_time_and_easing(Id::new("modal shift"), true, 1.2, ease_out_expo) * 100.0;
                ui.painter().rect_filled(ctx.screen_rect(), Rounding::ZERO, Color32::from_black_alpha(64));
                let banner_rect = ctx.screen_rect().shrink2(vec2(0.0, (ctx.screen_rect().height() - 400.0) / 2.0));
                ui.painter().rect_filled(banner_rect, Rounding::ZERO, Color32::from_gray(37));
                let content_rect = banner_rect.shrink2(vec2((banner_rect.width() - 600.0) / 2.0, 25.0));
                let content_rect = content_rect.with_min_x(content_rect.left() + offset).with_max_x(content_rect.right() + offset);
                let content = UiBuilder::new().max_rect(content_rect);
                ui.style_mut().visuals.extreme_bg_color = Color32::WHITE;
                ui.style_mut().visuals.selection.stroke = Stroke::NONE;
                ui.style_mut().visuals.selection.bg_fill = Color32::from_rgb(144, 209, 255); // default light mode selection fill

                // rust be good chllenge (IMPOSSIBLE)
                match modal {
                    ClientFrontendModalVariant::LoginModal(_) => {
                        ui.allocate_new_ui(content, |modal_contents| {
                            self.login_modal(modal_contents);
                        });
                    }
                    ClientFrontendModalVariant::ImportantErrorModal(_) => {
                        ui.allocate_new_ui(content, |modal_contents| {
                            self.important_error_modal(modal_contents);
                        });
                    }
                };
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.image.shutdown();
        self.backend.backend_commander.send(crate::bridge::FrontToBackMsg::ShutdownMessage).unwrap();
    }
}
