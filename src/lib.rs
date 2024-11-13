pub mod app;
pub mod consts;
pub mod views;

pub mod backend;
pub mod utils;


pub const DB_CHUNK_SIZE: usize = 500;
pub const DATE_FORMAT: &str = "%d/%m/%Y %H:%M";


#[cfg(feature = "ssr")]
pub mod ssr {
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use axum::extract::Host;
    use axum::handler::HandlerWithoutStateExt;
    use axum::response::Redirect;
    use axum::Router;
    use axum_server::tls_rustls::RustlsConfig;
    use http::{StatusCode, Uri};
    use leptos::logging::log;
    use crate::backend::live_game_cache;
    use leptos::prelude::*;

    #[derive(Clone, axum::extract::FromRef)]
    pub struct AppState {
        pub leptos_options: LeptosOptions,
        pub riot_api: std::sync::Arc<riven::RiotApi>,
        pub db: sqlx::PgPool,
        pub live_game_cache: std::sync::Arc<live_game_cache::LiveGameCache>,
        pub max_matches: usize,
    }

    pub fn init_riot_api() -> riven::RiotApi {
        let api_key = dotenv::var("RIOT_API_KEY").expect("RIOT_API_KEY must be set");
        riven::RiotApi::new(api_key)
    }
    pub async fn init_database() -> sqlx::PgPool {
        let database_url = dotenv::var("DATABASE_URL").expect("no database url specify");
        let max_connections = dotenv::var("MAX_PG_CONNECTIONS").unwrap_or("10".to_string());
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(max_connections.parse::<u32>().unwrap_or(10))
            .connect(database_url.as_str())
            .await
            .expect("could not connect to database_url");

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("migrations failed");

        pool
    }


    pub async fn serve(app:Router, is_prod:bool, socket_addr: SocketAddr) -> Result<(), axum::Error> {
        if is_prod{
            tokio::spawn(redirect_http_to_https());
            serve_with_tsl(app, socket_addr).await

        }else{
            serve_locally(app, socket_addr).await
        }
    }


    pub async fn serve_with_tsl(
        app: Router,
        socket_addr: SocketAddr
    ) -> Result<(), axum::Error> {


        let config = RustlsConfig::from_pem_file(
            PathBuf::from("signed_certs")
                .join("cert.pem"),
            PathBuf::from("signed_certs")
                .join("key.pem"),
        ).await.expect("failed to load rustls config");
        log!("listening on {}", socket_addr);
        axum_server::bind_rustls(socket_addr, config)
            .serve(app.into_make_service())
            .await
            .unwrap();
        Ok(())
    }


    async fn redirect_http_to_https() {
        fn make_https(host: String, uri: Uri) -> Result<Uri> {
            let mut parts = uri.into_parts();

            parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

            if parts.path_and_query.is_none() {
                parts.path_and_query = Some("/".parse()?);
            }

            let https_host = host.replace("80", "443");
            parts.authority = Some(https_host.parse()?);

            Ok(Uri::from_parts(parts)?)
        }

        let redirect = move |Host(host): Host, uri: Uri| async move {
            match make_https(host, uri) {
                Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
                Err(error) => {
                    tracing::warn!(%error, "failed to convert URI to HTTPS");
                    Err(StatusCode::BAD_REQUEST)
                }
            }
        };

        let addr = SocketAddr::from(([0, 0, 0, 0], 80));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, redirect.into_make_service())
            .await
            .unwrap();
    }

    pub async fn serve_locally(app: Router, socket_addr: SocketAddr) -> Result<(), axum::Error> {
        let listener = tokio::net::TcpListener::bind(&socket_addr)
            .await
            .expect("Creating listener");
        log!("listening on {}", socket_addr);
        axum::serve(listener, app.into_make_service()).await.unwrap();
        Ok(())
    }

}


#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}


