use chrono::NaiveDateTime;
use sitemap::writer::SiteMapWriter;
use sitemap::structs::UrlEntry;
use sqlx::PgPool;
use strum::IntoEnumIterator;
use crate::consts::PlatformRoute;
use crate::error_template::AppResult;
use crate::summoner_url;

pub async fn generate_site_map(db:&PgPool) ->AppResult<()>{
    let mut output = Vec::<u8>::new();
    {
        let mut writer = SiteMapWriter::new(&mut output);
        let base_url = "https://next-level.xyz";
        let mut url_writer = writer.start_urlset().unwrap();
        url_writer.url(UrlEntry::builder().loc(base_url).build().unwrap()).unwrap();
        for platform in  PlatformRoute::iter(){
            if platform == PlatformRoute::RU{
                continue;
            }
            let platform_str = platform.as_region_str();
            url_writer.url(UrlEntry::builder().loc(format!("{}/platform/{}",base_url,platform_str)).build().unwrap()).unwrap();
        }
        let total_summoners = get_total_summoners(db).await?;
        let per_page = 1000;
        let total_pages = total_summoners / per_page;
        for page in 1..=total_pages{
            let summoners = get_platforms_summoners_taglines(db,per_page,page).await?;
            for (game_name, tag_line, platform, updated_at) in summoners{
                let url = format!("{}{}",base_url,summoner_url(&platform, &game_name, &tag_line));
                url_writer.url(UrlEntry::builder().loc(url).lastmod(updated_at.and_utc().fixed_offset()).build().unwrap()).unwrap();
            }
        }
        url_writer.end().unwrap();
    }

    tokio::fs::write("public/sitemap.xml", output).await.unwrap();
    Ok(())
}




pub async fn get_total_summoners(db: &PgPool) -> AppResult<i64> {
    let total = sqlx::query_scalar("SELECT COUNT(*) FROM summoners WHERE tag_line != '' and game_name != ''")
        .fetch_one(db)
        .await?;
    Ok(total)
}


pub async fn get_platforms_summoners_taglines(db: &PgPool, per_page:i64, page:i64) -> AppResult<Vec<(String, String,String, NaiveDateTime)>> {
    let offset = (page - 1) * per_page;
     sqlx::query_as::<_, (String, String, String, NaiveDateTime)>(
        "SELECT game_name, tag_line, platform, updated_at FROM summoners WHERE tag_line != '' and game_name != '' ORDER BY platform, game_name  LIMIT $1 OFFSET $2"
    ).bind(per_page)
        .bind(offset)
        .fetch_all(db)
        .await
        .map_err(|e| e.into())
}