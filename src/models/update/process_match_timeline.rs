use crate::consts::PlatformRoute;
use crate::error_template::{AppError, AppResult};
use crate::models::entities::lol_match_timeline::{ItemEvent, LolMatchTimeline};
use crate::models::entities::summoner::Summoner;
use riven::RiotApi;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

pub struct TempLolMatchTimeline {
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub items_event_timeline: Vec<(i64, ItemEvent)>,
    pub skills_timeline: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    SkillLevelUp,
    ItemPurchased,
    ItemSold,
    ItemDestroyed,
    ItemUndo,
    Other(String),
}

impl From<&str> for EventType {
    fn from(s: &str) -> Self {
        match s {
            "SKILL_LEVEL_UP" => EventType::SkillLevelUp,
            "ITEM_PURCHASED" => EventType::ItemPurchased,
            "ITEM_SOLD" => EventType::ItemSold,
            "ITEM_DESTROYED" => EventType::ItemDestroyed,
            "ITEM_UNDO" => EventType::ItemUndo,
            other => EventType::Other(other.to_string()),
        }
    }
}


pub fn push_item_event_into_participant_id(participants: &mut HashMap<i32, TempLolMatchTimeline>, participant_id: i32, timestamp: i64, event: ItemEvent) {
    let participant = participants.get_mut(&participant_id).unwrap();
    participant.items_event_timeline.push((timestamp, event));
}


pub async fn process_match_timeline(
    db: &PgPool,
    api: Arc<RiotApi>,
    match_id: i32,
    riot_match_id: String,
    platform: String,
) -> AppResult<()> {
    // Fetch the match timeline
    let platform_route = PlatformRoute::from_region_str(platform.as_str()).unwrap();
    let riven_pr = riven::consts::PlatformRoute::from_str(platform_route.to_string().as_str()).unwrap();
    let timeline = api
        .match_v5()
        .get_timeline(riven_pr.to_regional(), &riot_match_id)
        .await?
        .ok_or_else(|| AppError::CustomError("Timeline not found".into()))?;

    let db_platform = PlatformRoute::from_region_str(riven_pr.as_region_str())
        .ok_or_else(|| AppError::CustomError(riven_pr.as_region_str().to_string()))?;

    let puuids_summoner_ids = Summoner::find_summoner_ids_by_puuids(
        db,
        db_platform,
        &timeline.metadata.participants,
    )
        .await?;

    let mut lol_match_timelines = HashMap::new();

    for participant in &timeline.info.participants.unwrap_or_default() {
        let summoner_id = puuids_summoner_ids
            .get(participant.puuid.as_str())
            .ok_or_else(|| {
                AppError::CustomError(format!("Summoner ID not found for PUUID {}", participant.puuid))
            })?;
        lol_match_timelines.insert(
            participant.participant_id,
            TempLolMatchTimeline {
                lol_match_id: match_id,
                summoner_id: *summoner_id,
                items_event_timeline: Vec::new(),
                skills_timeline: Vec::new(),
            },
        );
    }

    for (_, frame) in timeline.info.frames.iter().enumerate() {
        for event in &frame.events {
            let event_type = EventType::from(event.r#type.as_str());
            match event_type {
                EventType::SkillLevelUp => {
                    let skill_slot = event.skill_slot.ok_or_else(|| {
                        AppError::CustomError("Missing skill_slot in SKILL_LEVEL_UP event".into())
                    })?;
                    let participant_id = event.participant_id.ok_or_else(|| {
                        AppError::CustomError("Missing participant_id in event".into())
                    })?;
                    let participant = lol_match_timelines.get_mut(&participant_id).ok_or_else(|| {
                        AppError::CustomError(format!("Participant with ID {} not found", participant_id))
                    })?;
                    participant.skills_timeline.push(skill_slot);
                }
                EventType::ItemPurchased => {
                    let item_id = event.item_id.ok_or_else(|| {
                        AppError::CustomError("Missing item_id in ITEM_PURCHASED event".into())
                    })?;
                    push_item_event_into_participant_id(&mut lol_match_timelines, event.participant_id.unwrap(), event.timestamp, ItemEvent::Purchased { item_id });
                }
                EventType::ItemSold => {
                    let item_id = event.item_id.ok_or_else(|| {
                        AppError::CustomError("Missing item_id in ITEM_SOLD event".into())
                    })?;
                    push_item_event_into_participant_id(&mut lol_match_timelines, event.participant_id.unwrap(), event.timestamp, ItemEvent::Sold { item_id });
                }
                EventType::ItemDestroyed => {
                    let item_id = event.item_id.ok_or_else(|| {
                        AppError::CustomError("Missing item_id in ITEM_DESTROYED event".into())
                    })?;
                    push_item_event_into_participant_id(&mut lol_match_timelines, event.participant_id.unwrap(), event.timestamp, ItemEvent::Destroyed { item_id });
                }
                EventType::ItemUndo => {
                    let participant_id = event.participant_id.ok_or_else(|| {
                        AppError::CustomError("Missing participant_id in event".into())
                    })?;
                    let participant = lol_match_timelines.get_mut(&participant_id).ok_or_else(|| {
                        AppError::CustomError(format!("Participant with ID {} not found", participant_id))
                    })?;
                    if let Some(before_id) = event.before_id {
                        if before_id != 0 {
                            if let Some(pos) = participant
                                .items_event_timeline
                                .iter()
                                .rev()
                                .position(|item| {
                                    matches!(
                                        item.1,
                                        ItemEvent::Purchased { item_id}
                                        if item_id == before_id
                                    )
                                })
                            {
                                participant.items_event_timeline.remove(participant.items_event_timeline.len() - pos - 1);
                            }
                        }
                    }
                    if let Some(after_id) = event.after_id {
                        if after_id != 0 {
                            if let Some(pos) = participant
                                .items_event_timeline
                                .iter()
                                .rev()
                                .position(|item| {
                                    matches!(
                                        item.1,
                                        ItemEvent::Sold { item_id }
                                        if item_id == after_id
                                    )
                                })
                            {
                                participant.items_event_timeline.remove(participant.items_event_timeline.len() - pos - 1);
                            }
                        }
                    }
                }
                EventType::Other(_) => {}
            }
        }
    }

    let timelines = lol_match_timelines
        .into_iter()
        .map(|(_, mut v)| {
            v.items_event_timeline.sort_by_key(|(timestamp, _)| *timestamp);
            // remove if empty the vec in the hashmap is empty
            v.items_event_timeline.retain(|(_, item)| matches!(item, ItemEvent::Purchased{..} | ItemEvent::Sold{..} | ItemEvent::Destroyed{..}));
            v
        })
        .collect::<Vec<_>>();

    LolMatchTimeline::bulk_insert(db, timelines).await?;

    Ok(())
}


