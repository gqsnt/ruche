use crate::backend::ssr::{AppError, AppResult};
use crate::ssr::RiotApiState;
use crate::views::components::match_details::{ItemEvent, ItemEventType, Skill};
use chrono::NaiveDateTime;
use common::consts::platform_route::PlatformRoute;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::QueryBuilder;
use std::collections::HashMap;

pub async fn update_match_timeline(
    db: &PgPool,
    api: RiotApiState,
    match_id: i32,
    riot_match_id: &str,
    platform_route: PlatformRoute,
) -> AppResult<()> {
    // Fetch the match timeline
    let riven_pr = platform_route.to_riven();
    let timeline = api
        .match_v5()
        .get_timeline(riven_pr.to_regional(), riot_match_id)
        .await?
        .ok_or_else(|| AppError::CustomError("Timeline not found".into()))?;

    let puuids_summoner_ids =
        find_summoner_ids_by_puuids(db, &timeline.metadata.participants).await?;

    let mut lol_match_timelines = HashMap::new();

    for participant in &timeline.info.participants.unwrap_or_default() {
        let Some(summoner_id) = puuids_summoner_ids.get(participant.puuid.as_str()) else {
            continue;
        };
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

    for frame in timeline.info.frames.iter() {
        for event in &frame.events {
            let Some(participant_id) = event.participant_id.filter(|&id| id > 0) else {
                continue;
            };
            let event_type = EventType::from(event.r#type.as_str());
            match event_type {
                EventType::SkillLevelUp => {
                    let skill_slot = event.skill_slot.ok_or_else(|| {
                        AppError::CustomError("Missing skill_slot in SKILL_LEVEL_UP event".into())
                    })?;
                    let Some(participant) = lol_match_timelines.get_mut(&participant_id) else {
                        continue;
                    };
                    participant
                        .skills_timeline
                        .push(Skill::from(skill_slot as u8));
                }
                EventType::ItemPurchased => {
                    let item_id = event.item_id.ok_or_else(|| {
                        AppError::CustomError("Missing item_id in ITEM_PURCHASED event".into())
                    })? as u32;
                    push_item_event_into_participant_id(
                        &mut lol_match_timelines,
                        participant_id,
                        event.timestamp,
                        ItemEvent {
                            item_id,
                            event_type: ItemEventType::Purchased,
                        },
                    );
                }
                EventType::ItemSold => {
                    let item_id = event.item_id.ok_or_else(|| {
                        AppError::CustomError("Missing item_id in ITEM_SOLD event".into())
                    })? as u32;
                    push_item_event_into_participant_id(
                        &mut lol_match_timelines,
                        participant_id,
                        event.timestamp,
                        ItemEvent {
                            item_id,
                            event_type: ItemEventType::Sold,
                        },
                    );
                }
                EventType::ItemUndo => {
                    let Some(participant) = lol_match_timelines.get_mut(&participant_id) else {
                        continue;
                    };
                    if let Some(before_id) = event.before_id {
                        let before_id = before_id as u32;
                        if before_id != 0 {
                            if let Some(pos) = participant
                                .items_event_timeline
                                .iter()
                                .rev()
                                .position(|item| {
                                    matches!(
                                        item.1.event_type,
                                        ItemEventType::Purchased
                                        if item.1.item_id == before_id
                                    )
                                })
                            {
                                participant
                                    .items_event_timeline
                                    .remove(participant.items_event_timeline.len() - pos - 1);
                            }
                        }
                    }
                    if let Some(after_id) = event.after_id {
                        let after_id = after_id as u32;
                        if after_id != 0 {
                            if let Some(pos) = participant
                                .items_event_timeline
                                .iter()
                                .rev()
                                .position(|item| {
                                    matches!(
                                        item.1.event_type,
                                        ItemEventType::Sold
                                        if item.1.item_id == after_id
                                    )
                                })
                            {
                                participant
                                    .items_event_timeline
                                    .remove(participant.items_event_timeline.len() - pos - 1);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let timelines = lol_match_timelines
        .into_values()
        .map(|mut v| {
            v.items_event_timeline
                .sort_by_key(|(timestamp, _)| *timestamp);
            v
        })
        .collect::<Vec<_>>();

    bulk_insert_match_timeline(db, timelines).await?;

    Ok(())
}

pub fn push_item_event_into_participant_id(
    participants: &mut HashMap<i32, TempLolMatchTimeline>,
    participant_id: i32,
    timestamp: i64,
    event: ItemEvent,
) {
    let Some(participant) = participants.get_mut(&participant_id) else {
        return;
    };
    participant
        .items_event_timeline
        .push(((timestamp / 60000) as u16, event));
}

pub struct TempLolMatchTimeline {
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub items_event_timeline: Vec<(u16, ItemEvent)>,
    pub skills_timeline: Vec<Skill>,
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

pub async fn find_summoner_ids_by_puuids(
    db: &PgPool,
    puuids: &[String],
) -> AppResult<HashMap<String, i32>> {
    Ok(sqlx::query_as::<_, SummonerTimeLineInfo>(
        "SELECT id, puuid FROM summoners WHERE puuid = ANY($1)",
    )
    .bind(puuids)
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|x| (x.puuid, x.id))
    .collect::<HashMap<String, i32>>())
}

async fn bulk_insert_match_timeline(
    db: &PgPool,
    timelines: Vec<TempLolMatchTimeline>,
) -> AppResult<()> {
    // Check if timelines vector is empty
    if timelines.is_empty() {
        return Ok(());
    }

    // Prepare the insert SQL with placeholders
    let mut qb = QueryBuilder::new(
        "INSERT INTO lol_match_timelines (lol_match_id, summoner_id, items_event_timeline, skills_timeline) ",
    );

    qb.push_values(timelines.into_iter(), |mut b, rec| {
        let mut items_event_timeline = HashMap::new();
        for (timestamp, event) in rec.items_event_timeline {
            items_event_timeline
                .entry(timestamp)
                .or_insert_with(Vec::new)
                .push(event);
        }
        // convert to vec
        let mut items_event_timeline: Vec<_> = items_event_timeline.into_iter().collect();
        items_event_timeline.sort_by_key(|x| x.0);
        b.push_bind(rec.lol_match_id);
        b.push_bind(rec.summoner_id);
        b.push_bind(serde_json::to_value(&items_event_timeline).unwrap_or_default());
        b.push_bind(
            rec.skills_timeline
                .iter()
                .map(|&x| (x as u8) as i32)
                .collect::<Vec<_>>(),
        );
    });
    qb.build().fetch_all(db).await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct SummonerTimeLineInfo {
    pub id: i32,
    pub puuid: String,
    #[allow(dead_code)]
    #[sqlx(default)]
    pub updated_at: Option<NaiveDateTime>,
}
