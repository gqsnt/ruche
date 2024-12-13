use chrono::{Duration, Local, Timelike};
use tokio::time::Instant;

pub mod generate_sitemap;
pub mod handle_live_game_cache;
pub mod sse_broadcast_match_updated_cleanup;
pub mod update_matches;
pub mod update_pro_players;

pub fn calculate_next_run_to_fixed_start_hour(start_hour: u32) -> Instant {
    let now = Local::now();
    let target_time = if now.hour() >= start_hour {
        (now + Duration::days(1))
            .with_hour(start_hour)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
    } else {
        now.with_hour(start_hour)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
    };
    Instant::now() + (target_time - now).to_std().unwrap()
}
