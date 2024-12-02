use std::path::PathBuf;
use crate::backend::ssr::{AppResult, PlatformRouteDb};
use crate::backend::task_director::Task;
use crate::backend::tasks::calculate_next_run_to_fixed_start_hour;
use common::consts::platform_route::{PlatformRoute, PLATFORM_ROUTE_OPTIONS};
use crate::utils::summoner_url;
use axum::async_trait;
use chrono::NaiveDateTime;
use leptos::leptos_dom::log;
use sitemap::structs::UrlEntry;
use sitemap::writer::SiteMapWriter;
use sqlx::PgPool;
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
    pub fn new(db: PgPool, start_hour: u32, on_startup:bool) -> Self {
        let next_run = if on_startup{
            Instant::now()
        }else{
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

#[async_trait]
impl Task for GenerateSiteMapTask {
    async fn execute(&self) {
        if let Err(e) = generate_site_map(&self.db).await {
            log!("Failed to generate ruche-lol map: {:?}", e);
        } else {
            log!("Site map generated successfully");
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

pub async fn generate_site_map(db: &PgPool) -> AppResult<()> {
    let mut output = Vec::<u8>::new();
    {
        let writer = SiteMapWriter::new(&mut output);
        let base_url = "https://ruche.lol";
        let mut url_writer = writer.start_urlset()?;
        url_writer.url(UrlEntry::builder().loc(base_url).build()?)?;
        for platform in PLATFORM_ROUTE_OPTIONS {
            url_writer.url(
                UrlEntry::builder()
                    .loc(format!("{}/platform/{}", base_url, platform))
                    .build()?,
            )?;
        }
        let total_summoners = get_total_summoners(db).await?;
        let per_page = 500;
        let total_pages = total_summoners / per_page;
        for page in 1..=total_pages {
            let summoners = get_platforms_summoners_taglines(db, per_page, page).await?;
            for (game_name, tag_line, platform, updated_at) in summoners {
                let pt: PlatformRoute = platform.into();
                let url = format!(
                    "{}{}",
                    base_url,
                    summoner_url(pt.as_ref(), game_name.as_str(), tag_line.as_str())
                );
                url_writer.url(
                    UrlEntry::builder()
                        .loc(url)
                        .lastmod(updated_at.and_utc().fixed_offset())
                        .build()?,
                )?;
            }
        }
        url_writer.end()?;
    }

    let dest_path = PathBuf::from("target").join("site").join("sitemap.xml");
    tokio::fs::write(dest_path, output).await?;
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
