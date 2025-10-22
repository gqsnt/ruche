use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::future::Future;
use std::pin::Pin;
use sqlx::PgPool;
use tokio::time::Instant;
use leptos::leptos_dom::log;
use crate::backend::task_director::Task;
use crate::backend::tasks::calculate_next_run_to_fixed_start_hour;

pub struct DailySqlCleanTask {
    pub db: PgPool,
    pub start_hour: u32,
    pub next_run: Instant,
    pub running: Arc<AtomicBool>,
}

impl DailySqlCleanTask {
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

impl Task for DailySqlCleanTask {
    fn execute(&self) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        let db = self.db.clone();
        Box::pin(async move {
            let table_info = get_table_info(&db).await;
            log!("Daily Clean Task Before:");
            for row in table_info{
                log!("{}", row);
            }

            let _ = sqlx::query("VACUUM ANALYSE ").execute(&db).await;
            let table_info = get_table_info(&db).await;
            log!("Daily Clean Task After:");
            for row in table_info{
                log!("{}", row);
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
        "Daily SQL Clean Task"
    }

    fn allow_concurrent(&self) -> bool {
        false // Do not allow concurrent executions
    }
}


pub async fn get_table_info(db: &PgPool) -> Vec<TableInfo>{
    sqlx::query_as::<_, TableInfo>(r#"
            SELECT
                relname AS table_name,
                pg_size_pretty(pg_total_relation_size(relid)) AS total_size,
                pg_size_pretty(pg_relation_size(relid)) AS table_size,
                pg_size_pretty(pg_total_relation_size(relid) - pg_relation_size(relid)) AS index_size
            FROM
                pg_catalog.pg_statio_user_tables
            ORDER BY
                pg_total_relation_size(relid) DESC;
        "#).fetch_all(db).await.unwrap()
}


#[derive(sqlx::FromRow, Debug)]
pub struct TableInfo{
    table_name:String,
    total_size:String,
    table_size:String,
    index_size:String,
}


impl std::fmt::Display for TableInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Table Name: {}, Total Size: {}, Table Size: {}, Index Size: {}", self.table_name, self.total_size, self.table_size, self.index_size)
    }
}
