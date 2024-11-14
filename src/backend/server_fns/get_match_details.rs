#[cfg(feature = "ssr")]
use crate::backend::updates::update_match_timeline::update_match_timeline;
use crate::views::summoner_page::match_details::LolMatchParticipantDetails;
use leptos::prelude::*;
use leptos::server;

#[server]
pub async fn get_match_details(match_id: i32, riot_match_id: String, platform: String) -> Result<Vec<LolMatchParticipantDetails>, ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();

    let mut details = ssr::get_match_participants_details(&db, match_id).await.map_err(|e| e.to_server_fn_error())?;
    let mut match_timelines = ssr::get_match_timeline(&db, match_id).await?;
    if match_timelines.is_empty() {
        update_match_timeline(&db, state.riot_api.clone(), match_id, riot_match_id, platform).await?;
        match_timelines = ssr::get_match_timeline(&db, match_id).await?;
    }
    for detail in details.iter_mut() {
        if let Some(timeline) = match_timelines.iter().find(|x| x.summoner_id == detail.summoner_id).cloned() {
            detail.items_event_timeline = timeline.items_event_timeline;
            detail.skills_timeline = timeline.skills_timeline;
        }
    }
    Ok(details)
}


#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::ssr::AppResult;
    use crate::views::summoner_page::match_details::{LolMatchParticipantDetails, LolMatchTimeline};
    use bigdecimal::{BigDecimal, ToPrimitive};
    use sqlx::{FromRow, PgPool};
    use sqlx::types::JsonValue;

    pub async fn get_match_participants_details(db: &PgPool, match_id: i32) -> AppResult<Vec<LolMatchParticipantDetails>> {
        Ok(sqlx::query_as::<_, LolMatchParticipantDetailsModel>(
            "SELECT
                lmp.id,
                lmp.lol_match_id,
                lmp.summoner_id,
                ss.game_name AS summoner_name,
                ss.tag_line AS summoner_tag_line,
                ss.platform AS summoner_platform,
                ss.profile_icon_id AS summoner_icon_id,
                ss.summoner_level AS summoner_level,
                lmp.champion_id,
                lmp.team_id,
                lmp.won,
                lmp.kills,
                lmp.deaths,
                lmp.assists,
                lmp.champ_level,
                lmp.kda,
                lmp.kill_participation,
                lmp.damage_dealt_to_champions,
                lmp.damage_taken,
                lmp.gold_earned,
                lmp.wards_placed,
                lmp.cs,
                lmp.summoner_spell1_id,
                lmp.summoner_spell2_id,
                lmp.perk_defense_id,
                lmp.perk_flex_id,
                lmp.perk_offense_id,
                lmp.perk_primary_style_id,
                lmp.perk_sub_style_id,
                lmp.perk_primary_selection_id,
                lmp.perk_primary_selection1_id,
                lmp.perk_primary_selection2_id,
                lmp.perk_primary_selection3_id,
                lmp.perk_sub_selection1_id,
                lmp.perk_sub_selection2_id,
                lmp.item0_id,
                lmp.item1_id,
                lmp.item2_id,
                lmp.item3_id,
                lmp.item4_id,
                lmp.item5_id,
                lmp.item6_id
            FROM lol_match_participants  as lmp
            INNER JOIN (SELECT id,game_name, tag_line, platform, profile_icon_id, summoner_level FROM summoners) as ss ON ss.id = lmp.summoner_id
            WHERE lmp.lol_match_id = $1;",
        )
            .bind(match_id)
            .fetch_all(db)
            .await?
            .into_iter().map(|lmp| {
            LolMatchParticipantDetails {
                id: lmp.id,
                lol_match_id: lmp.lol_match_id,
                summoner_id: lmp.summoner_id,
                summoner_name: lmp.summoner_name,
                summoner_tag_line: lmp.summoner_tag_line,
                summoner_platform: lmp.summoner_platform,
                summoner_icon_id: lmp.summoner_icon_id as u16,
                summoner_level: lmp.summoner_level,
                champion_id: lmp.champion_id as u16,
                team_id: lmp.team_id,
                won: lmp.won,
                kills: lmp.kills,
                deaths: lmp.deaths,
                assists: lmp.assists,
                champ_level: lmp.champ_level,
                kda: lmp.kda.map_or(0.0, |bd| bd.to_f64().unwrap_or(0.0)),
                kill_participation: lmp.kill_participation.map_or(0.0, |bd| bd.to_f64().unwrap_or(0.0)),
                damage_dealt_to_champions: lmp.damage_dealt_to_champions,
                damage_taken: lmp.damage_taken,
                gold_earned: lmp.gold_earned,
                wards_placed: lmp.wards_placed,
                cs: lmp.cs,
                summoner_spell1_id: lmp.summoner_spell1_id.unwrap_or_default() as u16,
                summoner_spell2_id: lmp.summoner_spell2_id.unwrap_or_default() as u16,
                perk_defense_id: lmp.perk_defense_id.unwrap_or_default() as u16,
                perk_flex_id: lmp.perk_flex_id.unwrap_or_default() as u16,
                perk_offense_id: lmp.perk_offense_id.unwrap_or_default() as u16,
                perk_primary_style_id: lmp.perk_primary_style_id.unwrap_or_default() as u16,
                perk_sub_style_id: lmp.perk_sub_style_id.unwrap_or_default() as u16,
                perk_primary_selection_id: lmp.perk_primary_selection_id.unwrap_or_default() as u16,
                perk_primary_selection1_id: lmp.perk_primary_selection1_id.unwrap_or_default() as u16,
                perk_primary_selection2_id: lmp.perk_primary_selection2_id.unwrap_or_default() as u16,
                perk_primary_selection3_id: lmp.perk_primary_selection3_id.unwrap_or_default() as u16,
                perk_sub_selection1_id: lmp.perk_sub_selection1_id.unwrap_or_default() as u16,
                perk_sub_selection2_id: lmp.perk_sub_selection2_id.unwrap_or_default() as u16,
                item0_id: lmp.item0_id.unwrap_or_default() as u32,
                item1_id: lmp.item1_id.unwrap_or_default() as u32,
                item2_id: lmp.item2_id.unwrap_or_default() as u32,
                item3_id: lmp.item3_id.unwrap_or_default() as u32,
                item4_id: lmp.item4_id.unwrap_or_default() as u32,
                item5_id: lmp.item5_id.unwrap_or_default() as u32,
                item6_id: lmp.item6_id.unwrap_or_default() as u32,
                items_event_timeline: Vec::new(),
                skills_timeline: vec![],
            }
        }).collect::<Vec<_>>())
    }


    pub async fn get_match_timeline(db: &PgPool, match_id: i32) -> AppResult<Vec<LolMatchTimeline>> {
        let timelines = sqlx::query_as::<_, LolMatchTimelineModel>(
            "SELECT * FROM lol_match_timelines WHERE lol_match_id = $1"
        ).bind(match_id)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|x| LolMatchTimeline {
                id: x.id,
                lol_match_id: x.lol_match_id,
                summoner_id: x.summoner_id,
                items_event_timeline: serde_json::from_value(x.items_event_timeline).unwrap_or_default(),
                skills_timeline: x.skills_timeline,
            }).collect();
        Ok(timelines)
    }

    #[derive(FromRow)]
    struct LolMatchParticipantDetailsModel {
        pub id: i32,
        pub lol_match_id: i32,
        pub summoner_id: i32,
        pub summoner_name: String,
        pub summoner_tag_line: String,
        pub summoner_platform: String,
        pub summoner_icon_id: i32,
        pub summoner_level: i64,
        pub champion_id: i32,
        pub team_id: i32,
        pub won: bool,
        pub kills: i32,
        pub deaths: i32,
        pub assists: i32,
        pub champ_level: i32,
        pub kda: Option<BigDecimal>,
        pub kill_participation: Option<BigDecimal>,
        pub damage_dealt_to_champions: i32,
        pub damage_taken: i32,
        pub gold_earned: i32,
        pub wards_placed: i32,
        pub cs: i32,
        pub summoner_spell1_id: Option<i32>,
        pub summoner_spell2_id: Option<i32>,
        pub perk_defense_id: Option<i32>,
        pub perk_flex_id: Option<i32>,
        pub perk_offense_id: Option<i32>,
        pub perk_primary_style_id: Option<i32>,
        pub perk_sub_style_id: Option<i32>,
        pub perk_primary_selection_id: Option<i32>,
        pub perk_primary_selection1_id: Option<i32>,
        pub perk_primary_selection2_id: Option<i32>,
        pub perk_primary_selection3_id: Option<i32>,
        pub perk_sub_selection1_id: Option<i32>,
        pub perk_sub_selection2_id: Option<i32>,
        pub item0_id: Option<i64>,
        pub item1_id: Option<i64>,
        pub item2_id: Option<i64>,
        pub item3_id: Option<i64>,
        pub item4_id: Option<i64>,
        pub item5_id: Option<i64>,
        pub item6_id: Option<i64>,
    }


    #[derive(FromRow)]
    struct LolMatchTimelineModel {
        pub id: i32,
        pub lol_match_id: i32,
        pub summoner_id: i32,
        pub items_event_timeline: JsonValue,
        pub skills_timeline: Vec<i32>,
    }
}