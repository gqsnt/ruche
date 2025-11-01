use crate::backend::ssr::{AppResult, PlatformRouteDb};
use crate::backend::task_director::Task;
use crate::backend::tasks::calculate_next_run_to_fixed_start_hour;
use crate::utils::summoner_url;
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use common::consts::platform_route::PlatformRoute;
use leptos::leptos_dom::log;
use sitemap::structs::{SiteMapEntry, UrlEntry};
use sitemap::writer::{SiteMapIndexWriter, SiteMapWriter};
use sqlx::PgPool;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::time::Instant;

pub struct GenerateSiteMapTask {
    db: PgPool,
    start_hour: u32,
    next_run: Instant,
    running: Arc<AtomicBool>,
}

impl GenerateSiteMapTask {
    pub fn new(db: PgPool, start_hour: u32, on_startup: bool) -> Self {
        let next_run = if on_startup {
            Instant::now()
        } else {
            calculate_next_run_to_fixed_start_hour(start_hour)
        };
        Self {
            db,
            start_hour,
            next_run,
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Task for GenerateSiteMapTask {
    fn execute(&self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        let db = self.db.clone();
        Box::pin(async move {
            if let Err(e) = generate_site_map(&db).await {
                log!("Failed to generate ruche-lol map: {:?}", e);
            } else {
                log!("Site map generated successfully");
            }
        })
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
            start_hour: self.start_hour,
            next_run: self.next_run,
            running: self.running.clone(),
        })
    }

    fn name(&self) -> &'static str {
        "GenerateSiteMapTask"
    }

    fn allow_concurrent(&self) -> bool {
        false // Do not allow concurrent executions
    }
}

pub async fn generate_site_map_index(index: usize, urls: &[UrlEntry]) -> AppResult<()> {
    let mut output = Vec::<u8>::new();
    {
        let writer = SiteMapWriter::new(&mut output);
        let mut url_writer = writer.start_urlset()?;
        for url in urls {
            url_writer.url(url.clone())?;
        }
        url_writer.end()?;
    }
    let dest_path = PathBuf::from("target")
        .join("site")
        .join(format!("sitemap-index{}.xml", index));
    // let output = flate2::write::GzEncoder::new(output, flate2::Compression::default());
    // let output = output.finish()?;
    tokio::fs::write(dest_path, output).await?;
    Ok(())
}

pub async fn generate_site_map(db: &PgPool) -> AppResult<()> {
    let base_url = "https://ruche.lol";
    let mut urls = vec![get_site_map_url(base_url.to_string(), None)];
    let total_summoners = get_total_summoners(db).await?;
    let chunk_size = 1000;
    let total_chunks = (total_summoners + 1) / chunk_size;
    let mut output = Vec::<u8>::new();
    let now = chrono::Utc::now().fixed_offset();
    let mut site_map_index = 0;
    let writer = SiteMapWriter::new(&mut output);
    let mut url_writer = writer.start_sitemapindex()?;
    for page in 1..=total_chunks {
        let summoners = get_platforms_summoners_taglines(db, chunk_size, page).await?;
        for (game_name, tag_line, platform, updated_at) in summoners {
            let pt: PlatformRoute = platform.into();
            urls.push(get_site_map_url(
                format!(
                    "{}{}",
                    base_url,
                    summoner_url(pt.code(), game_name.as_str(), tag_line.as_str())
                ),
                Some(updated_at.and_utc().fixed_offset()),
            ));
            if urls.len() >=10_000{
                let _ = write_url_to_file(site_map_index, &mut urls,&mut url_writer, now, base_url).await;
                site_map_index +=1;
                urls.clear();
            }
        }
    }
    if urls.len() > 0 {
        let _ = write_url_to_file(site_map_index, &mut urls,&mut url_writer, now, base_url).await;
    }
    url_writer.end()?;
    let dest_path = PathBuf::from("target")
        .join("site")
        .join("sitemap-index.xml");
    tokio::fs::write(dest_path, output).await?;
    Ok(())
}


pub async fn write_url_to_file(idx: usize, urls:&mut Vec<UrlEntry>,url_writer:&mut SiteMapIndexWriter<&mut Vec<u8>>, now:DateTime<FixedOffset>, base_url:&str) -> AppResult<()>{


    generate_site_map_index(idx, urls).await?;
    url_writer.sitemap(
        SiteMapEntry::builder()
            .loc(format!("{}/sitemap-index{}.xml", base_url, idx))
            .lastmod(now)
            .build()?,
    )?;

    Ok(())
}




pub async fn get_total_summoners(db: &PgPool) -> AppResult<i64> {
    let total = sqlx::query_scalar(
        "SELECT COUNT(*) FROM summoners WHERE tag_line != '' and game_name != ''",
    )
    .fetch_one(db)
    .await?;
    Ok(total)
}

pub async fn get_platforms_summoners_taglines(
    db: &PgPool,
    per_page: i64,
    page: i64,
) -> AppResult<Vec<(String, String, PlatformRouteDb, NaiveDateTime)>> {
    let offset = (page - 1) * per_page;
    sqlx::query_as::<_, (String, String, PlatformRouteDb, NaiveDateTime)>(
        "SELECT game_name, tag_line, platform, updated_at FROM summoners WHERE tag_line != '' and game_name != '' ORDER BY platform, game_name  LIMIT $1 OFFSET $2"
    ).bind(per_page)
        .bind(offset)
        .fetch_all(db)
        .await
        .map_err(|e| e.into())
}

pub fn get_site_map_url(loc: String, lastmod: Option<DateTime<FixedOffset>>) -> UrlEntry {
    let mut builder = UrlEntry::builder().loc(loc);
    if let Some(lastmod) = lastmod {
        builder = builder.lastmod(lastmod);
    }
    builder.build().unwrap()
}
