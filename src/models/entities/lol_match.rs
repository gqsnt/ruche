use crate::models::types::PlatformType;
use serde::{Deserialize, Serialize};
use crate::models::entities::lol_match_participant::LolMatchParticipant;

/// Represents a League of Legends match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LolMatch {
    pub id: i32,
    pub match_id: String,
    pub version_id: Option<i32>,
    pub mode_id: Option<i32>,
    pub map_id: Option<i32>,
    pub queue_id: Option<i32>,
    pub platform: Option<PlatformType>,
    pub updated: bool,
    pub match_creation: Option<String>,
    pub match_end: Option<String>,
    pub match_duration: Option<i32>,

}

