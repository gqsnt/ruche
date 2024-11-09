use crate::apis::MatchFiltersSearch;
use crate::consts::PlatformRoute;
use crate::error_template::{AppError, AppResult};
use crate::models::db::db_model::{IdPuuidUpdatedAt, LolSummonerEncounter, SummonerDb, SummonerLiveGameDb};
use crate::models::db::{parse_date, Id, DATE_FORMAT};
use crate::models::entities::summoner::{LolSummonerEncounterPage, LolSummonerEncounterPageResult, Summoner};
use crate::models::update::summoner_matches::TempSummoner;
use riven::RiotApi;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{PgPool, QueryBuilder, Row};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use itertools::Itertools;

impl Summoner {
    pub async fn get_encounters(
        db: &PgPool,
        summoner_id: i32,
        page: i32,
        filters: MatchFiltersSearch,
        search_summoner: Option<String>,
    ) -> AppResult<LolSummonerEncounterPageResult> {
        let start_date = parse_date(filters.start_date.clone());
        let end_date = parse_date(filters.end_date.clone());
        let per_page = 40;
        let offset = (page.max(1) - 1) * per_page;

        // initial query
        let mut query = QueryBuilder::new(
            r#"
                SELECT
                    ss.id,
                    ss.tag_line,
                    ss.game_name,
                    ss.platform,
                    ss.profile_icon_id,
                    encounter_data.match_count as encounter_count,
                    total_count
                FROM
                    summoners AS ss
                        INNER JOIN (
                        SELECT
                            lmp.summoner_id,
                            COUNT(lmp.id) AS match_count,
                            COUNT(lmp.summoner_id) OVER() AS total_count
                        FROM
                            lol_match_participants AS lmp
                        WHERE
                lmp.summoner_id !=
        "#);
        query.push_bind(summoner_id);

        // add inner requests and filters
        query.push(
            r#"
                        AND EXISTS (
                            SELECT 1
                            FROM lol_match_participants AS lmp1
                                INNER JOIN (select id, queue_id, match_end from lol_matches) as lm on lmp1.lol_match_id = lm.id

        "#);


        // conditional join for search_summoner
        if search_summoner.is_some() && !search_summoner.as_ref().unwrap().is_empty(){
            query.push("inner join (select id, game_name from summoners) as s1 on lmp.summoner_id = s1.id");
        }

        query.push(
            r#"
                            WHERE lmp1.lol_match_id = lmp.lol_match_id
                              AND lmp1.summoner_id =
        "#);
        query.push_bind(summoner_id);
        if let Some(search_summoner) = search_summoner {
            if !search_summoner.is_empty(){
                query.push(" AND s1.game_name ILIKE ");
                query.push_bind(format!("%{}%", search_summoner));
            }
        }
        if let Some(champion_id) = filters.champion_id {
            query.push(" AND lmp1.champion_id = ");
            query.push_bind(champion_id);
        }
        if let Some(queue_id) = filters.queue_id {
            query.push(" AND lm.queue_id = ");
            query.push_bind(queue_id);
        }
        if let Some(start_date) = start_date {
            query.push(" AND lm.match_end >= ");
            query.push_bind(start_date);
        }
        if let Some(end_date) = end_date {
            query.push(" AND lm.match_end <= ");
            query.push_bind(end_date);
        }

        query.push(
            r#"
                            )
                        GROUP BY
                            lmp.summoner_id
                        ORDER BY
                            match_count DESC
                        LIMIT
        "#);
        query.push_bind(per_page);
        query.push(" OFFSET ");
        query.push_bind(offset);
        query.push(" ) AS encounter_data ON ss.id = encounter_data.summoner_id ORDER BY encounter_count DESC");

        let results = query.build_query_as::<LolSummonerEncounter>().fetch_all(db).await.unwrap();
        let total_pages = if results.is_empty(){
            0
        } else {
            (results.get(0).unwrap().total_count as f64 / per_page as f64).ceil() as i64
        };
        Ok(LolSummonerEncounterPageResult {
            total_pages,
            encounters: results.into_iter().map(|encounter| {
                LolSummonerEncounterPage {
                    id: encounter.id,
                    profile_icon_id: encounter.profile_icon_id,
                    count: encounter.encounter_count,
                    game_name: encounter.game_name,
                    tag_line: encounter.tag_line,
                    platform: encounter.platform,
                }
            }).collect::<Vec<_>>()
        })
    }


    pub async fn find_by_exact_details(
        db: &sqlx::PgPool,
        platform_route: &PlatformRoute,
        game_name: &str,
        tag_line: &str,
    ) -> AppResult<Summoner> {
        sqlx::query_as::<_, SummonerDb>(
            "SELECT * FROM summoners WHERE game_name = $1 AND tag_line = $2 AND platform = $3 LIMIT 1"
        ).bind(game_name)
            .bind(tag_line)
            .bind(platform_route.as_region_str())
            .fetch_one(db)
            .await
            .map(|x| x.map_to_summoner())
            .map_err(AppError::from)
    }


    pub async fn find_by_details(
        db: &sqlx::PgPool,
        platform_route: &PlatformRoute,
        game_name: &str,
        tag_line: &str,
    ) -> AppResult<Summoner> {
        sqlx::query_as::<_, SummonerDb>(
            "SELECT * FROM summoners WHERE game_name ILIKE $1 AND tag_line ILIKE $2 AND platform = $3"
        )
            .bind(game_name)
            .bind(tag_line)
            .bind(platform_route.as_region_str())

            .fetch_one(db)
            .await
            .map(|x| x.map_to_summoner())
            .map_err(AppError::from)
    }

    pub async fn fetch_existing_summoners(
        db: &sqlx::PgPool,
        puuids: &[String],
    ) -> AppResult<HashMap<String, (i32, i32)>> {
        Ok(sqlx::query_as::<_, IdPuuidUpdatedAt>("
            SELECT id, puuid, updated_at
            FROM summoners
            WHERE puuid = ANY($1)
        ")
            .bind(puuids)

            .fetch_all(db)
            .await?
            .into_iter()
            .map(|row| {
                (row.puuid, (row.id, row.updated_at.unwrap().and_utc().timestamp() as i32))
            })
            .collect::<HashMap<String, (i32, i32)>>())
    }


    pub async fn bulk_update(db: &sqlx::PgPool, summoners: &[TempSummoner]) -> AppResult<()> {
        let game_names = summoners.iter().map(|x| x.game_name.clone()).collect::<Vec<String>>();
        let tag_lines = summoners.iter().map(|x| x.tag_line.clone()).collect::<Vec<String>>();
        let puuids = summoners.iter().map(|x| x.puuid.clone()).collect::<Vec<String>>();
        let platforms = summoners.iter().map(|x| x.platform.to_string()).collect::<Vec<String>>();
        let summoner_levels = summoners.iter().map(|x| x.summoner_level).collect::<Vec<i64>>();
        let profile_icon_ids = summoners.iter().map(|x| x.profile_icon_id).collect::<Vec<i32>>();
        let updated_ats = summoners.iter().map(|x| x.updated_at).collect::<Vec<DateTime<Utc>>>();

        let sql = r"
        UPDATE summoners
        SET
            game_name = data.game_name,
            tag_line = data.tag_line,
            platform = data.platform,
            summoner_level = data.summoner_level,
            profile_icon_id = data.profile_icon_id,
            updated_at = data.updated_at
        FROM (
            SELECT
                unnest($1::VARCHAR(16)[]) AS game_name,
                unnest($2::VARCHAR(5)[]) AS tag_line,
                unnest($3::VARCHAR(78)[]) AS puuid,
                unnest($4::VARCHAR(4)[]) AS platform,
                unnest($5::INT[]) AS summoner_level,
                unnest($6::INT[]) AS profile_icon_id,
                unnest($7::TIMESTAMP[]) AS updated_at
        ) AS data
        WHERE summoners.puuid = data.puuid;
        ";

        sqlx::query(sql)
            .bind(game_names)
            .bind(tag_lines)
            .bind(puuids)
            .bind(platforms)
            .bind(summoner_levels)
            .bind(profile_icon_ids)
            .bind(updated_ats)
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn bulk_insert(db: &sqlx::PgPool, summoners: &[TempSummoner]) -> AppResult<HashMap<String, i32>> {
        let game_names = summoners.iter().map(|x| x.game_name.clone()).collect::<Vec<String>>();
        let tag_lines = summoners.iter().map(|x| x.tag_line.clone()).collect::<Vec<String>>();
        let puuids = summoners.iter().map(|x| x.puuid.clone()).collect::<Vec<String>>();
        let platforms = summoners.iter().map(|x| x.platform.to_string()).collect::<Vec<String>>();
        let summoner_levels = summoners.iter().map(|x| x.summoner_level).collect::<Vec<i64>>();
        let profile_icon_ids = summoners.iter().map(|x| x.profile_icon_id).collect::<Vec<i32>>();
        let updated_ats = summoners.iter().map(|x| x.updated_at).collect::<Vec<DateTime<Utc>>>();
        let sql = r"
        INSERT INTO
            summoners
            (
                game_name,
                tag_line,
                puuid,
                platform,
                summoner_level,
                profile_icon_id,
                updated_at
            ) SELECT * FROM UNNEST (
                $1::VARCHAR(16)[],
                $2::VARCHAR(5)[],
                $3::VARCHAR(78)[],
                $4::VARCHAR(4)[],
                $5::INT[],
                $6::INT[],
                $7::TIMESTAMP[]
            )
            ON CONFLICT (puuid)
            DO UPDATE SET
                game_name = EXCLUDED.game_name,
                tag_line = EXCLUDED.tag_line,
                platform = EXCLUDED.platform,
                summoner_level = EXCLUDED.summoner_level,
                profile_icon_id = EXCLUDED.profile_icon_id,
                updated_at = EXCLUDED.updated_at
            WHERE summoners.updated_at < EXCLUDED.updated_at
            returning id;
        ";
        let rows = sqlx::query_as::<_, Id>(sql)
            .bind(game_names)
            .bind(tag_lines)
            .bind(puuids)
            .bind(platforms)
            .bind(summoner_levels)
            .bind(profile_icon_ids)
            .bind(updated_ats)
            .fetch_all(db)
            .await?;
        Ok(rows.into_iter().enumerate().map(|(index, r)| (summoners.get(index).unwrap().puuid.clone(), r.id)).collect::<HashMap<String, i32>>())
    }

    pub async fn find_by_id(db: &sqlx::PgPool, id: i32) -> AppResult<Summoner> {
        sqlx::query_as::<_, SummonerDb>("SELECT * FROM summoners WHERE id = $1")
            .bind(id)
            .fetch_one(db)
            .await
            .map(|x| x.map_to_summoner())
            .map_err(AppError::from)
    }

    pub async fn update_summoner_account_by_id(
        db: &sqlx::PgPool,
        id: i32,
        account: riven::models::account_v1::Account,
    ) -> AppResult<()> {
        sqlx::query(
            "UPDATE summoners SET game_name = $1, tag_line = $2 , updated_at = NOW() WHERE id = $3"
        )
            .bind(account.game_name.unwrap_or_default())
            .bind(account.tag_line.unwrap_or_default())
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }


    pub async fn update_summoner_by_id(
        db: &sqlx::PgPool,
        id: i32,
        platform_route: PlatformRoute,
        account: riven::models::account_v1::Account,
        summoner: riven::models::summoner_v4::Summoner,
    ) -> AppResult<Summoner> {
        sqlx::query(
            "UPDATE summoners SET game_name = $1, tag_line = $2, puuid = $3, summoner_level = $4, profile_icon_id = $5, platform = $6, updated_at = NOW() WHERE id = $8"
        )
            .bind(account.game_name.clone())
            .bind(account.tag_line.clone())
            .bind(summoner.puuid.clone())
            .bind(summoner.summoner_level as i32)
            .bind(summoner.profile_icon_id)
            .bind(platform_route.as_region_str())
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(db)
            .await?;
        Ok(
            Summoner {
                id,
                game_name: account.game_name.unwrap_or_default(),
                tag_line: account.tag_line.unwrap_or_default(),
                puuid: summoner.puuid,
                platform: platform_route.into(),
                updated_at: Utc::now().format(DATE_FORMAT).to_string(),
                summoner_level: summoner.summoner_level,
                profile_icon_id: summoner.profile_icon_id,
            }
        )
    }

    pub async fn insert_summoner(
        db: &sqlx::PgPool,
        platform_route: PlatformRoute,
        account: riven::models::account_v1::Account,
        summoner: riven::models::summoner_v4::Summoner,
    ) -> AppResult<Summoner> {
        let rec = sqlx::query_as::<_, Id>(
            "INSERT INTO summoners(game_name, tag_line, puuid, platform, summoner_level, profile_icon_id, updated_at) VALUES ($1, $2, $3, $4, $5, $6, NOW()) RETURNING id"
        )
            .bind(account.game_name.clone())
            .bind(account.tag_line.clone())
            .bind(summoner.puuid.clone())
            .bind(platform_route.as_region_str())
            .bind(summoner.summoner_level as i32)
            .bind(summoner.profile_icon_id)
            .bind(Utc::now().naive_utc())
            .fetch_one(db)
            .await?;
        Ok(Summoner {
            id: rec.id,
            game_name: account.game_name.unwrap_or_default(),
            tag_line: account.tag_line.unwrap_or_default(),
            puuid: summoner.puuid,
            platform: platform_route.into(),
            updated_at: Utc::now().format(DATE_FORMAT).to_string(),
            summoner_level: summoner.summoner_level,
            profile_icon_id: summoner.profile_icon_id,
        })
    }


    pub async fn find_summoner_ids_by_puuids(db: &sqlx::PgPool, platform_route: PlatformRoute, puuids: &[String]) -> AppResult<HashMap<String, i32>> {
        Ok(sqlx::query_as::<_, IdPuuidUpdatedAt>(
            "SELECT id, puuid FROM summoners WHERE puuid = ANY($1) and platform = $2"
        )
            .bind(puuids)
            .bind(platform_route.as_region_str())

            .fetch_all(db)
            .await?
            .into_iter()
            .map(|x| (x.puuid, x.id))
            .collect::<HashMap<String, i32>>())
    }

    pub async fn find_summoner_live_details_by_puuids(db: &sqlx::PgPool, puuids: &[String]) -> AppResult<HashMap<String, SummonerLiveGameDb>> {
        Ok(sqlx::query_as::<_, SummonerLiveGameDb>(
            "SELECT  id,
                    puuid,
                    game_name,
                    tag_line,
                    platform,
                    summoner_level  FROM summoners WHERE puuid = ANY($1)"
        )
            .bind(puuids)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|x| (x.puuid.clone(), x))
            .collect::<HashMap<String, SummonerLiveGameDb>>())
    }

    pub async fn find_summoner_id_by_puuid(db: &sqlx::PgPool, platform_route: PlatformRoute, puuid: &str) -> AppResult<i32> {
        sqlx::query_as::<_, Id>("SELECT id FROM summoners WHERE puuid = $1 and platform = $2")
            .bind(puuid)
            .bind(platform_route.as_region_str())
            .fetch_one(db)
            .await
            .map(|x| x.id)
            .map_err(AppError::from)
    }

    pub async fn insert_or_update_account_and_summoner(
        db: &sqlx::PgPool,
        platform_route: PlatformRoute,
        account: riven::models::account_v1::Account,
        summoner: riven::models::summoner_v4::Summoner,
    ) -> AppResult<Summoner> {
        match Summoner::find_summoner_id_by_puuid(db, platform_route, &summoner.puuid).await {
            Ok(id) => {
                Summoner::update_summoner_by_id(db, id, platform_route, account, summoner).await
            }
            Err(_) => {
                Summoner::insert_summoner(db, platform_route, account, summoner).await
            }
        }
    }

    pub async fn find_conflicting_summoners(
        db: &sqlx::PgPool,
    ) -> AppResult<Vec<(String, String, String, Vec<IdPuuidUpdatedAt>)>> {
        Ok(sqlx::query_as::<_, SummonerDb>(
            "SELECT *
        FROM summoners
        WHERE (game_name, tag_line, platform) IN (
            SELECT game_name, tag_line, platform
            FROM summoners
            GROUP BY game_name, tag_line, platform
            HAVING COUNT(*) > 1
        )
        ORDER BY game_name, tag_line, platform, updated_at DESC"
        )
            .fetch_all(db)
            .await?
            .into_iter()
            .fold(HashMap::new(), |mut acc, row| {
                acc.entry((row.game_name.clone(), row.tag_line.clone(), row.platform.clone()))
                    .or_insert_with(Vec::new)
                    .push(IdPuuidUpdatedAt {
                        id: row.id,
                        puuid: row.puuid,
                        updated_at: Some(row.updated_at),
                    });
                acc
            })
            .into_iter()
            .map(|((game_name, tag_line, platform), ids)| (game_name, tag_line, platform, ids))
            .collect())
    }


    pub async fn resolve_conflicts(
        db: &sqlx::PgPool,
        api: &Arc<RiotApi>,
    ) -> AppResult<()> {
        let conflicts = Summoner::find_conflicting_summoners(db).await?;
        for (game_name, tag_line, platform, conflict_records) in conflicts {
            println!("Resolving conflict for {}#{} on {} with {:?}", game_name, tag_line, platform, conflict_records);
            for record in conflict_records {
                // Obtenir les informations actuelles pour chaque `puuid`
                let platform_route = PlatformRoute::from_str(&platform).unwrap();
                let riven_ptr = riven::consts::PlatformRoute::from_str(&platform_route.to_string()).unwrap();
                if let Ok(account) = api.account_v1().get_by_puuid(riven_ptr.to_regional(), &record.puuid).await {
                    Summoner::update_summoner_account_by_id(
                        db,
                        record.id,
                        account,
                    )
                        .await?;
                }
            }
        }

        Ok(())
    }
}
