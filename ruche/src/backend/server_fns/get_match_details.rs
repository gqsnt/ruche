use crate::utils::RiotMatchId;
use crate::views::summoner_page::match_details::LolMatchParticipantDetails;
use common::consts::platform_route::PlatformRoute;
use leptos::prelude::*;
use leptos::server;
use leptos::server_fn::codec::Bitcode;

#[cfg(feature = "ssr")]
use update_match_timeline::update_match_timeline;

#[cfg(feature = "ssr")]
pub mod update_match_timeline;

#[server(input=Bitcode,output=Bitcode)]
pub async fn get_match_details(
    match_id: i32,
    summoner_id: Option<i32>,
    platform: PlatformRoute,
    riot_match_id: RiotMatchId,
) -> Result<Vec<LolMatchParticipantDetails>, ServerFnError> {
    let state = expect_context::<crate::ssr::AppState>();
    let db = state.db.clone();

    let (details, match_timelines) = tokio::join!(
        ssr::get_match_participants_details(&db, match_id, summoner_id),
        ssr::get_match_timeline(&db, match_id)
    );
    let mut details = details?;
    let mut match_timelines = match_timelines?;

    if match_timelines.is_empty() {
        update_match_timeline(
            &db,
            state.riot_api.clone(),
            match_id,
            riot_match_id.as_ref(),
            platform,
        )
        .await?;
        match_timelines = ssr::get_match_timeline(&db, match_id).await?;
    }
    for detail in details.iter_mut() {
        if let Some(timeline) = match_timelines
            .iter()
            .find(|x| x.summoner_id == detail.summoner_id)
            .cloned()
        {
            detail.items_event_timeline = timeline.items_event_timeline;
            detail.skills_timeline = timeline.skills_timeline;
        }
    }
    Ok(details)
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::server_fns::get_matches::ssr::get_summoner_encounters;
    use crate::backend::ssr::{AppResult, PlatformRouteDb};
    use crate::utils::ProPlayerSlug;
    use crate::views::summoner_page::match_details::{
        LolMatchParticipantDetails, LolMatchTimeline, Skill,
    };
    use bigdecimal::{BigDecimal, ToPrimitive};
    use itertools::Itertools;
    use sqlx::types::JsonValue;
    use sqlx::{FromRow, PgPool};
    use std::collections::HashMap;

    pub async fn get_match_participants_details(
        db: &PgPool,
        match_id: i32,
        summoner_id: Option<i32>,
    ) -> AppResult<Vec<LolMatchParticipantDetails>> {
        let lol_match_participant_details = sqlx::query_as::<_, LolMatchParticipantDetailsModel>(
            r#"
            SELECT
                lmp.id,
               lmp.lol_match_id,
               lmp.summoner_id,
               ss.game_name,
               ss.tag_line,
               ss.platform,
               ss.summoner_level,
               ss.profile_icon_id,
               ss.pro_player_slug,
               lmp.champion_id,
               lmp.team_id,
               lmp.won,
               lmp.kills,
               lmp.deaths,
               lmp.assists,
               lmp.champ_level,
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
            FROM lol_match_participants as lmp
                left JOIN summoners as ss ON ss.id = lmp.summoner_id
            WHERE lmp.lol_match_id = $1;
        "#,
        )
        .bind(match_id)
        .fetch_all(db)
        .await?;
        let unique_summoner_ids = lol_match_participant_details
            .iter()
            .map(|p| p.summoner_id)
            .unique()
            .collect::<Vec<_>>();
        let encounters = if let Some(summoner_id) = summoner_id {
            get_summoner_encounters(db, summoner_id, &unique_summoner_ids).await?
        } else {
            HashMap::new()
        };
        Ok(lol_match_participant_details
            .into_iter()
            .map(|lmp| LolMatchParticipantDetails {
                id: lmp.id,
                lol_match_id: lmp.lol_match_id,
                summoner_id: lmp.summoner_id,
                game_name: lmp.game_name,
                tag_line: lmp.tag_line,
                platform: lmp.platform.into(),
                summoner_pro_player_slug: lmp
                    .pro_player_slug
                    .map(|s| ProPlayerSlug::new(s.as_str())),
                summoner_icon_id: lmp.profile_icon_id as u16,
                summoner_level: lmp.summoner_level as u16,
                encounter_count: encounters
                    .get(&lmp.summoner_id)
                    .cloned()
                    .unwrap_or_default(),
                champion_id: lmp.champion_id as u16,
                team_id: lmp.team_id as u16,
                won: lmp.won,
                kills: lmp.kills as u16,
                deaths: lmp.deaths as u16,
                assists: lmp.assists as u16,
                champ_level: lmp.champ_level as u16,
                kill_participation: (lmp
                    .kill_participation
                    .map_or(0.0, |bd| bd.to_f32().unwrap_or(0.0))
                    * 100.0)
                    .round() as u16,
                damage_dealt_to_champions: lmp.damage_dealt_to_champions as u32,
                damage_taken: lmp.damage_taken as u32,
                gold_earned: lmp.gold_earned as u32,
                wards_placed: lmp.wards_placed as u16,
                cs: lmp.cs as u16,
                summoner_spell1_id: lmp.summoner_spell1_id.unwrap_or_default() as u16,
                summoner_spell2_id: lmp.summoner_spell2_id.unwrap_or_default() as u16,
                perk_defense_id: lmp.perk_defense_id.unwrap_or_default() as u16,
                perk_flex_id: lmp.perk_flex_id.unwrap_or_default() as u16,
                perk_offense_id: lmp.perk_offense_id.unwrap_or_default() as u16,
                perk_primary_style_id: lmp.perk_primary_style_id.unwrap_or_default() as u16,
                perk_sub_style_id: lmp.perk_sub_style_id.unwrap_or_default() as u16,
                perk_primary_selection_id: lmp.perk_primary_selection_id.unwrap_or_default() as u16,
                perk_primary_selection1_id: lmp.perk_primary_selection1_id.unwrap_or_default()
                    as u16,
                perk_primary_selection2_id: lmp.perk_primary_selection2_id.unwrap_or_default()
                    as u16,
                perk_primary_selection3_id: lmp.perk_primary_selection3_id.unwrap_or_default()
                    as u16,
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
            })
            .collect_vec())
    }

    pub async fn get_match_timeline(
        db: &PgPool,
        match_id: i32,
    ) -> AppResult<Vec<LolMatchTimeline>> {
        let timelines = sqlx::query_as::<_, LolMatchTimelineModel>(
            "SELECT * FROM lol_match_timelines WHERE lol_match_id = $1",
        )
        .bind(match_id)
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|x| LolMatchTimeline {
            id: x.id,
            lol_match_id: x.lol_match_id,
            summoner_id: x.summoner_id,
            items_event_timeline: serde_json::from_value(x.items_event_timeline)
                .unwrap_or_default(),
            skills_timeline: x
                .skills_timeline
                .into_iter()
                .map(|s| Skill::from(s as u8))
                .collect_vec(),
        })
        .collect();
        Ok(timelines)
    }

    #[derive(FromRow)]
    struct LolMatchParticipantDetailsModel {
        pub id: i32,
        pub lol_match_id: i32,
        pub summoner_id: i32,
        pub game_name: String,
        pub tag_line: String,
        pub platform: PlatformRouteDb,
        pub summoner_level: i32,
        pub profile_icon_id: i32,
        pub pro_player_slug: Option<String>,
        pub champion_id: i32,
        pub team_id: i32,
        pub won: bool,
        pub kills: i32,
        pub deaths: i32,
        pub assists: i32,
        pub champ_level: i32,
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
