use crate::{backend::{
    main::BlueskyLoginResponse, record::{BlueskyApiCreateRecordResponse, BlueskyApiDeleteRecordResponse, BlueskyApiRecord}, thread::BlueskyApiGetThreadResponse, BlueskyApiError, ClientBackend
}, defs::{self, bsky::{actor::defs::Preference, embed::{self, AspectRatio}, feed::defs::GeneratorView}, Blob}, settings::Settings};
use crate::defs::bsky::{actor::defs::ProfileViewDetailed, feed::defs::{FeedCursorPair, PostView}};
use anyhow::Result;
use image::GenericImageView;
use std::{fs::File, io::Read, path::PathBuf, sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
}};
use crate::defs::bsky::actor::defs::ProfileView;
use crate::frontend::CursorListPair;

pub enum FrontToBackMsg {
    ShutdownMessage,
    LoginRequestStandard { handle: String, password: String },
    LoginRequest2FA(String, String, String),

    GetTimelineRequest { cursor: Option<String>, limit: Option<u32> },
    GetFeedRequest { feed: String, cursor: Option<String>, limit: Option<u32> },
    GetProfileRequest { did: String },
    GetThreadRequest { uri: String },
    GetAuthorFeedRequest { did: String, cursor: String, posts: Arc<Mutex<FeedCursorPair>> },
    GetFollowersRequest { did: String, profiles: Arc<Mutex<CursorListPair<ProfileView>>> },

    CreateRecordRequest(BlueskyApiRecord),
    CreateRecordWithMediaRequest(BlueskyApiRecord, Vec<PathBuf>),
    CreateRecordUnderPostRequest(BlueskyApiRecord, Arc<Mutex<PostView>>),

    DeleteRecordRequest { rkey: String, nsid: String },
    DeleteRecordUnderPostRequest { rkey: String, nsid: String, post_mod: Arc<Mutex<PostView>> },
}

pub enum BackToFrontMsg {
    BackendError(String),
    LoginResponse(BlueskyLoginResponse, Option<ProfileViewDetailed>, Vec<GeneratorView>),
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
                'gaming: {
                let login_response = api.login_refresh(token).await;
                let login_response = if let BlueskyLoginResponse::Success(inf) = login_response {
                    inf
                } else {
                    tx.send(BackToFrontMsg::LoginResponse(login_response, None, Vec::new()))?;
                    break 'gaming;
                };
                if let Ok(vault) = keyring::Entry::new("com.headassbtw.metro.bluesky", "refreshJwt") {
                    if let Err(error) = vault.set_password(&login_response.refresh_token) {
                        tx.send(BackToFrontMsg::KeyringFailure(format!("Error when caching login: {:?}", error)))?;
                    }
                }
                let profile = match api.get_profile_self().await {
                    Ok(p) => Some(p),
                    Err(_) => None,
                };
                //TODO: USE PREFS!
                let gen_views: Vec<GeneratorView> = match api.get_preferences().await {
                    Ok(ok) => {
                        let feeds:Vec<String> = {
                            let mut feeds = Vec::new();
                            for pref in ok {
                                if let Preference::SavedFeedsPrefV2(val) = pref {
                                    for feed in val.items {
                                        if feed.value.starts_with("at") && feed.pinned {
                                            feeds.push(feed.value);    
                                        }
                                    }
                                    break;
                                }
                            }
                            feeds
                        };

                        match api.get_feed_generators(feeds).await {
                            Ok(ok) => ok,
                            Err(err) => {
                                tx.send(BackToFrontMsg::BackendError(format!("Feed generator pull failed.\n{:?}", err)))?;
                                Vec::new()
                            },
                        }
                    },
                    Err(err) => {
                        tx.send(BackToFrontMsg::BackendError(format!("Preferences pull failed.\n{:?}", err)))?;
                        Vec::new()
                    },
                };

                
                
                tx.send(BackToFrontMsg::LoginResponse(BlueskyLoginResponse::Success(login_response), profile, gen_views))?;
                }
            } else {
                tx.send(BackToFrontMsg::LoginResponse(BlueskyLoginResponse::Info(crate::backend::main::BlueskyLoginResponseInfo::WasntLoggedIn), None, Vec::new()))?;
            }
        } else {
            tx.send(BackToFrontMsg::LoginResponse(BlueskyLoginResponse::Info(crate::backend::main::BlueskyLoginResponseInfo::WasntLoggedIn), None, Vec::new()))?;
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
                FrontToBackMsg::LoginRequestStandard { handle, password } => {
                    let login_response = api.login(handle, password).await;
                    let login_response = if let BlueskyLoginResponse::Success(inf) = login_response {
                        inf
                    } else {
                        tx.send(BackToFrontMsg::LoginResponse(login_response, None, Vec::new()))?;
                        continue;
                    };
                    if let Ok(vault) = keyring::Entry::new("com.headassbtw.metro.bluesky", "refreshJwt") {
                        if let Err(error) = vault.set_password(&login_response.refresh_token) {
                            tx.send(BackToFrontMsg::KeyringFailure(format!("Error when caching login: {:?}", error)))?;
                        }
                    }
                    let profile = match api.get_profile_self().await {
                        Ok(p) => Some(p),
                        Err(_) => None,
                    };

                    tx.send(BackToFrontMsg::LoginResponse(BlueskyLoginResponse::Success(login_response), profile, Vec::new()))?;
                }
                FrontToBackMsg::LoginRequest2FA(_, _, _) => todo!(),
                FrontToBackMsg::GetTimelineRequest { cursor, limit } => {
                    tx.send(BackToFrontMsg::TimelineResponse(api.get_timeline(cursor, limit).await))?;
                }
                FrontToBackMsg::GetFeedRequest { feed, cursor, .. } => {
                    tx.send(BackToFrontMsg::TimelineResponse(api.get_feed(feed, cursor).await))?;
                }
                FrontToBackMsg::GetProfileRequest { did } => {
                    tx.send(BackToFrontMsg::ProfileResponse(did.clone(), api.get_profile(did).await))?;
                }
                FrontToBackMsg::GetThreadRequest { uri } => {
                    tx.send(BackToFrontMsg::ThreadResponse(uri.clone(), api.get_thread(uri, None, None).await))?;
                }
                FrontToBackMsg::GetAuthorFeedRequest { did, cursor, posts } => {
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
                }
                FrontToBackMsg::GetFollowersRequest { did, profiles } => {
                    let cursor = {
                        let mut profiles = profiles.lock().unwrap();
                        let cursor = profiles.cursor.clone();
                        profiles.cursor = None;
                        cursor.unwrap_or(String::new())
                    };

                    let (new_cursor, mut followers) = api.get_followers(did, cursor).await.unwrap();

                    {
                        let mut profiles = profiles.lock().unwrap();
                        profiles.cursor = Some(new_cursor);
                        profiles.items.append(&mut followers);
                    }
                }

                FrontToBackMsg::CreateRecordRequest(record) => {
                    tx.send(BackToFrontMsg::RecordCreationResponse(api.create_record(record).await))?;
                }
                // TODO: FIX THIS LATER lmao, it doesn't support alt text n whatnot
                FrontToBackMsg::CreateRecordWithMediaRequest(record, images) => 'give_up: {
                    let mut blobs: Vec<(Blob, Option<AspectRatio>)> = Vec::new();
                    for image in images {
                        let mut file = File::open(&image).expect("no file found");
                        let metadata = std::fs::metadata(&image).expect("unable to read metadata");
                        let mut buffer = vec![0; metadata.len() as usize];
                        file.read(&mut buffer).expect("buffer overflow");

                        //THIS FUCKING SUCKS!
                        let img = image::load_from_memory(&buffer);
                        let ratio = match img {
                            Ok(img) => {
                                Some(AspectRatio {
                                    width: img.dimensions().0.clone(),
                                    height: img.dimensions().1.clone(),
                                })
                            },
                            Err(err) => {
                                println!("failed to open image: {}", err);
                                None
                            }
                        };

                        match api.upload_blob(buffer).await {
                            Ok(res) => blobs.push((res, ratio)),
                            Err(err) => {
                                tx.send(BackToFrontMsg::RecordCreationResponse(Err(err)))?;
                                break 'give_up;
                            }
                        }
                    }
                    let mut record = record;
                    match record {
                        BlueskyApiRecord::Post(ref mut post) => {
                            post.embed = Some(embed::Variant::ImagesRaw { images: {
                                let mut vec = Vec::new();
                                for (blob, ratio) in blobs {
                                    vec.push(defs::bsky::embed::images::Image {
                                        image: blob,
                                        alt: String::new(),
                                        aspect_ratio: ratio,
                                    });
                                }
                                vec
                            } }.into())
                        },
                        _ => todo!(),
                    }
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
                FrontToBackMsg::DeleteRecordRequest { .. } => {
                    tx.send(BackToFrontMsg::RecordDeletionResponse(Err(BlueskyApiError::NotImplemented)))?;
                }
                FrontToBackMsg::DeleteRecordUnderPostRequest{ rkey, nsid, post_mod } => {
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
