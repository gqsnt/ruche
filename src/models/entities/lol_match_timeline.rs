use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LolMatchTimeline {
    pub id: i32,
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub items_event_timeline: Vec<(i32, Vec<ItemEvent>)>,
    pub skills_timeline: Vec<i32>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemEvent {
    Purchased {
        item_id: i32,
    },
    Sold {
        item_id: i32,
    },
    Destroyed {
        item_id: i32,
    },
}

impl ItemEvent {
    pub fn get_id(&self) -> i32 {
        match self {
            ItemEvent::Purchased { item_id } => *item_id,
            ItemEvent::Sold { item_id } => *item_id,
            ItemEvent::Destroyed { item_id } => *item_id,
        }
    }
}

