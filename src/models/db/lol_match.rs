use crate::models::db::Id;
use crate::models::entities::lol_match::LolMatch;
use crate::version_to_major_minor;
use sqlx::types::chrono;

impl LolMatch {
    pub async fn bulk_default_insert(db: &sqlx::PgPool, match_ids: &[String]) -> Vec<i32> {
        let match_ids = match_ids.iter().map(|x| x.clone()).collect::<Vec<String>>();
        let platforms = match_ids.iter().map(|x| {
            let match_id_split = x.split("_").collect::<Vec<&str>>();
            match_id_split[0].to_string()
        }).collect::<Vec<String>>();
        let sql = r"
        INSERT INTO
            lol_matches
            (match_id, platform)
        SELECT * FROM UNNEST(
            $1::VARCHAR(17)[],
            $2::VARCHAR(4)[]
        )
        ON CONFLICT (match_id) DO NOTHING
        RETURNING id;
        ";
        let rows = sqlx::query_as::<_, Id>(sql)
            .bind(match_ids)
            .bind(platforms)
            .fetch_all(db)
            .await
            .unwrap();
        rows.into_iter().map(|r| r.id).collect()
    }


    pub async fn bulk_update(db: &sqlx::PgPool, matches: &[crate::riven_fix::Match]) -> Vec<i32> {
        let match_ids = matches.iter().map(|x| x.metadata.match_id.clone()).collect::<Vec<String>>();
        let match_creations = matches.iter().map(|x| chrono::DateTime::from_timestamp_millis(x.info.game_start_timestamp).unwrap()).collect::<Vec<_>>();
        let match_ends = matches.iter().map(|x| chrono::DateTime::from_timestamp_millis(x.info.game_end_timestamp.unwrap()).unwrap()).collect::<Vec<_>>();
        let match_durations = matches.iter().map(|x| x.info.game_duration as i32).collect::<Vec<i32>>();
        let queue_ids = matches.iter().map(|x| x.info.queue_id.0 as i32).collect::<Vec<i32>>();
        let map_ids = matches.iter().map(|x| x.info.map_id.0 as i32).collect::<Vec<i32>>();
        let versions = matches.iter().map(|x| version_to_major_minor(x.info.game_version.clone())).collect::<Vec<String>>();
        let modes = matches.iter().map(|x| x.info.game_mode.to_string()).collect::<Vec<String>>();
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
        WHERE lol_matches.match_id = data.match_id
        RETURNING lol_matches.id;
        ";
        let rows = sqlx::query_as::<_, Id>(sql)
            .bind(match_ids)
            .bind(match_creations)
            .bind(match_ends)
            .bind(match_durations)
            .bind(queue_ids)
            .bind(map_ids)
            .bind(modes)
            .bind(versions)
            .fetch_all(db)
            .await
            .unwrap();

        rows.into_iter().map(|r| r.id).collect()
    }



}