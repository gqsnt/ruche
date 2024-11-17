use crate::backend::ssr::AppResult;
use crate::consts::platform_route::PLATFORM_ROUTE_OPTIONS;
use crate::utils::summoner_url;
use chrono::{Duration, Local, NaiveDateTime, Timelike};
use leptos::leptos_dom::log;
use sitemap::structs::UrlEntry;
use sitemap::writer::SiteMapWriter;
use sqlx::PgPool;
use tokio::time::{sleep_until, Instant};

pub async fn schedule_generate_site_map(
    db: PgPool,
) {
    let start_hour = 3;
    tokio::spawn(async move {
        if let Err(e) = generate_site_map(&db).await {
            log!("Failed to update site map: {:?}", e);
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
            if let Err(e) = generate_site_map(&db).await {
                log!("Failed to update pro player data: {:?}", e);
            }
        }
    });
}

pub async fn generate_site_map(db: &PgPool) -> AppResult<()> {
    let mut output = Vec::<u8>::new();
    {
        let writer = SiteMapWriter::new(&mut output);
        let base_url = "https://next-level.xyz";
        let mut url_writer = writer.start_urlset()?;
        url_writer.url(UrlEntry::builder().loc(base_url).build()?)?;
        for platform in PLATFORM_ROUTE_OPTIONS {
            url_writer.url(UrlEntry::builder().loc(format!("{}/platform/{}", base_url, platform)).build()?)?;
        }
        let total_summoners = get_total_summoners(db).await?;
        let per_page = 1000;
        let total_pages = total_summoners / per_page;
        for page in 1..=total_pages {
            let summoners = get_platforms_summoners_taglines(db, per_page, page).await?;
            for (game_name, tag_line, platform, updated_at) in summoners {
                let url = format!("{}{}", base_url, summoner_url(&platform, &game_name, &tag_line));
                url_writer.url(UrlEntry::builder().loc(url).lastmod(updated_at.and_utc().fixed_offset()).build()?)?;
            }
        }
        url_writer.end()?;
    }

    tokio::fs::write("public/sitemap.xml", output).await?;
    Ok(())
}


pub async fn get_total_summoners(db: &PgPool) -> AppResult<i64> {
    let total = sqlx::query_scalar("SELECT COUNT(*) FROM summoners WHERE tag_line != '' and game_name != ''")
        .fetch_one(db)
        .await?;
    Ok(total)
}


pub async fn get_platforms_summoners_taglines(db: &PgPool, per_page: i64, page: i64) -> AppResult<Vec<(String, String, String, NaiveDateTime)>> {
    let offset = (page - 1) * per_page;
    sqlx::query_as::<_, (String, String, String, NaiveDateTime)>(
        "SELECT game_name, tag_line, platform, updated_at FROM summoners WHERE tag_line != '' and game_name != '' ORDER BY platform, game_name  LIMIT $1 OFFSET $2"
    ).bind(per_page)
        .bind(offset)
        .fetch_all(db)
        .await
        .map_err(|e| e.into())
}