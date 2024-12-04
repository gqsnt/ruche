pub mod app;
pub mod views;

pub mod backend;
pub mod utils;

pub const DB_CHUNK_SIZE: usize = 500;

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::live_game_cache;
    use axum::extract::{Host, Path, State};
    use axum::handler::HandlerWithoutStateExt;
    use axum::response::sse::{Event, KeepAlive, Sse};
    use axum::response::Redirect;
    use axum::Router;
    use axum_server::tls_rustls::RustlsConfig;
    use dashmap::DashMap;
    use http::{StatusCode, Uri};
    use leptos::logging::log;
    use leptos::prelude::*;
    use riven::RiotApi;
    use sqlx::postgres::PgConnectOptions;
    use sqlx::PgPool;
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::broadcast::Sender;
    use tokio::time;
    use tokio_stream::wrappers::BroadcastStream;
    use tokio_stream::StreamExt;

    pub type RiotApiState = Arc<RiotApi>;
    pub type SubscriberMap = DashMap<i32, Sender<()>>;

    #[derive(Clone, axum::extract::FromRef)]
    pub struct AppState {
        pub leptos_options: LeptosOptions,
        pub riot_api: RiotApiState,
        pub db: PgPool,
        pub live_game_cache: Arc<live_game_cache::LiveGameCache>,
        pub max_matches: usize,
        pub summoner_updated_sender: Arc<SubscriberMap>,
    }

    pub fn init_riot_api() -> RiotApi {
        let api_key = dotenv::var("RIOT_API_KEY").expect("RIOT_API_KEY must be set");
        RiotApi::new(api_key)
    }
    pub async fn init_database() -> PgPool {
        let max_connections = dotenv::var("MAX_PG_CONNECTIONS")
            .unwrap_or("10".to_string())
            .parse::<u32>()
            .unwrap_or(10);
        let db_username = dotenv::var("DB_USER_NAME").expect("no db username specify");
        let db_password = dotenv::var("DB_PASSWORD").expect("no db password specify");
        let db_name = dotenv::var("DB_NAME").expect("no db name specify");
        let socket = dotenv::var("DB_SOCKET").unwrap_or("".to_string());
        let opts = PgConnectOptions::new()
            .username(db_username.as_str())
            .password(db_password.as_str())
            .database(db_name.as_str())
            .socket(socket.as_str());
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(max_connections)
            .connect_with(opts)
            .await
            .expect("failed to connect to database");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("migrations failed");

        pool
    }

    pub async fn sse_broadcast_match_updated(
        Path(summoner_id): Path<i32>,
        State(state): State<AppState>,
    ) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
        // check if existing channel
        let rx = state
            .summoner_updated_sender
            .entry(summoner_id)
            .or_insert_with(|| {
                let (sender, _) = tokio::sync::broadcast::channel(1);
                sender
            })
            .value()
            .subscribe();
        let mut count = 0u16;
        let debounce_interval = Duration::from_secs(1);

        let stream = async_stream::stream! {
            // Use an interval timer to enforce the 1-second delay
            let mut interval = time::interval(debounce_interval);
            // Initially, we don't want to wait
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
            interval.reset();

            // Create a variable to track pending updates
            let mut pending_update = false;

            // Wrap the receiver in a stream
            let mut rx_stream = BroadcastStream::new(rx);

            loop {
                tokio::select! {
                    _ = rx_stream.next() => {
                        pending_update = true;
                    }
                    _ = interval.tick() => {
                        if pending_update {
                            // Send the update to the client
                            count+=1;
                            yield Ok(Event::default().data(count.to_string()));
                            pending_update = false;
                        }
                    }
                    else => {
                        // Stream has ended
                        break;
                    }
                }
            }
        };

        Sse::new(stream).keep_alive(KeepAlive::default())
    }

    pub async fn serve(
        app: Router,
        is_prod: bool,
        socket_addr: SocketAddr,
    ) -> Result<(), axum::Error> {
        if is_prod {
            tokio::spawn(redirect_http_to_https());
            serve_with_tsl(app, socket_addr).await
        } else {
            serve_locally(app, socket_addr).await
        }
    }

    pub async fn serve_with_tsl(app: Router, socket_addr: SocketAddr) -> Result<(), axum::Error> {
        let lets_encrypt_dir = dotenv::var("LETS_ENCRYPT_PATH").expect("LETS_ENCRYPT_PATH not set");
        let lets_encrypt_dir = PathBuf::from(lets_encrypt_dir);
        let cert = lets_encrypt_dir.join("fullchain.pem");
        let key = lets_encrypt_dir.join("privkey.pem");
        if !cert.exists() || !key.exists() {
            panic!("Certificate or key file not found");
        }
        let config = RustlsConfig::from_pem_file(
            cert,
            key,
        )
        .await
        .expect("failed to load rustls config");
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

            parts.scheme = Some(http::uri::Scheme::HTTPS);

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
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
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
