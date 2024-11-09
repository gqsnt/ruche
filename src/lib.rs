pub mod app;
pub mod error_template;
pub mod models;

#[cfg(feature = "ssr")]
pub mod lol_static;


pub mod apis;
pub mod consts;

#[cfg(feature = "ssr")]
pub mod live_game_cache;
pub mod views;

#[cfg(feature = "ssr")]
pub const DB_CHUNK_SIZE: usize = 500;
#[cfg(feature = "ssr")]
use axum::handler::HandlerWithoutStateExt;
#[cfg(feature = "ssr")]
use axum::{ServiceExt};
#[cfg(feature = "ssr")]
use futures::StreamExt;
use leptos::prelude::BindAttribute;
#[cfg(feature = "ssr")]
use leptos::prelude::LeptosOptions;


#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

pub fn version_to_major_minor(version: String) -> String {
    let mut split = version.split(".");
    if split.clone().count() < 2 {
        panic!("version_to_major_minor: version: {}", version);
    }
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
    pub live_game_cache: std::sync::Arc<live_game_cache::LiveGameCache>,
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


// AXUM AS ROUTER FULL
//
// #[cfg(feature = "ssr")]
// pub async fn serve_with_tsl(
//     app: Router,
//     domains: impl IntoIterator<Item=impl AsRef<str>>,
//     email_for_lets_encrypt: &str,
//     cert_cache_dir: impl Into<PathBuf>,
// ) -> Result<(), axum::Error> {
//     let ccache: PathBuf = cert_cache_dir.into();
//     if !ccache.exists() {
//         fs::create_dir_all(&ccache).expect("failed to create cache dir");
//     }
//
//     let mut state = AcmeConfig::new(domains)
//         .contact([format!("mailto:{email_for_lets_encrypt}")])
//         .cache(DirCache::new(ccache))
//         .directory_lets_encrypt(true)
//         .state();
//
//     let acceptor = state.axum_acceptor(state.challenge_rustls_config());
//
//     tokio::spawn(async move {
//         loop {
//             match state.next().await.unwrap() {
//                 Ok(ok) => log!("event: {ok:?}"),
//                 Err(err) => log!("error: {err}"),
//             }
//         }
//     });
//
//
//     let addr = SocketAddr::from(([0, 0, 0, 0], 443));
//     tokio::spawn(redirect_http_to_https());
//
//     let tls_server = axum_server::bind(addr)
//         .acceptor(acceptor)
//         .serve(app.into_make_service());
//
//     tls_server.await.unwrap();
//     Ok(())
// }
//
//
// #[cfg(feature = "ssr")]
// async fn redirect_http_to_https() {
//     fn make_https(host: String, uri: Uri) -> Result<Uri, BoxError> {
//         let mut parts = uri.into_parts();
//
//         parts.scheme = Some(axum::http::uri::Scheme::HTTPS);
//
//         if parts.path_and_query.is_none() {
//             parts.path_and_query = Some("/".parse().unwrap());
//         }
//
//         let https_host = host.replace("80", "443");
//         parts.authority = Some(https_host.parse()?);
//
//         Ok(Uri::from_parts(parts)?)
//     }
//
//     let redirect = move |Host(host): Host, uri: Uri| async move {
//         match make_https(host, uri) {
//             Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
//             Err(error) => {
//                 tracing::warn!(%error, "failed to convert URI to HTTPS");
//                 Err(StatusCode::BAD_REQUEST)
//             }
//         }
//     };
//
//     let addr = SocketAddr::from(([0, 0, 0, 0], 80));
//     let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
//     tracing::debug!("listening on {}", listener.local_addr().unwrap());
//     axum::serve(listener, redirect.into_make_service())
//         .await
//         .unwrap();
// }
//
// #[cfg(feature = "ssr")]
// pub async fn server_locally(app: Router) -> Result<(), axum::Error> {
//     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//     let listener = tokio::net::TcpListener::bind(&addr)
//         .await
//         .expect("Creating listener");
//     Ok(axum::serve(listener, app.into_make_service()).await.unwrap())
// }
