use std::{collections::HashMap, fs::{self, File}, io::Write, path::PathBuf, sync::{mpsc::{Receiver, Sender}, Arc, Mutex}, hash::{DefaultHasher, Hash, Hasher}};
use image::{ImageReader, DynamicImage};
use directories::ProjectDirs;
use egui::{ColorImage, TextureHandle, TextureId, TextureOptions, Vec2};


pub enum LoadableImage {
    /// Completely unplanned, doesn't exist
    Unloaded,
    /// In progress
    Loading,
    /// Here ya go
    Loaded(TextureId, Vec2)
}

enum LoaderRequest {
    Shutdown,
    GetImg(String)
}

pub struct ImageCache {
    db: Arc<Mutex<HashMap<String, Option<TextureHandle>>>>,
    tx: Sender<LoaderRequest>,
}

impl ImageCache {
    pub fn new(ctx: egui::Context) -> Self {
        let (tx0, rx1) = std::sync::mpsc::channel();

        let proj_dirs = ProjectDirs::from("dev", "headassbtw",  "com.headassbtw.metro.bluesky");
        if proj_dirs.is_none() { println!("Could not create image cache folder"); return Self { db: Arc::new(Mutex::new(HashMap::new())), tx: tx0}; }
        let proj_dirs = proj_dirs.unwrap();
        
        let dir = proj_dirs.data_local_dir().join("image_cache");
        if !dir.exists() { fs::create_dir_all(&dir).unwrap(); }
        
        let cache = dir.clone();
        
        let map: Arc<Mutex<HashMap<String, Option<TextureHandle>>>> = Arc::new(Mutex::new(HashMap::new()));
        let map0 = map.clone();
        
        tokio::task::spawn(async move {
            let _result = ImageCache::run(rx1, map0, cache, ctx).await;
        });

        Self { db: map, tx: tx0}
    }
    
    async fn run(
        rx1: Receiver<LoaderRequest>,
        map: Arc<Mutex<HashMap<String, Option<TextureHandle>>>>,
        cache: PathBuf,
        ctx: egui::Context
    ) -> anyhow::Result<()>  {
        println!("image cache dir: {}", cache.to_str().unwrap());
        let client = reqwest::Client::builder().user_agent("some fuckass rust app that looks like windows 8").build();
        if let Err(err) = client { return Err(anyhow::Error::msg(format!("{:?}", err))); }
        let client = client.unwrap();
        
        'outer: loop {
            while let Ok(req) = rx1.try_recv() {
                let req = match req {
                    LoaderRequest::Shutdown => break 'outer Ok(()),
                    LoaderRequest::GetImg(resdb_path) => resdb_path,
                };
                
                if req.is_empty() { println!("tried to load an empty URL from image cache"); continue; }
                let split_idx = if let Some(pos) = req.find("://") { pos } else { println!("no beginner: {}", req); continue; };
                let (_, path) = req.split_at(split_idx+3);

                let extension = if let Some(ex) = path.find("@") {
                    path.split_at(ex+1).1
                } else if let Some(ex) = path.split(".").last() {
                    ex
                } else {
                    "webp" //optimism
                };

                let web_path = req.clone();
                let mut hasher = DefaultHasher::new();
                web_path.hash(&mut hasher);
                let mut file_path = cache.clone();
                file_path.push(format!("{}.{}", hasher.finish().to_string(), extension));

                if !file_path.exists() {
                    let dl = client.get(&web_path).send().await;

                    if let Err(err) = dl {
                        println!("Failed to download {}! Reason: {:?}", &web_path, &err);
                        continue;
                    } let dl = dl.unwrap();

                    let body = dl.bytes().await?;
                    let file = File::create(&file_path);
                    if file.is_err() {
                        println!("Failed to create {:?}! Reason: {:?}", &file_path, &file);
                        continue;
                    } let mut file = file.unwrap();

                    if let Err(err) = file.write_all(&body) {
                        println!("Failed to copy file! Reason: {:?}", err);
                        continue;
                    }
                }

                let file_read = Self::load_from_fs(ctx.clone(), &file_path);

                if let Ok(fil) = file_read {
                    let mut map = map.lock().unwrap();
                    map.insert(req.clone(), Some(fil));
                    ctx.request_repaint(); // there's probably a user waiting on this, it won't update until it needs to, or this requests it
                } else if let Err(err) = file_read {
                    println!("Failed to read image! {:?}", err);
                }
            }
        }
    }

    pub fn load_from_fs(ctx: egui::Context, path: &PathBuf) -> anyhow::Result<TextureHandle> {

        let identifier = path.file_name().unwrap().to_str().unwrap();

        //println!("Loading image {:?}", path);
        let img = ImageReader::open(path);
        if img.is_err() {
            return Err(anyhow::Error::msg(format!("Failed to open \"{}\"!", path.to_string_lossy())));
        }

        let img_decoded = img?.decode();
        if let Err(err) = img_decoded {
            return Err(anyhow::Error::msg(format!("Failed to decode image: {:?}", err)));
        }

        let img_decoded = img_decoded?;

        match img_decoded.color().channel_count() {
            2 => {
                let img_a = DynamicImage::ImageRgba8(img_decoded.into_rgba8());
                let ci = ColorImage::from_rgba_unmultiplied(
                    [img_a.width() as usize, img_a.height() as usize],
                    img_a.as_bytes(),
                );

                return Ok(ctx.load_texture(identifier, ci, TextureOptions::NEAREST));
                
            }
            3 => {
                let ci = ColorImage::from_rgb(
                    [img_decoded.width() as usize, img_decoded.height() as usize],
                    img_decoded.as_bytes(),
                );
                
                return Ok(ctx.load_texture(identifier, ci, TextureOptions::NEAREST));
            }
            4 => {
                let ci = ColorImage::from_rgba_unmultiplied(
                    [img_decoded.width() as usize, img_decoded.height() as usize],
                    img_decoded.as_bytes(),
                );

                return Ok(ctx.load_texture(identifier, ci, TextureOptions::NEAREST));
            }
            _ => return Err(anyhow::Error::msg("unsupported amount of channels")),
        }
    }

    /// tells the thread to \*lightning\*
    pub fn shutdown(&mut self) {
        self.tx.send(LoaderRequest::Shutdown).unwrap();
    }

    /// Accepts a URL and gets an egui-drawable image (or lack thereof) from it
    pub fn get_image(&self, id: &String) -> LoadableImage {
        let mut db = self.db.lock().unwrap();
        if let Some(img) = db.get(id) {
            if let Some(id) = img {
                return LoadableImage::Loaded(id.id(), id.size_vec2());
            } else {
                return LoadableImage::Loading;
            }
        } else {
            // i don't want another enum so i just use existing as unloaded, and the option as loading/loaded
            // race conditions arise from setting None on the thread (loader thread is busy loading, and doesn't set None itself)
            // so we do it here
            db.insert(id.clone(), None);
            if let Err(err) = self.tx.send(LoaderRequest::GetImg(id.to_string())) {
                println!("Send error! {:?}", err);
            }
        }
        

        LoadableImage::Unloaded
    }
}