use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum PreferredImageFormat {
	Original,
	Png,
	Jpeg
}

#[derive(Serialize, Deserialize)]
pub enum Theme {
	System,
	Dark,
	Light
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
	pub preferred_image_format: PreferredImageFormat,
	pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
        	preferred_image_format: PreferredImageFormat::Original,
        	theme: Theme::System,
        }
    }
}