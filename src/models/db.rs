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

pub fn round_to_2_decimal_places(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}