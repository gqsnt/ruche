use crate::backend::ssr::AppResult;
use crate::backend::tasks::update_matches::LolMatchNotUpdated;
use crate::utils::version_to_major_minor;
use itertools::Itertools;
use leptos::logging::log;
use riven::models::match_v5::Match;
use riven::RiotApiError;
use sqlx::PgPool;

pub async fn bulk_trashed_matches(
    db: &PgPool,
    matches: Vec<(Result<Option<Match>,RiotApiError>, LolMatchNotUpdated)>,
) -> AppResult<()> {
    let match_ids = matches
        .iter()
        .map(|(_, db_match)| db_match.id)
        .collect::<Vec<i32>>();
    let sql = r"
        UPDATE lol_matches
        SET
            trashed = true,
            updated = true
        WHERE id = ANY($1)
        RETURNING id;
        ";
    sqlx::query(sql).bind(match_ids).fetch_all(db).await?;
    Ok(())
}

pub async fn bulk_update_matches(
    db: &PgPool,
    matches: Vec<(Match, LolMatchNotUpdated)>,
) -> AppResult<()> {
    let (
        match_ids,
        match_creations,
        match_ends,
        match_durations,
        queue_ids,
        map_ids,
        versions,
        modes,
    ): (
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
        Vec<_>,
    ) = matches
        .iter()
        .map(|(x, _)| {
            // log!(
            //     "Match: Queue: {:?}, Map: {:?}, Version: {:?}, Mode: {:?}",
            //     x.info.queue_id.0,
            //     x.info.map_id.0,
            //     x.info.game_version,
            //     x.info.game_mode
            // );
            (
                x.metadata.match_id.as_str(),
                chrono::DateTime::from_timestamp_millis(x.info.game_start_timestamp)
                    .unwrap_or_default(),
                chrono::DateTime::from_timestamp_millis(
                    x.info.game_end_timestamp.unwrap_or_default(),
                )
                .unwrap_or_default(),
                x.info.game_duration as i32,
                x.info.queue_id.0 as i32,
                x.info.map_id.0 as i32,
                version_to_major_minor(x.info.game_version.as_str()),
                x.info.game_mode.to_string(),
            )
        })
        .multiunzip();

    let sql = r"
        UPDATE lol_matches
        SET
            match_creation = data.match_creation,
            match_end = data.match_end,
            match_duration = data.match_duration,
            queue_id = data.queue_id,
            map_id = data.map_id,
            game_mode = data.game_mode,
            version = data.version,
            updated = true
        FROM (
            SELECT
                UNNEST($1::VARCHAR(17)[]) AS match_id,
                UNNEST($2::TIMESTAMP[]) AS match_creation,
                UNNEST($3::TIMESTAMP[]) AS match_end,
                UNNEST($4::INT[]) AS match_duration,
                UNNEST($5::INT[]) AS queue_id,
                UNNEST($6::INT[]) AS map_id,
                UNNEST($7::VARCHAR(15)[]) AS game_mode,
                UNNEST($8::VARCHAR(5)[]) AS version
        ) AS data
        WHERE lol_matches.match_id = data.match_id;
        ";
    sqlx::query(sql)
        .bind(match_ids)
        .bind(match_creations)
        .bind(match_ends)
        .bind(match_durations)
        .bind(queue_ids)
        .bind(map_ids)
        .bind(modes)
        .bind(versions)
        .fetch_all(db)
        .await?;

    Ok(())
}
