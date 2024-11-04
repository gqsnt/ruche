use leptos::prelude::LeptosOptions;

pub mod app;
pub mod error_template;
pub mod models;

#[cfg(feature = "ssr")]
pub mod lol_static;
pub mod components;


pub mod apis;
pub mod consts;

#[cfg(feature = "ssr")]
pub const DB_CHUNK_SIZE: usize = 500;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

pub fn version_to_major_minor(version: String) -> String {
    let mut split = version.split(".");
    let major = split.next().unwrap();
    let minor = split.next().unwrap();
    format!("{}.{}", major, minor)
}


#[cfg(feature = "ssr")]
#[derive(Clone, axum::extract::FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub riot_api: std::sync::Arc<riven::RiotApi>,
    pub db: sqlx::PgPool,
}


#[cfg(feature = "ssr")]
pub fn init_riot_api() -> riven::RiotApi {
    let api_key = dotenv::var("RIOT_API_KEY").expect("RIOT_API_KEY must be set");
    riven::RiotApi::new(api_key)
}

#[cfg(feature = "ssr")]
pub async fn init_database() -> sqlx::PgPool {
    let database_url = dotenv::var("DATABASE_URL").expect("no database url specify");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(100)
        .connect(database_url.as_str())
        .await
        .expect("could not connect to database_url");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("migrations failed");

    pool
}

