pub mod app;
pub mod views;

pub mod backend;
pub mod utils;

pub const DB_CHUNK_SIZE: usize = 500;

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::backend::live_game_cache;
    use crate::backend::server_fns::get_encounter::ssr::find_summoner_puuid_by_id;
    use crate::backend::server_fns::get_live_game::ssr;
    use crate::utils::{Puuid, SSEEvent};
    use axum::body::Body;
    use axum::extract::{Path, Request, State};
    use axum::handler::HandlerWithoutStateExt;
    use axum::response::sse::{Event, KeepAlive, Sse};
    use axum::response::{IntoResponse, Redirect};
    use axum::{BoxError, Router};
    use axum_server::tls_rustls::RustlsConfig;
    use common::consts::platform_route::PlatformRoute;
    use dashmap::DashMap;
    use http::{StatusCode, Uri};
    use leptos::logging::log;
    use leptos::prelude::*;
    use riven::RiotApi;
    use sqlx::postgres::PgConnectOptions;
    use sqlx::PgPool;
    use std::net::SocketAddr;
    use std::sync::Arc;

    use std::path::PathBuf;

    use axum_extra::extract::Host;
    use http::HeaderValue;
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    use rustls::ServerConfig;
    use std::net::{IpAddr, Ipv6Addr};
    use std::time::Duration;
    use tokio::sync::broadcast::Sender;
    use tokio::time;
    use tokio_stream::wrappers::BroadcastStream;
    use tokio_stream::StreamExt;
    use tower::ServiceExt;
    use tower_http::services::ServeFile;
    use tower_http::set_header::SetResponseHeaderLayer;

    pub type RiotApiState = Arc<RiotApi>;
    pub type SubscriberMap = DashMap<i32, Sender<SSEEvent>>;

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
        Path((platform_route, summoner_id)): Path<(String, i32)>,
        State(state): State<AppState>,
    ) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
        let (rx, mut pending_event) = {
            let mut in_live_game = state
                .live_game_cache
                .summoner_id_to_game
                .get(&summoner_id)
                .is_some();
            // check if existing channel exists
            let entry = state
                .summoner_updated_sender
                .entry(summoner_id)
                .or_insert_with(|| {
                    let (sender, _) = tokio::sync::broadcast::channel(3);
                    if state
                        .live_game_cache
                        .summoner_id_to_game
                        .get(&summoner_id)
                        .is_some()
                    {
                        in_live_game = true;
                    } else {
                        // fetch first time live game data
                        let inner_sender = sender.clone();
                        let db = state.db.clone();
                        let riot_api = state.riot_api.clone();
                        let live_game_cache = state.live_game_cache.clone();
                        let platform_route = PlatformRoute::from_code(platform_route.as_str()).unwrap();
                        tokio::spawn(async move {
                            let puuid = Puuid::new(
                                find_summoner_puuid_by_id(&db, summoner_id)
                                    .await
                                    .unwrap()
                                    .as_str(),
                            );
                            let live_game =
                                ssr::get_live_game_data(&db, &riot_api, puuid, platform_route)
                                    .await
                                    .unwrap();
                            if let Some((summoner_ids, live_game)) = live_game {
                                live_game_cache.set_game_data(
                                    live_game.game_id,
                                    summoner_ids,
                                    live_game,
                                );
                                inner_sender.send(SSEEvent::LiveGame(Some(1))).unwrap();
                            } else {
                                inner_sender.send(SSEEvent::LiveGame(None)).unwrap();
                            }
                        });
                    }

                    sender
                });

            (
                entry.value().subscribe(),
                in_live_game.then(|| SSEEvent::LiveGame(Some(1))),
            )
        };
        let mut summoner_matches_update_count = 0u16;
        let mut summoner_live_game_version_update_count = 0u16;
        let debounce_interval = Duration::from_millis(500);

        let stream = async_stream::stream! {

                      let mut event_id: u64 = 0;
                   // Use an interval timer to enforce the 1-second delay
                   let mut interval = time::interval(debounce_interval);
                   interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
                   interval.reset();


                   // Wrap the receiver in a stream
                   let mut rx_stream = BroadcastStream::new(rx);

                   loop {
                       tokio::select! {
                           message = rx_stream.next() =>{
                               pending_event= match message{
                                   Some(Ok(SSEEvent::SummonerMatches(_))) => {
                                        summoner_matches_update_count += 1;
                                       Some(SSEEvent::SummonerMatches(summoner_matches_update_count))
                                   }
                                   Some(Ok(SSEEvent::LiveGame(Some(_))) ) => {
                                       summoner_live_game_version_update_count += 1;
                                       Some(SSEEvent::LiveGame(Some(summoner_live_game_version_update_count)))
                                   }
                                   Some(Ok(SSEEvent::LiveGame(None))) => {
                                       Some(SSEEvent::LiveGame(None))
                                   }
                                   _ => None,
                               };
                           }
                           _ = interval.tick() => {
                                 if let Some(event) = pending_event.take() {
                                       event_id = event_id.wrapping_add(1);
                                        yield Ok(
                                            Event::default()
                                                .id(event_id.to_string())
                                                .data(event.to_string())
                                                .retry(Duration::from_millis(3000))
                                        );
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

        let cert_der = CertificateDer::from_pem_file(&cert).expect("failed to load cert");
        let key_der = PrivateKeyDer::from_pem_file(&key).expect("failed to load key");

        let h2_tls = make_rustls_server_config_h2(cert_der.clone(), key_der.clone_key());

        let h3_tls = make_rustls_server_config_h3(cert_der, key_der.clone_key());

        let config = RustlsConfig::from_config(h2_tls.clone());
        // Create a QUIC (HTTP/3) endpoint so we advertise Alt-Svc for browsers.
        // Keep the endpoint alive for the lifetime of the server by binding it here.
        let quic_ep = make_quinn_server_endpoint_dual(socket_addr, h3_tls);
        let acceptor = h3_util::quinn::H3QuinnAcceptor::new(quic_ep);

        // Spawn the H3 router in the background. Clone the app so we don't move it
        // twice (once into the H3 task, once into the h2 server below).
        let alt_svc_value = format!("h3=\":{}\"; ma=2592000; persist=1", socket_addr.port());
        let srv_h = axum_h3::H3Router::new(app.clone()).serve(acceptor);
        log!("listening on {}", socket_addr);
        // Advertise HTTP/3 (h3) to browsers using Alt-Svc so that clients can attempt h3 (QUIC) on h3_addr
        let app = app.layer(SetResponseHeaderLayer::if_not_present(
            http::header::ALT_SVC,
            HeaderValue::from_str(&alt_svc_value).unwrap(),
        ));
        let srv = axum_server::bind_rustls(socket_addr, config).serve(app.into_make_service());
        let (srv_h, srv) = tokio::join!(srv_h, srv,);
        match srv_h {
            Ok(_) => log!("H3 server exited normally"),
            Err(e) => log!("H3 server exited with error: {}", e),
        }
        match srv {
            Ok(_) => log!("H2 server exited normally"),
            Err(e) => log!("H2 server exited with error: {}", e),
        }
        Ok(())
    }

    async fn redirect_http_to_https() {
        fn make_https(host: String, uri: Uri) -> Result<Uri, BoxError> {
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
        let cert_path = PathBuf::from("certs").join("localhost+2.pem");
        let key_path = PathBuf::from("certs").join("localhost+2-key.pem");
        let axum_rustls = RustlsConfig::from_pem_file(cert_path.clone(), key_path.clone())
            .await
            .expect("failed to load rustls config for local TLS");
        log!("listening (local TLS, H1/H2 only) on {}", socket_addr);
        axum_server::bind_rustls(socket_addr, axum_rustls)
            .serve(app.into_make_service())
            .await
            .unwrap();
        Ok(())
    }

    pub async fn get_sitemap() -> impl IntoResponse {
        match ServeFile::new(
            PathBuf::from("target")
                .join("site")
                .join("sitemap-index.xml"),
        )
        .oneshot(Request::new(Body::empty()))
        .await
        {
            Ok(mut resp) => {
                resp.headers_mut()
                    .insert("Content-Type", "application/xml".parse().unwrap());
                Ok(resp.into_response())
            }
            Err(e) => {
                log!("Error serving sitemap: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error serving sitemap".to_string(),
                ))
            }
        }
    }

    pub fn make_quinn_server_endpoint_dual(
        in_addr: SocketAddr,
        tls_config: Arc<ServerConfig>,
    ) -> quinn::Endpoint {
        let mut tcfg = quinn::TransportConfig::default();
        // plus de flux simultanés (ajustez selon charge)
        tcfg.max_concurrent_bidi_streams(quinn::VarInt::from_u32(256));
        tcfg.max_concurrent_uni_streams(quinn::VarInt::from_u32(256));
        // éviter les coupures sur connexions « calmes » (SSE)
        tcfg.max_idle_timeout(Some(
            std::time::Duration::from_secs(120).try_into().unwrap(),
        ));
        tcfg.keep_alive_interval(Some(std::time::Duration::from_secs(15)));
        // si vous projetez d’utiliser des datagrams (WebTransport plus tard)
        tcfg.datagram_receive_buffer_size(Some(1 << 20)); // 1 MiB

        let mut scfg = quinn::ServerConfig::with_crypto(Arc::new(
            quinn::crypto::rustls::QuicServerConfig::try_from(tls_config).unwrap(),
        ));
        scfg.transport_config(Arc::new(tcfg));
        // Bind QUIC on [::]:port (IPv6 unspecified). On Windows this is dual-stack by default.
        let v6_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), in_addr.port());
        quinn::Endpoint::server(scfg, v6_addr).expect("failed to bind QUIC endpoint")
    }

    pub fn make_rustls_server_config_h3(
        cert: CertificateDer<'static>,
        key: PrivateKeyDer,
    ) -> Arc<rustls::ServerConfig> {
        let mut tls_config = rustls::ServerConfig::builder_with_provider(
            rustls::crypto::ring::default_provider().into(),
        )
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert.clone()], key.clone_key())
        .unwrap();
        tls_config.alpn_protocols = vec![b"h3".to_vec()];
        tls_config.max_early_data_size = u32::MAX;
        Arc::new(tls_config)
    }

    pub fn make_rustls_server_config_h2(
        cert: CertificateDer<'static>,
        key: PrivateKeyDer,
    ) -> Arc<rustls::ServerConfig> {
        let mut tls_config = rustls::ServerConfig::builder_with_provider(
            rustls::crypto::ring::default_provider().into(),
        )
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert.clone()], key.clone_key())
        .unwrap();
        tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Arc::new(tls_config)
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
