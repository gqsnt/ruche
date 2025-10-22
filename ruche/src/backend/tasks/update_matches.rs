pub mod bulk_lol_match_participants;
pub mod bulk_lol_matches;
pub mod bulk_summoners;

use crate::backend::ssr::{AppError, AppResult, PlatformRouteDb};
use crate::backend::task_director::Task;
use crate::backend::tasks::update_matches::bulk_lol_match_participants::bulk_insert_lol_match_participants;
use crate::backend::tasks::update_matches::bulk_lol_matches::{
    bulk_trashed_matches, bulk_update_matches,
};
use crate::backend::tasks::update_matches::bulk_summoners::{
    bulk_insert_summoners, bulk_update_summoners,
};
use crate::ssr::{RiotApiState, SubscriberMap};
use crate::utils::{ProPlayerSlug, SSEEvent};
use crate::DB_CHUNK_SIZE;
use std::future::Future;
use std::pin::Pin;
use chrono::NaiveDateTime;
use common::consts;
use common::consts::platform_route::PlatformRoute;
use futures::stream::{FuturesOrdered, FuturesUnordered, StreamExt};
use itertools::Itertools;
use leptos::logging::log;
use riven::consts::Champion;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{Duration, Instant};

pub struct UpdateMatchesTask {
    db: PgPool,
    api: RiotApiState,
    update_interval: Duration,
    update_matches_sender: Arc<SubscriberMap>,
    next_run: Instant,
    running: Arc<AtomicBool>,
}

impl UpdateMatchesTask {
    pub fn new(
        db: PgPool,
        api: RiotApiState,
        update_interval: Duration,
        update_matches_sender: Arc<SubscriberMap>,
    ) -> Self {
        let next_run = Instant::now() + update_interval;
        Self {
            db,
            api,
            update_interval,
            update_matches_sender,
            next_run,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Task for UpdateMatchesTask {
    fn execute(&self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        let db = self.db.clone();
        let api = self.api.clone();
        let update_matches_sender = self.update_matches_sender.clone();
        Box::pin(async move {
            while let Ok(matches) = get_not_updated_match(&db, 100).await {
                let start = Instant::now();
                let match_len = matches.len();
                match update_matches_task(&db, &api, matches).await {
                    Ok(summoner_ids) => {
                        for id in summoner_ids {
                            if let Some(sender) = update_matches_sender.get(&id) {
                                let _ = sender.send(SSEEvent::SummonerMatches(0));
                            }
                        }
                        log!("Updated {} matches in {:?}", match_len, start.elapsed());
                    }
                    Err(e) => {
                        log!("Error updating matches: {:?}", e);
                    }
                };
            }
        })
    }

    fn next_execution(&self) -> Instant {
        self.next_run
    }

    fn update_schedule(&mut self) {
        self.next_run = Instant::now() + self.update_interval;
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::SeqCst);
    }

    fn clone_box(&self) -> Box<dyn Task> {
        Box::new(Self {
            db: self.db.clone(),
            api: self.api.clone(),
            update_interval: self.update_interval,
            update_matches_sender: self.update_matches_sender.clone(),
            next_run: self.next_run,
            running: self.running.clone(),
        })
    }

    fn name(&self) -> &'static str {
        "UpdateMatchesTask"
    }

    fn allow_concurrent(&self) -> bool {
        false // Do not allow concurrent executions
    }
}

async fn update_matches_task(
    db: &PgPool,
    api: &RiotApiState,
    matches_to_update: Vec<LolMatchNotUpdated>,
) -> AppResult<HashSet<i32>> {
    let match_data_futures = matches_to_update.iter().map(|match_| {
        let api = Arc::clone(api);
        let pt = consts::platform_route::PlatformRoute::from(match_.platform).to_riven();
        async move {
            api.match_v5()
                .get_match(pt.to_regional(), &match_.match_id)
                .await
        }
    });

    let match_raw_datas: Vec<_> = FuturesOrdered::from_iter(match_data_futures)
        .collect()
        .await;

    let ( trashed_matches, match_datas): (Vec<_>, Vec<_>) = match_raw_datas
        .into_iter()
        .zip(matches_to_update.into_iter())
        .partition(|(match_, _)| {
            match match_{
                Ok(Some(match_)) => {
                    match_.info.game_mode == riven::consts::GameMode::STRAWBERRY
                        || match_.info.game_version.is_empty()
                        || match_.info.game_id == 0
                }
                Ok(None) => {
                    log!("Match not found");
                    true
                },
                Err(e) => {
                    log!("Error fetching match: {:?}", e);
                    true
                }
            }
        });
    let trashed_matches:Vec<_> = trashed_matches
        .into_iter()
        .map(|(match_, match_not_updated)| (match_,match_not_updated))
        .collect();
    let match_datas = match_datas
        .into_iter()
        .map(|(match_, match_not_updated)| (match_.unwrap().unwrap(), match_not_updated))
        .collect_vec();

    // Collect TempSummoner data from match data
    let mut participants_map = HashMap::new();
    for (match_data, _) in match_datas.iter() {
        let platform_code = match_data
            .metadata
            .match_id
            .split('_')
            .next()
            .unwrap_or_default();
        let match_platform = consts::platform_route::PlatformRoute::try_from(platform_code).unwrap_or_default();

        for participant in &match_data.info.participants {
            if participant.puuid == "BOT" {
                continue;
            }
            participants_map
                .entry(participant.puuid.clone())
                .or_insert_with(|| TempSummoner {
                    puuid: participant.puuid.clone(),
                    game_name: participant.riot_id_game_name.clone().unwrap_or_default(),
                    tag_line: participant.riot_id_tagline.clone().unwrap_or_default(),
                    platform: match_platform.to_string(),
                    summoner_level: participant.summoner_level,
                    profile_icon_id: participant.profile_icon as u16,
                    updated_at: DateTime::from_timestamp_millis(
                        match_data.info.game_end_timestamp.unwrap_or(0),
                    )
                    .expect("update match task:timestamp error"),
                });
        }
    }

    // Separate summoners into those to insert and update
    let puuids: Vec<String> = participants_map.keys().cloned().collect();
    let existing_summoners = fetch_existing_summoners(db, &puuids).await?;

    let mut summoners_to_insert = Vec::new();
    let mut summoners_to_update = Vec::new();
    let mut summoners_to_dl = Vec::new();

    for summoner in participants_map.values() {
        let game_name = summoner.game_name.trim();
        let tag_line = summoner.tag_line.trim();
        if let Some((_, existing_timestamp)) = existing_summoners.get(&summoner.puuid) {
            if summoner.updated_at.timestamp() > *existing_timestamp as i64
                && !game_name.is_empty()
                && !tag_line.is_empty()
            {
                summoners_to_update.push(summoner.clone());
            }
        } else if game_name.is_empty() || tag_line.is_empty() {
            summoners_to_dl.push(summoner.clone());
        } else {
            summoners_to_insert.push(summoner.clone());
        }
    }

    if !summoners_to_dl.is_empty() {
        log!("Summoners to download: {}", summoners_to_dl.len());
    }
    // dl summoners
    let summoners_futures = summoners_to_dl.into_iter().map(|summoner| {
        let api = Arc::clone(api);
        let pt = consts::platform_route::PlatformRoute::try_from(summoner.platform.as_str()).unwrap_or_default().to_riven();
        let puuid = summoner.puuid.clone();
        async move {
            (
                summoner,
                api.account_v1()
                    .get_by_puuid(pt.to_regional(), &puuid)
                    .await,
            )
        }
    });

    let summoners_data: Vec<_> = FuturesUnordered::from_iter(summoners_futures)
        .collect()
        .await;

    for (mut summoner, summoner_data) in summoners_data {
        match summoner_data {
            Ok(account) => {
                if account.game_name.is_some() && account.tag_line.is_some() {
                    summoner.game_name = account.game_name.unwrap();
                    summoner.tag_line = account.tag_line.unwrap();
                    summoners_to_insert.push(summoner);
                }
            }
            Err(e) => {
                log!("Summoner not found: {:?} on ", summoner);
                log!("Error: {:?}", e);
            }
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
            let inserted_summoners = bulk_insert_summoners(db, summoners_to_insert).await?;
            summoner_map.extend(
                inserted_summoners
                    .into_iter()
                    .map(|(puuid, summoner_full)| (puuid, summoner_full.id))
                    .collect::<HashMap<String, i32>>(),
            );
        }
    }

    // Bulk update existing summoners
    if !summoners_to_update.is_empty() {
        for chunk in summoners_to_update.chunks(DB_CHUNK_SIZE) {
            bulk_update_summoners(db, chunk).await?;
        }
    }
    resolve_summoner_conflicts(db, api).await?;

    // Prepare participants for bulk insert
    let match_participants: Vec<TempParticipant> = match_datas
        .iter()
        .flat_map(|(match_data, match_)| {
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
                .filter_map(|participant| {
                    if participant.puuid == "BOT" {
                        return None;
                    }
                    let summoner_id =
                        if let Some(summoner_id) = summoner_map.get(&participant.puuid) {
                            *summoner_id
                        } else {
                            return None;
                        };
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
                    let champion_id = Champion::try_from(participant.champion_name.as_str())
                        .unwrap()
                        .0;
                    Some(TempParticipant {
                        champion_id,
                        summoner_id,
                        lol_match_id: match_.id,
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
                        cs_per_minute: participant.total_minions_killed as f64
                            / (match_data.info.game_duration as f64 / 60.0),
                        double_kills: participant.double_kills,
                        triple_kills: participant.triple_kills,
                        quadra_kills: participant.quadra_kills,
                        penta_kills: participant.penta_kills,
                        perk_defense_id: participant.perks.stat_perks.defense,
                        perk_flex_id: participant.perks.stat_perks.flex,
                        perk_offense_id: participant.perks.stat_perks.offense,
                        perk_primary_style_id: participant
                            .perks
                            .styles
                            .first()
                            .map_or(0, |style| style.style),
                        perk_sub_style_id: participant
                            .perks
                            .styles
                            .get(1)
                            .map_or(0, |style| style.style),
                        perk_primary_selection_id: participant
                            .perks
                            .styles
                            .first()
                            .and_then(|style| style.selections.first())
                            .map_or(0, |sel| sel.perk),
                        perk_primary_selection1_id: participant
                            .perks
                            .styles
                            .first()
                            .and_then(|style| style.selections.get(1))
                            .map_or(0, |sel| sel.perk),
                        perk_primary_selection2_id: participant
                            .perks
                            .styles
                            .first()
                            .and_then(|style| style.selections.get(2))
                            .map_or(0, |sel| sel.perk),
                        perk_primary_selection3_id: participant
                            .perks
                            .styles
                            .first()
                            .and_then(|style| style.selections.get(3))
                            .map_or(0, |sel| sel.perk),
                        perk_sub_selection1_id: participant
                            .perks
                            .styles
                            .get(1)
                            .and_then(|style| style.selections.first())
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
                    })
                })
                .collect::<Vec<_>>() // Collect the iterator into a Vec
        })
        .collect(); // Collect the Vecs from each closure into a single Vec

    // Bulk insert participants
    for chunk in match_participants.chunks(DB_CHUNK_SIZE) {
        bulk_insert_lol_match_participants(db, chunk).await?;
    }
    // Bulk update matches
    bulk_update_matches(db, match_datas).await?;
    bulk_trashed_matches(db, trashed_matches).await?;
    Ok(summoner_map.into_values().collect::<HashSet<i32>>())
}

#[derive(Clone, Debug)]
pub struct TempSummoner {
    pub game_name: String,
    pub tag_line: String,
    pub puuid: String,
    pub platform: String,
    pub summoner_level: i32,
    pub profile_icon_id: u16,
    pub updated_at: DateTime<Utc>,
}
#[derive(Clone, Debug)]
pub struct SummonerFull {
    pub id: i32,
    pub game_name: String,
    pub tag_line: String,
    pub puuid: String,
    pub platform: PlatformRoute,
    pub summoner_level: i32,
    pub profile_icon_id: u16,
    pub pro_player_slug: Option<ProPlayerSlug>,
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
    pub champ_level: i32,
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
    pub cs_per_minute: f64,
    pub double_kills: i32,
    pub triple_kills: i32,
    pub quadra_kills: i32,
    pub penta_kills: i32,
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

pub async fn get_not_updated_match(db: &PgPool, limit: i32) -> AppResult<Vec<LolMatchNotUpdated>> {
    let result = sqlx::query_as::<_, LolMatchNotUpdated>(
        r#"
            SELECT id, match_id, platform, updated FROM lol_matches
            WHERE updated = false
            ORDER BY match_id DESC
            LIMIT $1;
        "#,
    )
    .bind(limit)
    .fetch_all(db)
    .await?;
    if !result.is_empty() {
        Ok(result)
    } else {
        Err(AppError::CustomError("No matches to update".to_string()))
    }
}

pub async fn fetch_existing_summoners(
    db: &PgPool,
    puuids: &[String],
) -> AppResult<HashMap<String, (i32, i32)>> {
    Ok(sqlx::query_as::<_, SummonerShortModel>(
        "
            SELECT puuid, id, updated_at
            FROM summoners
            WHERE puuid = ANY($1)
        ",
    )
    .bind(puuids)
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|row| {
        (
            row.puuid,
            (row.id, row.updated_at.and_utc().timestamp() as i32),
        )
    })
    .collect::<HashMap<String, (i32, i32)>>())
}

pub async fn resolve_summoner_conflicts(db: &PgPool, api: &RiotApiState) -> AppResult<()> {
    let conflicts = find_conflicting_summoners(db).await?;
    for (game_name, tag_line, platform, conflict_records) in conflicts {
        println!(
            "Resolving conflict for {}#{} on {} with {:?}",
            game_name, tag_line, platform, conflict_records
        );
        for record in conflict_records {
            // Obtenir les informations actuelles pour chaque `puuid`
            let platform_route = PlatformRoute::try_from(platform.as_str()).unwrap_or_default();
            let riven_ptr =
                riven::consts::PlatformRoute::from_str(&platform_route.to_string()).unwrap();
            if let Ok(account) = api
                .account_v1()
                .get_by_puuid(riven_ptr.to_regional(), &record.puuid)
                .await
            {
                update_summoner_account_by_id(db, record.id, account).await?;
            }
        }
    }

    Ok(())
}

pub async fn find_conflicting_summoners(
    db: &PgPool,
) -> AppResult<Vec<(String, String, String, Vec<SummonerModel>)>> {
    Ok(sqlx::query_as::<_, SummonerModel>(
        "SELECT *
        FROM summoners
        WHERE (game_name, tag_line, platform) IN (
            SELECT game_name, tag_line, platform
            FROM summoners
            GROUP BY game_name, tag_line, platform
            HAVING COUNT(*) > 1
        )
        ORDER BY game_name, tag_line, platform, updated_at DESC",
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .fold(
        HashMap::new(),
        |mut acc: HashMap<(String, String, String), Vec<SummonerModel>>, row| {
            acc.entry((
                row.game_name.clone(),
                row.tag_line.clone(),
                row.platform.to_string(),
            ))
            .or_default()
            .push(row);
            acc
        },
    )
    .into_iter()
    .map(|((game_name, tag_line, platform), ids)| (game_name, tag_line, platform, ids))
    .collect())
}

pub async  fn delete_summoner_account_by_id(db: &PgPool, id: i32) -> AppResult<()> {
    sqlx::query("DELETE FROM summoners WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn update_summoner_account_by_id(
    db: &PgPool,
    id: i32,
    account: riven::models::account_v1::Account,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE summoners SET game_name = $1, tag_line = $2 , updated_at = NOW() WHERE id = $3",
    )
    .bind(account.game_name.unwrap_or_default().trim())
    .bind(account.tag_line.unwrap_or_default().trim())
    .bind(id)
    .execute(db)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow, Debug)]
pub struct SummonerModel {
    pub id: i32,
    pub game_name: String,
    pub tag_line: String,
    pub puuid: String,
    pub platform: PlatformRouteDb,
    pub updated_at: NaiveDateTime,
    pub summoner_level: i32,
    pub profile_icon_id: i32,
}

#[derive(sqlx::FromRow, Debug)]
pub struct SummonerShortModel {
    pub id: i32,
    pub puuid: String,
    pub updated_at: NaiveDateTime,
}

#[derive(FromRow, Debug)]
pub struct LolMatchNotUpdated {
    pub id: i32,
    pub match_id: String,
    pub platform: PlatformRouteDb,
    pub updated: bool,
}
