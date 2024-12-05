use crate::backend::{
    main::BlueskyLoginResponse, profile::BlueskyApiProfile, record::{BlueskyApiCreateRecordResponse, BlueskyApiDeleteRecordResponse, BlueskyApiRecord}, responses::timeline::{BlueskyApiPostView, BlueskyApiTimelineResponse, BlueskyApiTimelineResponseObject}, thread::BlueskyApiGetThreadResponse, BlueskyApiError, ClientBackend
};
use anyhow::Result;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};

pub enum FrontToBackMsg {
    ShutdownMessage,
    LoginRequestStandard(String, String),
    LoginRequest2FA(String, String, String),

    GetTimelineRequest(Option<String>, Option<u32>),
    GetProfileRequest(String),
    GetThreadRequest(String),

    CreateRecordRequest(BlueskyApiRecord),
    CreateRecordUnderPostRequest(BlueskyApiRecord, Arc<Mutex<BlueskyApiPostView>>),

    DeleteRecordRequest(String, String),
    DeleteRecordUnderPostRequest(String, String, Arc<Mutex<BlueskyApiPostView>>),
}

pub enum BackToFrontMsg {
    LoginResponse(BlueskyLoginResponse, Option<BlueskyApiProfile>),
    TimelineResponse(Result<BlueskyApiTimelineResponse, BlueskyApiError>),
    KeyringFailure(String),
    RecordCreationResponse(Result<BlueskyApiCreateRecordResponse, BlueskyApiError>),
    RecordDeletionResponse(Result<BlueskyApiDeleteRecordResponse, BlueskyApiError>),
    ProfileResponse(String, Result<BlueskyApiProfile, BlueskyApiError>),
    ThreadResponse(String, Result<BlueskyApiGetThreadResponse, BlueskyApiError>),
}

pub struct Bridge {
    pub frontend_listener: Receiver<BackToFrontMsg>,
    pub backend_commander: Sender<FrontToBackMsg>,
}

impl Bridge {
    pub fn new(ctx: egui::Context) -> Self {
        let (backend_commander, backend_listener) = std::sync::mpsc::channel();
        let (frontend_commander, frontend_listener) = std::sync::mpsc::channel();
        let ctx_burn = ctx.clone();
        tokio::task::spawn(async move {
            //let die_fallback_transmittter = backend_responder.clone();
            //panic::set_hook(Box::new( |_| {}));
            let result = Self::run(backend_listener, frontend_commander, ctx_burn).await;
            if let Err(result) = result {
                panic!("Bridge failed! {}", result);
            }
        });

        Self { frontend_listener, backend_commander }
    }

    async fn run(rx: Receiver<FrontToBackMsg>, tx: Sender<BackToFrontMsg>, ctx: egui::Context) -> Result<()> {
        let mut api = ClientBackend::new();

        let vault = keyring::Entry::new("com.headassbtw.metro.bluesky", "refreshJwt");
        if let Ok(vault) = vault {
            if let Ok(token) = vault.get_password() {
                let login_response = api.login_refresh(token).await;
                match &login_response {
                    BlueskyLoginResponse::Success(_, refresh) => {
                        if let Err(error) = vault.set_password(refresh) {
                            tx.send(BackToFrontMsg::KeyringFailure(format!("Error when caching login: {:?}", error)))?;
                        }
                    }
                    _ => {}
                }
                let profile = match api.get_profile_self().await {
                    Ok(p) => Some(p),
                    Err(_) => None,
                };
                tx.send(BackToFrontMsg::LoginResponse(login_response, profile))?;
            } else {
                tx.send(BackToFrontMsg::LoginResponse(BlueskyLoginResponse::Info(crate::backend::main::BlueskyLoginResponseInfo::WasntLoggedIn), None))?;
            }
        } else {
            tx.send(BackToFrontMsg::LoginResponse(BlueskyLoginResponse::Info(crate::backend::main::BlueskyLoginResponseInfo::WasntLoggedIn), None))?;
            tx.send(BackToFrontMsg::KeyringFailure(format!("Failed to initialize keyring. {:?}", vault.err().unwrap())))?;
        }

        ctx.request_repaint();
        'outer: loop {
            let request = rx.try_recv();
            if request.is_err() {
                continue;
            }

            match request? {
                FrontToBackMsg::ShutdownMessage => break 'outer,
                FrontToBackMsg::LoginRequestStandard(handle, password) => {
                    let login_response = api.login(handle, password).await;
                    match &login_response {
                        BlueskyLoginResponse::Success(_, refresh_token) => {
                            if let Ok(vault) = keyring::Entry::new("com.headassbtw.metro.bluesky", "refreshJwt") {
                                if let Err(error) = vault.set_password(&refresh_token) {
                                    tx.send(BackToFrontMsg::KeyringFailure(format!("Error when caching login: {:?}", error)))?;
                                }
                            }
                        }
                        _ => {}
                    }
                    let profile = match api.get_profile_self().await {
                        Ok(p) => Some(p),
                        Err(_) => None,
                    };
                    tx.send(BackToFrontMsg::LoginResponse(login_response, profile))?;
                }
                FrontToBackMsg::LoginRequest2FA(_, _, _) => todo!(),
                FrontToBackMsg::GetTimelineRequest(cursor, limit) => {
                    tx.send(BackToFrontMsg::TimelineResponse(api.get_timeline(cursor, limit).await))?;
                }
                FrontToBackMsg::GetProfileRequest(did) => {
                    tx.send(BackToFrontMsg::ProfileResponse(did.clone(), api.get_profile(did).await))?;
                }
                FrontToBackMsg::GetThreadRequest(uri) => {
                    tx.send(BackToFrontMsg::ThreadResponse(uri.clone(), api.get_thread(uri, None, None).await))?;
                }
                FrontToBackMsg::CreateRecordRequest(record) => {
                    tx.send(BackToFrontMsg::RecordCreationResponse(api.create_record(record).await))?;
                }
                FrontToBackMsg::CreateRecordUnderPostRequest(record, post_mod) => match record {
                    BlueskyApiRecord::Post(_) => {
                        tx.send(BackToFrontMsg::RecordCreationResponse(Err(BlueskyApiError::ParseError("Tried to use a post-callback record creation call on a standalone post".to_owned()))))?;
                    }
                    BlueskyApiRecord::Like(record) => match api.create_record(BlueskyApiRecord::Like(record)).await {
                        Ok(res) => {
                            let mut post = post_mod.lock().unwrap();
                            if let Some(viewer) = &mut post.viewer {
                                viewer.like = Some(res.uri);
                            }
                        }
                        Err(err) => tx.send(BackToFrontMsg::RecordCreationResponse(Err(err)))?,
                    },
                    BlueskyApiRecord::Repost(record) => match api.create_record(BlueskyApiRecord::Repost(record)).await {
                        Ok(res) => {
                            let mut post = post_mod.lock().unwrap();
                            if let Some(viewer) = &mut post.viewer {
                                viewer.repost = Some(res.uri);
                            }
                        }
                        Err(err) => tx.send(BackToFrontMsg::RecordCreationResponse(Err(err)))?,
                    },
                },
                FrontToBackMsg::DeleteRecordRequest(_rkey, _nsid) => {
                    tx.send(BackToFrontMsg::RecordDeletionResponse(Err(BlueskyApiError::ParseError("Not Implemented".to_owned()))))?;
                }
                FrontToBackMsg::DeleteRecordUnderPostRequest(rkey, nsid, post_mod) => {
                    println!("deleting {}", rkey);
                    match api.delete_record(rkey, nsid.clone()).await {
                        Ok(_) => match nsid.as_str() {
                            "app.bsky.feed.like" => {
                                if let Some(viewer) = &mut post_mod.lock().unwrap().viewer {
                                    viewer.like = None;
                                }
                            }
                            "app.bsky.feed.repost" => {
                                if let Some(viewer) = &mut post_mod.lock().unwrap().viewer {
                                    viewer.repost = None;
                                }
                            }
                            _ => {
                                tx.send(BackToFrontMsg::RecordDeletionResponse(Err(BlueskyApiError::ParseError("Tried to delete a post under a post, not implemented".to_owned()))))?;
                            }
                        },
                        Err(err) => tx.send(BackToFrontMsg::RecordDeletionResponse(Err(err)))?,
                    }
                }
            }
            // if we processed anhything, we want the frontend to do it as well, this is the closest to doing that we can get.
            // i COULD probably do something with mutexes but that's janky and my dog is making it very annoying to write code.
            ctx.request_repaint();
        }
        Ok(())
    }
}
