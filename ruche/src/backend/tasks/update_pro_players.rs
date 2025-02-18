use crate::backend::ssr::{AppError, AppResult, PlatformRouteDb};
use crate::backend::task_director::Task;
use crate::backend::tasks::calculate_next_run_to_fixed_start_hour;
use crate::backend::tasks::update_matches::bulk_summoners::bulk_insert_summoners;
use crate::backend::tasks::update_matches::TempSummoner;
use crate::ssr::RiotApiState;
use crate::DB_CHUNK_SIZE;
use axum::async_trait;
use chrono::Utc;
use common::consts::platform_route::PlatformRoute;
use futures::stream::FuturesUnordered;
use futures::{stream, StreamExt};
use itertools::Itertools;
use leptos::logging::log;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::Instant;

pub struct UpdateProPlayerTask {
    db: PgPool,
    api: RiotApiState,
    start_hour: u32,
    next_run: Instant,
    running: Arc<AtomicBool>,
}

impl UpdateProPlayerTask {
    pub fn new(db: PgPool, api: RiotApiState, start_hour: u32, on_startup: bool) -> Self {
        let next_run = if on_startup {
            Instant::now()
        } else {
            calculate_next_run_to_fixed_start_hour(start_hour)
        };
        Self {
            db,
            api,
            start_hour,
            next_run,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl Task for UpdateProPlayerTask {
    async fn execute(&self) {
        if let Err(e) = update_pro_player(&self.db, self.api.clone()).await {
            log!("Failed to update pro player data: {:?}", e);
        }
    }

    fn next_execution(&self) -> Instant {
        self.next_run
    }

    fn update_schedule(&mut self) {
        self.next_run = calculate_next_run_to_fixed_start_hour(self.start_hour);
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
            start_hour: self.start_hour,
            next_run: self.next_run,
            running: self.running.clone(),
        })
    }

    fn name(&self) -> &'static str {
        "UpdateProPlayerTask"
    }

    fn allow_concurrent(&self) -> bool {
        false // Do not allow concurrent executions
    }
}

pub async fn update_pro_player(db: &PgPool, api: RiotApiState) -> AppResult<()> {
    let mut start = Instant::now();
    let pro_players = get_all_pro_players().await?;
    log!("Found {} Pro players", pro_players.len());
    // Define the concurrency limit
    let concurrency_limit = 4;

    // Create a stream of futures with limited concurrency
    let pro_players_data = stream::iter(
        pro_players
            .into_iter()
            .map(|slug| async move { get_pro_player_info(slug.as_str()).await }),
    )
    .buffer_unordered(concurrency_limit)
    .filter_map(|response| async {
        match response {
            Ok(r) => Some(r),
            Err(e) => {
                log!("Failed to fetch pro player: {:?}", e);
                None
            }
        }
    })
    .collect::<Vec<_>>()
    .await;
    log!("Time to fetch pro_data: {:?}", start.elapsed());
    start = Instant::now();
    let pro_accounts = pro_players_data
        .iter()
        .flat_map(|pro_player| {
            pro_player
                .accounts
                .iter()
                .cloned()
                .map(|acc| (acc, pro_player.slug.clone()))
                .collect_vec()
        })
        .collect::<HashMap<ProPlayerAccountShort, String>>();
    let keys = pro_accounts.keys().cloned().collect_vec();
    let mut existing_summoner_ids = fetch_existing_accounts(db, &keys).await?;
    let not_found_accounts = keys
        .iter()
        .filter(|&account| !existing_summoner_ids.keys().contains(account))
        .collect::<Vec<_>>();
    println!("Not found accounts: {:?}", not_found_accounts.len());

    // dl summoners
    let summoners_futures = not_found_accounts.into_iter().map(|pro_player_account| {
        let api = api.clone();
        let pt = PlatformRoute::from(pro_player_account.platform).to_riven();
        async move {
            let response = api
                .account_v1()
                .get_by_riot_id(
                    pt.to_regional(),
                    pro_player_account.game_name.as_str(),
                    pro_player_account.tag_line.as_str(),
                )
                .await;
            match response {
                Ok(Some(account)) => Ok(TempSummoner {
                    game_name: account.game_name.unwrap_or_default(),
                    tag_line: account.tag_line.unwrap_or_default(),
                    puuid: account.puuid,
                    platform: pro_player_account.platform.to_string(),
                    summoner_level: 0,
                    profile_icon_id: 0,
                    updated_at: Utc::now(),
                }),
                _ => Err(AppError::CustomError(format!(
                    "Summoner not found: {:?}",
                    pro_player_account
                ))),
            }
        }
    });
    println!("Fetching summoners");
    let summoners_to_insert: Vec<_> = FuturesUnordered::from_iter(summoners_futures)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|result| result.ok())
        .collect::<Vec<_>>();
    println!("Fetched summoners: {:?}", summoners_to_insert.len());
    for chunk in summoners_to_insert.chunks(DB_CHUNK_SIZE) {
        let inserted_summoners = bulk_insert_summoners(db, chunk).await?;
        inserted_summoners
            .into_iter()
            .for_each(|(_, summoner_full)| {
                existing_summoner_ids.insert(
                    ProPlayerAccountShort {
                        game_name: summoner_full.game_name,
                        tag_line: summoner_full.tag_line,
                        platform: summoner_full.platform,
                    },
                    summoner_full.id,
                );
            })
    }
    println!("Time taken to fetch summoners: {:?}", start.elapsed());
    start = Instant::now();
    //let pro_players_db = mass_upsert_pro_players(db, &pro_players_data).await?;
    remove_pro_players_from_summoners(db).await?;
    mass_update_adding_pro_player_to_summoners(db, existing_summoner_ids, pro_accounts).await?;
    println!("Time taken to update pro players: {:?}", start.elapsed());
    Ok(())
}

pub async fn remove_pro_players_from_summoners(db: &PgPool) -> AppResult<()> {
    let query = r#"
        UPDATE summoners
        SET pro_player_slug = null
        where pro_player_slug != null
    "#;
    sqlx::query(query).execute(db).await?;
    Ok(())
}

pub async fn mass_update_adding_pro_player_to_summoners(
    db: &PgPool,
    summoner_ids: HashMap<ProPlayerAccountShort, i32>,
    pro_players_data: HashMap<ProPlayerAccountShort, String>,
) -> AppResult<()> {
    let (summoner_ids, pro_player_slugs): (Vec<_>, Vec<_>) = summoner_ids
        .iter()
        .map(|(account, id)| (*id, pro_players_data.get(account).unwrap().clone()))
        .multiunzip();
    let query = r#"
        UPDATE summoners
        SET pro_player_slug = data.pro_player_slug
        FROM (SELECT UNNEST($1::INT[]) AS summoner_id, UNNEST($2::TEXT[]) AS pro_player_slug) AS data
        WHERE summoners.id = data.summoner_id
    "#;
    sqlx::query(query)
        .bind(&summoner_ids)
        .bind(&pro_player_slugs)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn fetch_existing_accounts(
    db: &PgPool,
    player_shorts: &[ProPlayerAccountShort],
) -> AppResult<HashMap<ProPlayerAccountShort, i32>> {
    let (platforms, game_names, tag_lines): (Vec<_>, Vec<_>, Vec<_>) = player_shorts
        .iter()
        .map(|player_short| {
            (
                PlatformRouteDb::from(player_short.platform),
                player_short.game_name.clone(),
                player_short.tag_line.clone(),
            )
        })
        .multiunzip();

    let query = r#"
        SELECT id, game_name, tag_line, platform
        FROM summoners
        WHERE (game_name, tag_line, platform) IN (
            SELECT UNNEST($1::text[]), UNNEST($2::text[]), UNNEST($3::platform_type[])
        )
    "#;
    let rows = sqlx::query_as::<_, (i32, String, String, PlatformRouteDb)>(query)
        .bind(&game_names)
        .bind(&tag_lines)
        .bind(&platforms)
        .fetch_all(db)
        .await?;

    // Convert rows into a hashmap for quick lookups
    let existing_summoners = rows
        .into_iter()
        .map(|(id, game_name, tag_line, platform)| {
            (
                ProPlayerAccountShort {
                    game_name,
                    tag_line,
                    platform: PlatformRoute::from(platform),
                },
                id,
            )
        })
        .collect::<HashMap<_, _>>();
    Ok(existing_summoners)
}

pub async fn get_all_pro_players() -> AppResult<Vec<String>> {
    let mut pro_player_slugs = Vec::new();
    let per_page = 1000;
    let mut page = 1;
    while let Ok(pro_players) = get_pro_players(page, per_page).await {
        let curr_len = pro_players.len();
        pro_player_slugs.extend(pro_players.into_iter().map(|pro_player| pro_player.slug));
        if curr_len < per_page as usize {
            break;
        } else {
            page += 1;
        }
    }

    Ok(pro_player_slugs)
}

pub async fn get_pro_player_info(slug: &str) -> AppResult<ProPlayerShort> {
    let response: ProProfile = reqwest::get(format!(
        "https://api.lolpros.gg/es/profiles/{}",
        urlencoding::encode(slug)
    ))
    .await?
    .json()
    .await?;
    Ok(ProPlayerShort {
        slug: response.slug,
        pro_uuid: response.uuid,
        accounts: response
            .league_player
            .accounts
            .into_iter()
            .map(|account| ProPlayerAccountShort {
                game_name: account.gamename,
                tag_line: account.tagline,
                platform: PlatformRoute::from(account.server.as_str()),
            })
            .collect(),
    })
}

pub async fn get_pro_players(page: i32, per_page: i32) -> AppResult<Vec<ProPlayer>> {
    match reqwest::get(format!(
        "https://api.lolpros.gg/es/ladder?page={}&page_size={}",
        page, per_page
    ))
    .await?
    .json()
    .await
    {
        Ok(pro_players) => Ok(pro_players),
        Err(e) => Err(AppError::CustomError(format!(
            "Failed to fetch pro players: {:?}",
            e
        ))),
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProPlayerShort {
    pub slug: String,
    pub pro_uuid: String,
    pub accounts: Vec<ProPlayerAccountShort>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProPlayerAccountShort {
    pub game_name: String,
    pub tag_line: String,
    pub platform: PlatformRoute,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProPlayer {
    pub slug: String,
}



#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProProfile {
    pub uuid: String,
    pub slug: String,
    #[serde(rename = "league_player")]
    pub league_player: LeaguePlayer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaguePlayer {
    pub accounts: Vec<ProfileAccount>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAccount {
    pub server: String,
    pub gamename: String,
    pub tagline: String,
}
