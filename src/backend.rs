#[cfg(feature = "ssr")]
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod server_fns;

#[cfg(feature = "ssr")]
pub mod updates;

#[cfg(feature = "ssr")]
pub mod lol_static;

#[cfg(feature = "ssr")]
pub mod generate_sitemap;

#[cfg(feature = "ssr")]
fn format_duration_since(dt: NaiveDateTime) -> String {
    let match_end = dt.and_utc();
    let now = Utc::now();
    let duration = now.signed_duration_since(match_end);

    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        format!("{} seconds ago", duration.num_seconds())
    }
}

#[cfg(feature = "ssr")]
pub fn parse_date(date: Option<String>) -> Option<chrono::NaiveDateTime> {
    date.as_deref().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            chrono::NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), "%Y-%m-%d %H:%M:%S").ok()
        }
    })
}


#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct Id {
    id: i32,
}



