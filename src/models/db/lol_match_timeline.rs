use std::collections::HashMap;
use serde_json::json;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};
use sqlx::types::{Json, JsonValue};
use unzip_n::unzip_n;
use crate::error_template::AppResult;
use crate::models::entities::lol_match_timeline::{ItemEvent, LolMatchTimeline};
use crate::models::update::process_match_timeline::TempLolMatchTimeline;

unzip_n!(4);


#[derive(FromRow)]
pub struct LolMatchTimelineQueryResult{
    pub id: i32,
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub items_event_timeline: JsonValue,
    pub skills_timeline: Vec<i32>,
}

impl LolMatchTimeline{
    pub async fn get_match_timeline(db:&PgPool, match_id:i32)->AppResult<Vec<LolMatchTimeline>>{
        let timelines = sqlx::query_as::<_, LolMatchTimelineQueryResult>(
            "SELECT * FROM lol_match_timelines WHERE lol_match_id = $1"
        ).bind(match_id)
        .fetch_all(db)
        .await?
            .into_iter()
            .map(|x|LolMatchTimeline{
                id: x.id,
                lol_match_id: x.lol_match_id,
                summoner_id: x.summoner_id,
                items_event_timeline: serde_json::from_value(x.items_event_timeline).unwrap(),
                skills_timeline: x.skills_timeline,
            }).collect();
        Ok(timelines)
    }


    pub async fn bulk_insert(db: &PgPool, timelines: Vec<TempLolMatchTimeline>) -> AppResult<()> {
        // Check if timelines vector is empty
        if timelines.is_empty() {
            return Ok(());
        }

        // Prepare the insert SQL with placeholders
        let mut qb = QueryBuilder::new(
            "INSERT INTO lol_match_timelines (lol_match_id, summoner_id, items_event_timeline, skills_timeline) ",
        );

        qb.push_values(timelines.into_iter(), |mut b, rec|{
            let mut items_event_timeline= HashMap::new();
            for (timestamp, event) in rec.items_event_timeline {
                let frame_idx = (timestamp / 60000) as i32;
                items_event_timeline.entry(frame_idx).or_insert_with(Vec::new).push(event);
            }
            // convert to vec
            let mut items_event_timeline: Vec<_> = items_event_timeline.into_iter().collect();
            items_event_timeline.sort_by_key(|x| x.0);
            b.push_bind(rec.lol_match_id);
            b.push_bind(rec.summoner_id);
            b.push_bind(json!(items_event_timeline));
            b.push_bind(rec.skills_timeline.clone());
        });
        qb.build().fetch_all(db).await?;
        Ok(())
    }
}