use crate::{backend::{
    main::BlueskyLoginResponse,
    record::{BlueskyApiCreateRecordResponse, BlueskyApiDeleteRecordResponse, BlueskyApiRecord},
    thread::BlueskyApiGetThreadResponse,
    BlueskyApiError, ClientBackend,
}, defs::bsky::feed::defs::FeedViewPost, settings::Settings};
use crate::defs::bsky::{actor::defs::ProfileViewDetailed, feed::defs::{FeedCursorPair, PostView}};
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
    GetAuthorFeedRequest(String, String, Arc<Mutex<FeedCursorPair>>),

    CreateRecordRequest(BlueskyApiRecord),
    CreateRecordUnderPostRequest(BlueskyApiRecord, Arc<Mutex<PostView>>),

    DeleteRecordRequest(String, String),
    DeleteRecordUnderPostRequest(String, String, Arc<Mutex<PostView>>),
}

pub enum BackToFrontMsg {
    LoginResponse(BlueskyLoginResponse, Option<ProfileViewDetailed>),
    TimelineResponse(Result<FeedCursorPair, BlueskyApiError>),
    KeyringFailure(String),
    RecordCreationResponse(Result<BlueskyApiCreateRecordResponse, BlueskyApiError>),
    RecordDeletionResponse(Result<BlueskyApiDeleteRecordResponse, BlueskyApiError>),
    ProfileResponse(String, Result<ProfileViewDetailed, BlueskyApiError>),
    ThreadResponse(String, Result<BlueskyApiGetThreadResponse, BlueskyApiError>),
}

pub struct Bridge {
    pub frontend_listener: Receiver<BackToFrontMsg>,
    pub backend_commander: Sender<FrontToBackMsg>,
    pub working_indicator: Arc<tokio::sync::Mutex<bool>>,
}

impl Bridge {
    pub fn new(ctx: egui::Context, settings: Arc<Mutex<Settings>>) -> Self {
        let (backend_commander, backend_listener) = std::sync::mpsc::channel();
        let (frontend_commander, frontend_listener) = std::sync::mpsc::channel();
        let ctx_burn = ctx.clone();
        let working_indicator = Arc::new(tokio::sync::Mutex::new(false));
        let indicator_burn = working_indicator.clone();
        tokio::task::spawn(async move {
            //let die_fallback_transmittter = backend_responder.clone();
            //panic::set_hook(Box::new( |_| {}));
            let result = Self::run(backend_listener, frontend_commander, ctx_burn, settings, indicator_burn).await;
            if let Err(result) = result {
                panic!("Bridge failed! {}", result);
            }
        });

        Self { frontend_listener, backend_commander, working_indicator }
    }

    async fn run(rx: Receiver<FrontToBackMsg>, tx: Sender<BackToFrontMsg>, ctx: egui::Context, _settings: Arc<Mutex<Settings>>, working_indicator: Arc<tokio::sync::Mutex<bool>>) -> Result<()> {
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
            let mut working = working_indicator.lock().await;
            *working = true;
            ctx.request_repaint();

            match request? {
                FrontToBackMsg::ShutdownMessage => {drop(working); break 'outer},
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
                },
                FrontToBackMsg::GetAuthorFeedRequest(did, cursor, posts) => {
                    let res = api.get_author_feed(did, cursor).await;
                    if let Err(err) = &res {
                        println!("Failure {:?}", err);
                    }

                    let mut res = res.unwrap();

                    {
                        let mut poasts = posts.lock().unwrap();
                        poasts.cursor = res.cursor;
                        poasts.feed.append(&mut res.feed);
                    }
                },
                FrontToBackMsg::CreateRecordRequest(record) => {
                    tx.send(BackToFrontMsg::RecordCreationResponse(api.create_record(record).await))?;
                }
                FrontToBackMsg::CreateRecordUnderPostRequest(record, post_mod) => match record {
                    BlueskyApiRecord::Post(_) => {
                        tx.send(BackToFrontMsg::RecordCreationResponse(Err(BlueskyApiError::NotImplemented)))?;
                    }
                    BlueskyApiRecord::Like(record) => match api.create_record(BlueskyApiRecord::Like(record)).await {
                        Ok(res) => {
                            let mut post = post_mod.lock().unwrap();
                            if let Some(viewer) = &mut post.viewer {
                                viewer.like = Some(res.uri);
                            }
                            if let Some(count) = &mut post.like_count {
                                *count += 1;
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
                            if let Some(count) = &mut post.repost_count {
                                *count += 1;
                            }
                        }
                        Err(err) => tx.send(BackToFrontMsg::RecordCreationResponse(Err(err)))?,
                    },
                },
                FrontToBackMsg::DeleteRecordRequest(_rkey, _nsid) => {
                    tx.send(BackToFrontMsg::RecordDeletionResponse(Err(BlueskyApiError::NotImplemented)))?;
                }
                FrontToBackMsg::DeleteRecordUnderPostRequest(rkey, nsid, post_mod) => {
                    println!("deleting {}", rkey);
                    match api.delete_record(rkey, nsid.clone()).await {
                        Ok(_) => match nsid.as_str() {
                            "app.bsky.feed.like" => {
                                let mut post = post_mod.lock().unwrap();
                                if let Some(viewer) = &mut post.viewer {
                                    viewer.like = None;
                                }
                                if let Some(count) = &mut post.like_count {
                                    *count -= 1;
                                }
                            }
                            "app.bsky.feed.repost" => {
                                let mut post = post_mod.lock().unwrap();
                                if let Some(viewer) = &mut post.viewer {
                                    viewer.repost = None;
                                }
                                if let Some(count) = &mut post.repost_count {
                                    *count -= 1;
                                }
                            }
                            _ => {
                                tx.send(BackToFrontMsg::RecordDeletionResponse(Err(BlueskyApiError::NotImplemented)))?;
                            }
                        },
                        Err(err) => tx.send(BackToFrontMsg::RecordDeletionResponse(Err(err)))?,
                    }
                }
            }
            // if we processed anhything, we want the frontend to do it as well, this is the closest to doing that we can get.
            // i COULD probably do something with mutexes but that's janky and my dog is making it very annoying to write code.
            ctx.request_repaint();
            *working = false;
            drop(working);
        }
        Ok(())
    }
}
