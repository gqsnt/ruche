use chrono::NaiveDateTime;

pub mod lol_match;
pub mod lol_match_participant;
pub mod summoner;
mod lol_match_timeline;
pub mod db_model;

#[derive(sqlx::FromRow)]
struct Id {
    id: i32,
}

pub const DATE_FORMAT: &str = "%d/%m/%Y %H:%M";


pub fn parse_date(date: Option<String>) -> Option<chrono::NaiveDateTime> {
    date.as_deref().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), "%Y-%m-%d %H:%M:%S").ok()
        }
    })
}

pub fn round_to_2_decimal_places(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}