use chrono::{DateTime, Datelike, Timelike, Utc};

pub mod post;
pub mod feed_post;
pub mod profile;
pub mod embeds;
pub mod profile_picture;

fn offset_time(time: DateTime<Utc>) -> String {
    puffin::profile_function!();
    let offset = Utc::now() - time;
    if offset.num_days() >= 7 {
        //TODO: OS formatter
        format!("{:02}:{:02} {}/{}/{}", time.hour(), time.minute(), time.month(), time.day(), time.year())
    } else if offset.num_hours() >= 24 {
        format!("{}d", offset.num_days())
    } else if offset.num_minutes() >= 60 {
        format!("{}h", offset.num_hours())
    } else if offset.num_seconds() >= 60 {
        format!("{}m", offset.num_minutes())
    } else {
        format!("{}s", offset.num_seconds())
    }
}