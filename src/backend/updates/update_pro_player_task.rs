use crate::backend::ssr::{AppError, AppResult};
use crate::backend::updates::update_matches_task::bulk_summoners::bulk_insert_summoners;
use crate::backend::updates::update_matches_task::TempSummoner;
use crate::{consts, DB_CHUNK_SIZE};
use chrono::{Duration, Local, Timelike, Utc};
use futures::stream::FuturesUnordered;
use futures::{stream, StreamExt};
use itertools::Itertools;
use leptos::logging::log;
use riven::RiotApi;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Uuid;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep_until, Instant};

pub async fn schedule_update_pro_player_task(
    db: PgPool,
    api: Arc<RiotApi>,
) {
    let start_hour = 2;
    tokio::spawn(async move {
        if let Err(e) = update_pro_player_task(&db, api.clone()).await {
            log!("Failed to update pro player data: {:?}", e);
        }
        loop {
            // Calculate the time until the next 2 a.m.
            let now = Local::now();
            let target_time = if now.hour() >= start_hour {
                // If it's past 2 a.m. today, schedule for 2 a.m. the next day
                (now + Duration::days(1)).with_hour(start_hour).unwrap().with_minute(0).unwrap().with_second(0).unwrap()
            } else {
                // Otherwise, schedule for 2 a.m. today
                now.with_hour(start_hour).unwrap().with_minute(0).unwrap().with_second(0).unwrap()
            };

            let duration_until_target = target_time - now;
            let sleep_duration = duration_until_target.to_std().expect("Failed to calculate sleep duration");

            // Wait until the next 2 a.m.
            sleep_until(Instant::now() + sleep_duration).await;

            // Execute the task
            if let Err(e) = update_pro_player_task(&db, api.clone()).await {
                log!("Failed to update pro player data: {:?}", e);
            }
        }
    });
}


pub async fn update_pro_player_task(
    db: &PgPool,
    api: Arc<RiotApi>,
)
    -> AppResult<()> {
    let mut start = Instant::now();
    let pro_players = get_all_pro_players().await?;
    log!("Found {} Pro players", pro_players.len());

    // Define the concurrency limit
    let concurrency_limit = 50;

    // Create a stream of futures with limited concurrency
    let pro_players_data = stream::iter(
        pro_players
            .into_iter()
            .take(50)
            .map(|slug| {
                async move {
                    get_pro_player_info(slug.as_str()).await
                }
            })
    )
        .buffer_unordered(concurrency_limit)
        .filter_map(|response| async {
            Some(response.unwrap())
        })
        .collect::<Vec<_>>()
        .await;
    log!("Time to fetch pro_data: {:?}", start.elapsed());
    start = Instant::now();
    let pro_accounts = pro_players_data.iter().flat_map(|pro_player| {
        pro_player.accounts.iter().cloned()
    }).collect::<Vec<ProPlayerAccountShort>>();

    let mut existing_summoner_ids = fetch_existing_accounts(db, &pro_accounts).await?;
    let not_found_accounts = pro_accounts.iter().filter(|&account| {
        !existing_summoner_ids.keys().contains(account)
    }).collect::<Vec<_>>();
    println!("Not found accounts: {:?}", not_found_accounts.len());


    // dl summoners
    let summoners_futures = not_found_accounts.into_iter().map(|pro_player_account| {
        let api = api.clone();
        let pt = consts::platform_route::PlatformRoute::from(pro_player_account.platform.as_str()).to_riven();
        async move {
            let response = api.account_v1().get_by_riot_id(pt.to_regional(), pro_player_account.game_name.as_str(), pro_player_account.tag_line.as_str()).await;
            match response {
                Ok(Some(account)) => {
                    Ok(TempSummoner {
                        game_name: account.game_name.unwrap_or_default(),
                        tag_line: account.tag_line.unwrap_or_default(),
                        puuid: account.puuid,
                        platform: pro_player_account.platform.clone(),
                        summoner_level: 0,
                        profile_icon_id: 0,
                        updated_at: Utc::now(),
                    })
                }
                _ => {
                    Err(AppError::CustomError(format!("Summoner not found: {:?}", pro_player_account)))
                }
            }
        }
    });
    println!("Fetching summoners");
    let summoners_to_insert: Vec<_> = FuturesUnordered::from_iter(summoners_futures)
        .collect::<Vec<_>>()
        .await.into_iter()
        .filter_map(|result| {
            result.ok()
        }).collect::<Vec<_>>();
    println!("Fetched summoners: {:?}", summoners_to_insert.len());
    for chunk in summoners_to_insert.chunks(DB_CHUNK_SIZE) {
        let inserted_summoners = bulk_insert_summoners(db, chunk).await?;
        inserted_summoners.into_iter().for_each(|(_, (id, platform, game_name, tag_line))| {
            existing_summoner_ids.insert(ProPlayerAccountShort {
                game_name,
                tag_line,
                platform,
            }, id);
        })
    }
    println!("Time taken to fetch summoners: {:?}", start.elapsed());
    start = Instant::now();
    let pro_players_db = mass_upsert_pro_players(db, &pro_players_data).await?;
    remove_pro_players_from_summoners(db).await?;
    mass_update_adding_pro_player_to_summoners(db, pro_players_db, existing_summoner_ids, &pro_players_data).await?;
    println!("Time taken to update pro players: {:?}", start.elapsed());
    Ok(())
}

pub async fn remove_pro_players_from_summoners(db: &PgPool) -> AppResult<()> {
    let query = r#"
        UPDATE summoners
        SET pro_player_id = null
        where pro_player_id != null
    "#;
    sqlx::query(query)
        .execute(db)
        .await?;
    Ok(())
}


pub async fn mass_update_adding_pro_player_to_summoners(
    db: &PgPool,
    pro_players: HashMap<String, i32>,
    summoner_ids: HashMap<ProPlayerAccountShort, i32>,
    pro_players_data: &[ProPlayerShort],
) -> AppResult<()> {
    let pro_player_summoners = pro_players_data.iter().flat_map(|pro_player| {
        let pro_player_id = *pro_players.get(&pro_player.pro_uuid).unwrap();
        pro_player.accounts.iter().filter_map(|account| {
            summoner_ids.get(account).map(|summoner_id| {
                (pro_player_id, *summoner_id)
            })
        }).collect_vec()
    }).collect_vec();

    let (pro_player_ids, summoner_ids): (Vec<_>, Vec<_>) = pro_player_summoners.iter().map(|&(pro_player_id, summoner_id)| {
        (pro_player_id, summoner_id)
    }).multiunzip();

    let query = r#"
        UPDATE summoners
        SET pro_player_id = data.pro_player_id
        FROM (SELECT UNNEST($1::INT[]) AS summoner_id, UNNEST($2::INT[]) AS pro_player_id) AS data
        WHERE summoners.id = data.summoner_id
    "#;
    sqlx::query(query)
        .bind(&summoner_ids)
        .bind(&pro_player_ids)
        .execute(db)
        .await?;
    Ok(())
}


pub async fn mass_upsert_pro_players(
    db: &PgPool,
    pro_players: &[ProPlayerShort],
) -> AppResult<HashMap<String, i32>> {
    let uuids = pro_players.iter().map(|pro_player| Uuid::parse_str(pro_player.pro_uuid.as_str()).unwrap()).collect::<Vec<Uuid>>();
    let slugs = pro_players.iter().map(|pro_player| pro_player.slug.clone()).collect::<Vec<String>>();
    let query = r#"
        INSERT INTO pro_players (pro_uuid, slug)
        SELECT * FROM UNNEST($1::UUID[], $2::VARCHAR(50)[])
        ON CONFLICT (pro_uuid) DO UPDATE SET slug = EXCLUDED.slug
        RETURNING pro_uuid, id
    "#;
    Ok(
        sqlx::query_as::<_, (Uuid, i32)>(query)
            .bind(&uuids)
            .bind(&slugs)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|(uuid, id)| {
                (uuid.to_string(), id)
            })
            .collect()
    )
}

pub async fn fetch_existing_accounts(
    db: &PgPool,
    player_shorts: &[ProPlayerAccountShort],
) -> AppResult<HashMap<ProPlayerAccountShort, i32>> {
    let (platforms, game_names, tag_lines): (Vec<_>, Vec<_>, Vec<_>) = player_shorts.iter().map(|player_short| {
        (player_short.platform.clone(), player_short.game_name.clone(), player_short.tag_line.clone())
    }).multiunzip();

    let query = r#"
        SELECT id, game_name, tag_line, platform
        FROM summoners
        WHERE (game_name, tag_line, platform) IN (
            SELECT UNNEST($1::text[]), UNNEST($2::text[]), UNNEST($3::text[])
        )
    "#;
    let rows = sqlx::query_as::<_, (i32, String, String, String)>(query)
        .bind(&game_names)
        .bind(&tag_lines)
        .bind(&platforms)
        .fetch_all(db)
        .await?;

    // Convert rows into a hashmap for quick lookups
    let existing_summoners = rows
        .into_iter()
        .map(|(id, game_name, tag_line, platform)| {
            (ProPlayerAccountShort {
                game_name,
                tag_line,
                platform,
            }, id)
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
    let response: ProProfile = reqwest::get(
        format!("https://api.lolpros.gg/es/profiles/{}", urlencoding::encode(slug))
    ).await?.json().await?;
    Ok(ProPlayerShort {
        slug: response.slug,
        pro_uuid: response.uuid,
        accounts: response.league_player.accounts.into_iter().map(|account| ProPlayerAccountShort {
            game_name: account.gamename,
            tag_line: account.tagline,
            platform: account.server,
        }).collect(),
    })
}


pub async fn get_pro_players(page: i32, per_page: i32) -> AppResult<Vec<ProPlayer>> {
    match reqwest::get(format!("https://api.lolpros.gg/es/ladder?page={}&page_size={}", page, per_page)).await?.json().await {
        Ok(pro_players) => Ok(pro_players),
        Err(e) => Err(AppError::CustomError(format!("Failed to fetch pro players: {:?}", e)))
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
    pub platform: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProPlayer {
    pub slug: String,
    pub country: String,
    #[serde(rename = "other_countries")]
    pub other_countries: Vec<String>,
    pub position: String,
    #[serde(rename = "total_games")]
    pub total_games: i64,
    pub score: i64,
    pub team: Option<Team>,
    pub account: Account,
    pub leagues: Vec<League>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub tag: String,
    pub logo: Logo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logo {
    #[serde(rename = "public_id")]
    pub public_id: String,
    pub version: i64,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub uuid: String,
    pub server: String,
    #[serde(rename = "profile_icon_id")]
    pub profile_icon_id: String,
    #[serde(rename = "summoner_name")]
    pub summoner_name: String,
    pub tier: String,
    pub division: i64,
    #[serde(rename = "league_points")]
    pub league_points: i64,
    pub games: i64,
    pub winrate: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct League {
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub shorthand: String,
    pub servers: Vec<String>,
    pub logo: Logo,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProProfile {
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub country: String,
    #[serde(rename = "other_countries")]
    pub other_countries: Vec<String>,
    #[serde(rename = "league_player")]
    pub league_player: LeaguePlayer,
    pub staff: Value,
    //#[serde(rename = "social_media")]
    //pub social_media: SocialMedia,
    pub leagues: Vec<League>,
    //pub rankings: Rankings,
    //#[serde(rename = "previous_teams")]
    //pub previous_teams: Vec<PreviousTeam>,
    pub teams: Vec<Team>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaguePlayer {
    pub position: String,
    pub score: i64,
    pub accounts: Vec<ProfileAccount>,
    pub servers: Vec<String>,
    #[serde(rename = "in_game")]
    pub in_game: bool,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAccount {
    pub uuid: String,
    pub server: String,
    #[serde(rename = "encrypted_puuid")]
    pub encrypted_puuid: String,
    #[serde(rename = "summoner_name")]
    pub summoner_name: String,
    pub gamename: String,
    pub tagline: String,
    //#[serde(rename = "summoner_names")]
    //pub summoner_names: Vec<SummonerName>,
}
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct SummonerName {
//     pub name: String,
//     #[serde(rename = "created_at")]
//     pub created_at: String,
// }
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Rank {
//     pub score: i64,
//     pub tier: String,
//     pub division: i64,
//     #[serde(rename = "league_points")]
//     pub league_points: i64,
//     pub wins: i64,
//     pub losses: i64,
//     #[serde(rename = "created_at")]
//     pub created_at: String,
// }
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Peak {
//     pub score: i64,
//     pub tier: String,
//     pub division: i64,
//     #[serde(rename = "league_points")]
//     pub league_points: i64,
//     pub wins: i64,
//     pub losses: i64,
//     #[serde(rename = "created_at")]
//     pub created_at: String,
// }
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Season {
//     pub id: String,
//     pub end: End,
//     pub peak: Peak,
// }
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct End {
//     pub score: i64,
//     pub tier: String,
//     pub division: i64,
//     #[serde(rename = "league_points")]
//     pub league_points: i64,
//     pub wins: i64,
//     pub losses: i64,
//     #[serde(rename = "created_at")]
//     pub created_at: String,
// }
//
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct SocialMedia {
//     pub discord: Value,
//     pub facebook: Value,
//     pub instagram: Value,
//     pub gamesoflegends: Value,
//     pub leaguepedia: String,
//     pub twitch: String,
//     pub twitter: String,
// }
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Rankings {
//     pub global: i64,
//     pub country: i64,
//     pub position: i64,
//     #[serde(rename = "country_position")]
//     pub country_position: i64,
// }
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct PreviousTeam {
//     pub uuid: String,
//     pub tag: String,
//     pub name: String,
//     pub slug: String,
//     pub logo: Logo,
//     pub server: String,
//     #[serde(rename = "join_date")]
//     pub join_date: String,
//     #[serde(rename = "leave_date")]
//     pub leave_date: String,
//     pub role: String,
//     pub members: Vec<Member>,
// }
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Member {
//     pub uuid: String,
//     pub profile: String,
//     pub name: String,
//     pub slug: String,
//     pub current: bool,
//     pub country: String,
//     pub role: String,
//     pub position: String,
//     #[serde(rename = "join_date")]
//     pub join_date: String,
//     #[serde(rename = "join_timestamp")]
//     pub join_timestamp: i64,
//     #[serde(rename = "leave_date")]
//     pub leave_date: Option<String>,
//     #[serde(rename = "leave_timestamp")]
//     pub leave_timestamp: Option<i64>,
// }
//
// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct CurrentMember {
//     pub uuid: String,
//     pub profile: String,
//     pub name: String,
//     pub slug: String,
//     pub current: bool,
//     pub country: String,
//     pub role: String,
//     pub position: String,
//     #[serde(rename = "join_date")]
//     pub join_date: String,
//     #[serde(rename = "join_timestamp")]
//     pub join_timestamp: i64,
//     #[serde(rename = "leave_date")]
//     pub leave_date: Value,
//     #[serde(rename = "leave_timestamp")]
//     pub leave_timestamp: Value,
//     #[serde(rename = "profile_icon_id")]
//     pub profile_icon_id: Option<String>,
//     #[serde(rename = "summoner_name")]
//     pub summoner_name: Option<String>,
//     pub tier: Option<String>,
//     pub division: Option<i64>,
//     #[serde(rename = "league_points")]
//     pub league_points: Option<i64>,
//     pub score: Option<i64>,
// }
