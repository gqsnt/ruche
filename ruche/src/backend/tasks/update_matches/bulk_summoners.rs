use crate::backend::ssr::{AppResult, Id, PlatformRouteDb};
use crate::backend::tasks::update_matches::{SummonerFull, TempSummoner};
use chrono::{DateTime, Utc};
use common::consts::platform_route::PlatformRoute;
use itertools::Itertools;
use std::collections::HashMap;

pub async fn bulk_update_summoners(db: &sqlx::PgPool, summoners: &[TempSummoner]) -> AppResult<()> {
    let (game_names, tag_lines, puuids, platforms, summoner_levels, profile_icon_ids, updated_ats) =
        summoners_multiunzip(summoners);

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
                unnest($3::platform_type[]) AS platform,
                unnest($4::INT[]) AS summoner_level,
                unnest($5::INT[]) AS profile_icon_id,
                unnest($6::TIMESTAMP[]) AS updated_at,
                unnest($6::VARCHAR(78)[]) AS puuid

        ) AS data
        WHERE summoners.puuid = data.puuid;
        ";

    sqlx::query(sql)
        .bind(game_names)
        .bind(tag_lines)
        .bind(platforms)
        .bind(summoner_levels)
        .bind(profile_icon_ids)
        .bind(updated_ats)
        .bind(puuids)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn bulk_insert_summoners(
    db: &sqlx::PgPool,
    summoners: &[TempSummoner],
) -> AppResult<HashMap<String, SummonerFull>> {
    let (game_names, tag_lines, puuids, platforms, summoner_levels, profile_icon_ids, updated_ats) =
        summoners_multiunzip(summoners);
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
                $4::platform_type[],
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
    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(index, r)| {
            let summoner_index = summoners.get(index).unwrap();
            (
                summoner_index.puuid.clone(),
                SummonerFull {
                    id: r.id,
                    game_name: summoner_index.game_name.clone(),
                    tag_line: summoner_index.tag_line.clone(),
                    puuid: summoner_index.puuid.clone(),
                    platform: PlatformRoute::from(summoner_index.platform.as_str()),
                    summoner_level: summoner_index.summoner_level,
                    profile_icon_id: summoner_index.profile_icon_id,
                    pro_player_slug: None,
                },
            )
        })
        .collect::<HashMap<String, SummonerFull>>())
}

#[allow(clippy::type_complexity)]
pub fn summoners_multiunzip(
    summoners: &[TempSummoner],
) -> (
    Vec<&str>,
    Vec<&str>,
    Vec<&str>,
    Vec<PlatformRouteDb>,
    Vec<i32>,
    Vec<i32>,
    Vec<DateTime<Utc>>,
) {
    summoners
        .iter()
        .map(|s| {
            (
                s.game_name.trim(),
                s.tag_line.trim(),
                s.puuid.as_str(),
                PlatformRouteDb::from_raw_str(s.platform.as_str()),
                s.summoner_level,
                s.profile_icon_id as i32,
                s.updated_at,
            )
        })
        .multiunzip()
}
