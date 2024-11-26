#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> leptos_broken_gg::backend::ssr::AppResult<()> {
    use axum::routing::get;
    use axum::Router;
    use dashmap::DashMap;
    use dotenv::dotenv;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use leptos_broken_gg::app::*;
    use leptos_broken_gg::backend;
    use leptos_broken_gg::backend::generate_sitemap::schedule_generate_site_map;
    use leptos_broken_gg::backend::live_game_cache;
    use leptos_broken_gg::backend::live_game_cache::schedule_live_game_cache_cleanup_task;
    use leptos_broken_gg::backend::updates::update_matches_task::schedule_update_matches_task;
    use leptos_broken_gg::backend::updates::update_pro_player_task::schedule_update_pro_player_task;
    use leptos_broken_gg::ssr::serve;
    use leptos_broken_gg::ssr::subscribe_sse;
    use leptos_broken_gg::ssr::AppState;
    use leptos_broken_gg::ssr::{init_database, init_riot_api};
    use memory_serve::{load_assets, CacheControl, MemoryServe};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tower_http::compression::predicate::NotForContentType;
    use tower_http::compression::predicate::SizeAbove;
    use tower_http::compression::CompressionLayer;
    use tower_http::compression::Predicate;

    dotenv().ok();
    let conf = get_configuration(None).unwrap();
    let mut leptos_options = conf.leptos_options;
    let is_prod = dotenv::var("ENV").unwrap_or("DEV".to_string()) == "PROD";
    if is_prod {
        leptos_options.site_addr = SocketAddr::from(([0, 0, 0, 0], 443));
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");
    } else {
        leptos_options.site_addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    }

    let site_address = leptos_options.site_addr;
    backend::lol_static::init_static_data().await?;
    let db = init_database().await;
    let riot_api = Arc::new(init_riot_api());
    // live game caching
    let expiration_duration = std::time::Duration::from_secs(60);
    let cleanup_interval = std::time::Duration::from_secs(30);

    let live_game_cache = Arc::new(live_game_cache::LiveGameCache::new(expiration_duration));
    let cache_for_cleanup = Arc::clone(&live_game_cache);

    let max_matches = dotenv::var("MAX_MATCHES")
        .unwrap_or_else(|_| "1500".to_string())
        .parse()?;

    let summoner_updated_sender = Arc::new(DashMap::new());

    // because of mass update/inserts and trying to limit usage of riot api request.
    // we dont want n concurrent thread updating matches and summoners
    // schedule_update_matches_task update all matches not "updated" evry  MATCH_TASK_UPDATE_INTERVAL using batch of 100 match (0.34s on server if no api limit)
    let update_interval =
        dotenv::var("MATCH_TASK_UPDATE_INTERVAL").unwrap_or_else(|_| "5".to_string());
    let update_interval_duration = tokio::time::Duration::from_secs(update_interval.parse()?);

    // thread to update matches details
    schedule_update_matches_task(
        db.clone(),
        riot_api.clone(),
        update_interval_duration,
        summoner_updated_sender.clone(),
    )
    .await;

    // thread to cleanup live game cache
    schedule_live_game_cache_cleanup_task(cache_for_cleanup, cleanup_interval).await;
    if is_prod {
        // thread to generate site map (on launch and at 3am)
        schedule_generate_site_map(db.clone()).await;
        // thead to update pro player (on launch and at 2am)
        schedule_update_pro_player_task(db.clone(), riot_api.clone()).await;
    }

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        riot_api: riot_api.clone(),
        db: db.clone(),
        live_game_cache,
        max_matches,
        summoner_updated_sender,
    };

    let routes = generate_route_list(App);
    // build our application with a route
    let app = Router::<AppState>::new()
        .nest(
            "/assets",
            MemoryServe::new(load_assets!("target/site/assets"))
                .enable_brotli(!cfg!(debug_assertions))
                .cache_control(CacheControl::Custom("public, max-age=31536000"))
                .into_router(),
        )
        .nest(
            "/pkg",
            MemoryServe::new(load_assets!("target/site/pkg"))
                .enable_brotli(!cfg!(debug_assertions))
                .cache_control(CacheControl::Custom("public, max-age=31536000"))
                .into_router(),
        )
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || provide_context(app_state.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .route("/subscribe/:summoner_id", get(subscribe_sse))
        .fallback(leptos_axum::file_and_error_handler::<LeptosOptions, _>(
            shell,
        ))
        .layer(
            CompressionLayer::new()
                .br(true)
                .deflate(true)
                .gzip(true)
                .zstd(true)
                .compress_when(
                    SizeAbove::new(32)
                        .and(NotForContentType::GRPC)
                        .and(NotForContentType::SSE),
                ),
        )
        .with_state(app_state);

    serve(app, is_prod, site_address)
        .await
        .expect("failed to serve");
    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
