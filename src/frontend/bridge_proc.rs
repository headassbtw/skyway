use crate::{backend::{feeds::ActorFeedsResponse, main::{BlueskyLoginResponseError, BlueskyLoginResponseInfo}}, frontend::{main::{ClientFrontendFlyoutVariant, ClientFrontendModalVariant}, pages::{timeline::FrontendTimelineView, FrontendMainView}}};

use super::main::ClientFrontend;

impl ClientFrontend {
	pub fn proc(&mut self) {
		if let Ok(proc) = self.backend.frontend_listener.try_recv() {
            puffin::profile_scope!("Bridge processing");
            match proc {
                crate::bridge::BackToFrontMsg::BackendError(err) => {
                    self.info_modal("Backend Error", &err);
                }
                crate::bridge::BackToFrontMsg::LoginResponse(bluesky_login_response, profile, feeds) => {
                    self.profile = profile;
                    match bluesky_login_response {
                        crate::backend::main::BlueskyLoginResponse::Success(inf) => {
                            self.active = true;
                            self.authenticated = true;
                            self.view_stack.set(FrontendMainView::Timeline(FrontendTimelineView::new(feeds)));
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
                crate::bridge::BackToFrontMsg::TimelineResponse(tl) => match tl {
                    Ok(tl) => {
                        //TODO: FIX
                        if let Some(page) = self.view_stack.top() {
                            match page {
                                FrontendMainView::Timeline(ref mut data) => {
                                    if data.feed == 0 {
                                        data.timeline.cursor = tl.cursor;
                                        for post in tl.feed {
                                            data.timeline.feed.push(post);
                                        }
                                    } else {
                                        if let Some(feed) = data.feeds.get_mut(data.feed - 1) {
                                            feed.1.cursor = tl.cursor;
                                            for post in tl.feed {
                                                feed.1.feed.push(post);
                                            }    
                                        }
                                        
                                    }
                                    
                                }
                                _ => println!("fix this :)"),
                            }
                        }
                    }
                    Err(err) => self.error_modal("Failed to get timeline", err),
                }
                crate::bridge::BackToFrontMsg::KeyringFailure(reason) => self.info_modal("OS Keyring Failure", &reason),
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
                    Err(err) => self.error_modal("Failed to create record", err),
                },
                crate::bridge::BackToFrontMsg::ProfileResponse(id, profile) => {
                    if let Some(page) = self.view_stack.top() {
                        match page {
                            FrontendMainView::Profile(data) => {
                                if data.id_cmp == id {
                                    match profile {
                                        Ok(profile) => {
                                            data.profile_data = Some(profile);
                                            data.loading = false;
                                        },
                                        Err(err) => self.error_modal("Failed to get profile", err),
                                    }
                                }
                            },
                            _ => println!("bridge target missed"),
                        }
                    }
                }
                crate::bridge::BackToFrontMsg::RecordDeletionResponse(data) => {
                    if let Err(err) = data {
                        self.error_modal("Failed to delete record", err)
                    }
                }
                crate::bridge::BackToFrontMsg::ThreadResponse(uri, res) => {
                    if let Some(page) = self.view_stack.top() {
                        match page {
                            FrontendMainView::Thread(data) => {
                                if data.id_cmp == uri {
                                    match res {
                                        Ok(thread) => {
                                            data.data = Some(crate::defs::bsky::feed::defs::ThreadPostVariant::ThreadView(thread.thread));
                                            data.loading = false;
                                        }
                                        Err(err) => self.error_modal("Failed to get thread", err),
                                    }
                                }
                            },
                            _ => println!("fix this, use a callback for thread responses pleeeeeeeeeeease"),
                        }
                    }
                }
            }
        }
	}
}