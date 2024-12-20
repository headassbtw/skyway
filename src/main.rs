#![feature(let_chains)]

pub mod backend;
pub mod bridge;
pub mod frontend;
pub mod image;
pub mod widgets;
pub mod defs;
pub mod settings;

use crate::frontend::main::ClientFrontend;

const BSKY_BLUE: egui::Color32 = egui::Color32::from_rgb(32, 139, 254);

fn open_in_browser(url: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd.exe").arg("/C").arg("start").arg(url).spawn();
}

#[tokio::main]
async fn main() -> eframe::Result {
    if cfg!(debug_assertions) {
        puffin::set_scopes_on(true);
        match puffin_http::Server::new("127.0.0.1:8585") {
            Ok(puffin_server) => {
                println!("Profiling enabled on port 8585");
                /*
                std::process::Command::new("puffin_viewer")
                    .arg("--url")
                    .arg("127.0.0.1:8585")
                    .spawn()
                    .ok();
                */
                #[allow(clippy::mem_forget)]
                std::mem::forget(puffin_server);
            }
            Err(err) => {
                println!("Failed to start puffin server: {err}");
            }
        };
    }

    let native_options = eframe::NativeOptions { viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]).with_app_id("com.headassbtw.metro.bluesky").with_min_inner_size([640.0, 480.0]), ..Default::default() };
    eframe::run_native("BLUESKY!", native_options, Box::new(|cc| Ok(Box::new(ClientFrontend::new(cc)))))
}
