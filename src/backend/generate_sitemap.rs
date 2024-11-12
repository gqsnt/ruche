use crate::backend::ssr::AppResult;
use crate::consts::platform_route::PLATFORM_ROUTE_OPTIONS;
use crate::utils::summoner_url;
use chrono::NaiveDateTime;
use sitemap::structs::UrlEntry;
use sitemap::writer::SiteMapWriter;
use sqlx::PgPool;

pub async fn generate_site_map(db: &PgPool) -> AppResult<()> {
    let mut output = Vec::<u8>::new();
    {
        let writer = SiteMapWriter::new(&mut output);
        let base_url = "https://next-level.xyz";
        let mut url_writer = writer.start_urlset()?;
        url_writer.url(UrlEntry::builder().loc(base_url).build()?)?;
        for platform in PLATFORM_ROUTE_OPTIONS {
            url_writer.url(UrlEntry::builder().loc(format!("{}/platform/{}", base_url, platform.to_string())).build()?)?;
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