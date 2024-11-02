use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

use crate::error_template::{AppError, AppResult};
use crate::models::entities::lol_match::LolMatch;
use crate::models::entities::lol_match_participant::{LolMatchParticipant, LolMatchParticipantStats};
use crate::models::entities::summoner::Summoner;
use riven::consts::{Champion, PlatformRoute, RegionalRoute};
use riven::RiotApi;
use sqlx::types::chrono;
use sqlx::types::chrono::{DateTime, Utc};
use futures::stream::{FuturesUnordered, StreamExt};
use crate::{consts, DB_CHUNK_SIZE};

pub async fn update_summoner_matches(
    db: sqlx::PgPool,
    api: Arc<RiotApi>,
    puuid: String,
    platform: PlatformRoute,
    max_matches: usize,
) -> AppResult<()> {
    let match_ids = fetch_all_match_ids(&api, platform.to_regional(), &puuid, max_matches).await?;

    // Fetch existing match IDs from the database
    let existing_match_ids: HashSet<String> = sqlx::query_scalar(
        "SELECT match_id FROM lol_matches WHERE match_id = ANY($1)",
    )
        .bind(&match_ids)
        .fetch_all(&db)
        .await?
        .into_iter()
        .collect();

    // Filter out matches that are already saved
    let new_riot_match_ids: Vec<String> = match_ids
        .into_iter()
        .filter(|id| !existing_match_ids.contains(id))
        .collect();

    println!("New {} match ids for puuid {}", new_riot_match_ids.len(), puuid);
    let t = std::time::Instant::now();
    if !new_riot_match_ids.is_empty() {
        let mut match_ids = Vec::with_capacity(new_riot_match_ids.len());
        for chunk in new_riot_match_ids.chunks(DB_CHUNK_SIZE) {
            match_ids.extend(LolMatch::bulk_default_insert(&db, chunk).await);
        }
        for (riot_ids_chunk, match_ids_chunk) in new_riot_match_ids.chunks(100).zip(match_ids.chunks(100)) {
            update_matches(&db, &api, platform, match_ids_chunk.to_vec(), riot_ids_chunk.to_vec()).await?;
        }
    }
    println!("Updated {} matches in {:?} for {}", new_riot_match_ids.len(), t.elapsed(), puuid);

    Ok(())
}

async fn update_matches(
    db: &sqlx::PgPool,
    api: &Arc<RiotApi>,
    platform: PlatformRoute,
    match_ids: Vec<i32>,
    match_riot_ids: Vec<String>,
) -> AppResult<()> {
    // Collect match data concurrently
    let match_data_futures = match_riot_ids.into_iter().map(|match_id| {
        let api = Arc::clone(&api);
        let platform = platform.clone();

        async move {
            api.match_v5().get_match(platform.to_regional(), &match_id).await
        }
    });

    let mut match_datas: Vec<_> = FuturesUnordered::from_iter(match_data_futures)
        .filter_map(|result| async move { result.ok().flatten() })
        .collect()
        .await;
    match_datas.sort_by_key(|x| -x.info.game_id);

    // Collect TempSummoner data from match data
    let mut participants_map = HashMap::new();
    for match_data in match_datas.iter(){
        let platform_code = match_data
            .metadata
            .match_id
            .split('_')
            .next()
            .unwrap_or_default();
        let match_platform = consts::PlatformRoute::from_str(platform_code).unwrap_or(consts::PlatformRoute::from_region_str(platform.as_region_str()).unwrap());

        for participant in &match_data.info.participants {
            participants_map
                .entry(participant.puuid.clone())
                .or_insert_with(|| TempSummoner {
                    puuid: participant.puuid.clone(),
                    game_name: participant.riot_id_game_name.clone().unwrap_or_default(),
                    tag_line: participant.riot_id_tagline.clone(),
                    platform: match_platform.as_region_str().to_string(),
                    summoner_level: participant.summoner_level as i64,
                    profile_icon_id: participant.profile_icon,
                    updated_at: DateTime::from_timestamp_millis(
                        match_data.info.game_end_timestamp.unwrap_or(0)
                    ).unwrap(),
                });
        }
    }

    // Separate summoners into those to insert and update
    let puuids: Vec<String> = participants_map.keys().cloned().collect();
    let existing_summoners = Summoner::fetch_existing_summoners(db, &puuids).await?;

    let mut summoners_to_insert = Vec::new();
    let mut summoners_to_update = Vec::new();

    for summoner in participants_map.values() {
        if let Some((id, existing_timestamp)) = existing_summoners.get(&summoner.puuid) {
            if summoner.updated_at.timestamp() > *existing_timestamp as i64 {
                summoners_to_update.push(summoner.clone());
            }
        } else {
            summoners_to_insert.push(summoner.clone());
        }
    }

    // Map of puuid to summoner ID
    let mut summoner_map: HashMap<String, i32> = existing_summoners
        .iter()
        .map(|(puuid, (id, _))| (puuid.clone(), *id))
        .collect();

    // Bulk Insert new summoners
    if !summoners_to_insert.is_empty() {
        for summoners_to_insert in summoners_to_insert.chunks(DB_CHUNK_SIZE) {
            let inserted_summoners = Summoner::bulk_insert(db, summoners_to_insert).await?;
            summoner_map.extend(inserted_summoners);
        }
    }

    // Bulk update existing summoners
    if !summoners_to_update.is_empty() {
        for chunk in summoners_to_update.chunks(DB_CHUNK_SIZE) {
            Summoner::bulk_update(db, chunk).await?;
        }
    }


    // Prepare participants for bulk insert
    let match_participants: Vec<TempParticipant> = match_datas
        .iter()
        .zip(match_ids.iter())
        .flat_map(|( match_data, &match_id)| {
            if match_data.info.game_mode == riven::consts::GameMode::STRAWBERRY {
                return vec![];
            }
            let won_team_id = match_data
                .info
                .teams
                .iter()
                .find(|team| team.win)
                .map(|team| team.team_id);

            let team_kills: HashMap<riven::consts::Team, i32> = match_data
                .info
                .teams
                .iter()
                .map(|team| (team.team_id, team.objectives.champion.kills))
                .collect();

            // Instead of returning an iterator, collect the results into a Vec
            match_data
                .info
                .participants
                .iter()
                .map(|participant| {
                    let summoner_id = *summoner_map.get(&participant.puuid).unwrap();
                    let team_kill_count = *team_kills.get(&participant.team_id).unwrap_or(&0);

                    let kda = if participant.deaths == 0 {
                        (participant.kills + participant.assists) as f64
                    } else {
                        (participant.kills + participant.assists) as f64 / participant.deaths as f64
                    };
                    let kda = (kda * 100.0).round() / 100.0;

                    let kill_participation = if team_kill_count == 0 {
                        0.0
                    } else {
                        (participant.kills + participant.assists) as f64 / team_kill_count as f64
                    };
                    let kill_participation = (kill_participation * 100.0).round() / 100.0;
                    let champion_id =  Champion::try_from(participant.champion_name.as_str()).unwrap().0;
                    TempParticipant {
                        champion_id: champion_id,
                        summoner_id,
                        lol_match_id: match_id,
                        summoner_spell1_id: participant.summoner1_id,
                        summoner_spell2_id: participant.summoner2_id,
                        team_id: participant.team_id as i32,
                        won: Some(participant.team_id) == won_team_id,
                        kill_participation,
                        champ_level: participant.champ_level,
                        kda,
                        kills: participant.kills,
                        deaths: participant.deaths,
                        assists: participant.assists,
                        damage_dealt_to_champions: participant.total_damage_dealt_to_champions,
                        damage_taken: participant.total_damage_taken,
                        gold_earned: participant.gold_earned,
                        wards_placed: participant.wards_placed,
                        cs: participant.total_minions_killed,
                        stats: LolMatchParticipantStats {
                            largest_killing_spree: participant.largest_killing_spree,
                            double_kills: participant.double_kills,
                            triple_kills: participant.triple_kills,
                            quadra_kills: participant.quadra_kills,
                            penta_kills: participant.penta_kills,

                        },
                        perk_defense_id: participant.perks.stat_perks.defense,
                        perk_flex_id: participant.perks.stat_perks.flex,
                        perk_offense_id: participant.perks.stat_perks.offense,
                        perk_primary_style_id: participant
                            .perks
                            .styles
                            .get(0)
                            .map_or(0, |style| style.style),
                        perk_sub_style_id: participant
                            .perks
                            .styles
                            .get(1)
                            .map_or(0, |style| style.style),
                        perk_primary_selection_id: participant
                            .perks
                            .styles
                            .get(0)
                            .and_then(|style| style.selections.get(0))
                            .map_or(0, |sel| sel.perk),
                        perk_primary_selection1_id: participant
                            .perks
                            .styles
                            .get(0)
                            .and_then(|style| style.selections.get(1))
                            .map_or(0, |sel| sel.perk),
                        perk_primary_selection2_id: participant
                            .perks
                            .styles
                            .get(0)
                            .and_then(|style| style.selections.get(2))
                            .map_or(0, |sel| sel.perk),
                        perk_primary_selection3_id: participant
                            .perks
                            .styles
                            .get(0)
                            .and_then(|style| style.selections.get(3))
                            .map_or(0, |sel| sel.perk),
                        perk_sub_selection1_id: participant
                            .perks
                            .styles
                            .get(1)
                            .and_then(|style| style.selections.get(0))
                            .map_or(0, |sel| sel.perk),
                        perk_sub_selection2_id: participant
                            .perks
                            .styles
                            .get(1)
                            .and_then(|style| style.selections.get(1))
                            .map_or(0, |sel| sel.perk),
                        item0_id: participant.item0,
                        item1_id: participant.item1,
                        item2_id: participant.item2,
                        item3_id: participant.item3,
                        item4_id: participant.item4,
                        item5_id: participant.item5,
                        item6_id: participant.item6,
                    }
                })
                .collect::<Vec<_>>() // Collect the iterator into a Vec
        })
        .collect(); // Collect the Vecs from each closure into a single Vec

    // Bulk insert participants
    for chunk in match_participants.chunks(DB_CHUNK_SIZE) {
        LolMatchParticipant::bulk_insert(db, chunk).await?;
    }
    // Bulk update matches
    LolMatch::bulk_update(db, &match_datas).await;
    Ok(())
}

async fn fetch_all_match_ids(
    api: &RiotApi,
    region: RegionalRoute,
    puuid: &str,
    max_matches: usize,
) -> AppResult<Vec<String>> {
    let max_fetch_limit = max_matches.min(100);
    let mut matches_list = Vec::new();
    let mut begin_index = 0;

    loop {
        let fetched_matches = fetch_match_ids(api, region, puuid, Some(max_fetch_limit as i32), Some(begin_index)).await?;
        if fetched_matches.is_empty() {
            break;
        }
        begin_index += fetched_matches.len() as i32;
        matches_list.extend(fetched_matches);
        if max_matches != 0 && matches_list.len() >= max_matches {
            matches_list.truncate(max_matches);
            break;
        }
    }

    Ok(matches_list)
}

async fn fetch_match_ids(
    api: &RiotApi,
    region: RegionalRoute,
    puuid: &str,
    count: Option<i32>,
    start: Option<i32>,
) -> AppResult<Vec<String>> {
    api.match_v5()
        .get_match_ids_by_puuid(region, puuid, count, None, None, None, start, None)
        .await
        .map_err(AppError::from)
}

#[derive(Clone)]
pub struct TempSummoner {
    pub game_name: String,
    pub tag_line: String,
    pub puuid: String,
    pub platform: String,
    pub summoner_level: i64,
    pub profile_icon_id: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TempParticipant {
    pub champion_id: i16,
    pub summoner_id: i32,
    pub lol_match_id: i32,
    pub summoner_spell1_id: i32,
    pub summoner_spell2_id: i32,
    pub team_id: i32,
    pub won: bool,
    pub champ_level:i32,
    pub kda: f64,
    pub kill_participation: f64,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub damage_dealt_to_champions: i32,
    pub damage_taken: i32,
    pub gold_earned: i32,
    pub wards_placed: i32,
    pub cs: i32,
    pub stats: LolMatchParticipantStats,
    pub perk_defense_id: i32,
    pub perk_flex_id: i32,
    pub perk_offense_id: i32,
    pub perk_primary_style_id: i32,
    pub perk_sub_style_id: i32,
    pub perk_primary_selection_id: i32,
    pub perk_primary_selection1_id: i32,
    pub perk_primary_selection2_id: i32,
    pub perk_primary_selection3_id: i32,
    pub perk_sub_selection1_id: i32,
    pub perk_sub_selection2_id: i32,
    pub item0_id: i32,
    pub item1_id: i32,
    pub item2_id: i32,
    pub item3_id: i32,
    pub item4_id: i32,
    pub item5_id: i32,
    pub item6_id: i32,
}
