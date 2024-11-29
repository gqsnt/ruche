#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> broken_gg::backend::ssr::AppResult<()> {
    use axum::routing::get;
    use axum::Router;
    use dashmap::DashMap;
    use dotenv::dotenv;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use broken_gg::app::*;
    use broken_gg::backend::live_game_cache::LiveGameCache;
    use broken_gg::backend::task_director::TaskDirector;
    use broken_gg::backend::tasks::generate_sitemap::GenerateSiteMapTask;
    use broken_gg::backend::tasks::live_game_cache_cleanup::LiveGameCacheCleanupTask;
    use broken_gg::backend::tasks::sse_broadcast_match_updated_cleanup::SummonerUpdatedSenderCleanupTask;
    use broken_gg::backend::tasks::update_matches::UpdateMatchesTask;
    use broken_gg::backend::tasks::update_pro_players::UpdateProPlayerTask;
    use broken_gg::ssr::serve;
    use broken_gg::ssr::sse_broadcast_match_updated;
    use broken_gg::ssr::AppState;
    use broken_gg::ssr::{init_database, init_riot_api};
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
    let env_type = dotenv::var("ENV").unwrap_or("DEV".to_string());
    let is_prod = env_type == "PROD";

    let max_matches = dotenv::var("MAX_MATCHES")
        .unwrap_or_else(|_| "1500".to_string())
        .parse()?;

    let update_interval_duration = tokio::time::Duration::from_secs(
        dotenv::var("MATCH_TASK_UPDATE_INTERVAL")
            .unwrap_or_else(|_| "5".to_string())
            .parse()?,
    );

    let lol_pro_task_on_startup=dotenv::var("LOL_PRO_TASK_ON_STARTUP").unwrap_or("false".to_string()).as_str() == "true";
    let site_map_task_on_startup=dotenv::var("SITE_MAP_TASK_ON_STARTUP").unwrap_or("false".to_string()) == "true";

    log!("Starting BrokenGG as {}", env_type);
    log!("Update interval duration: {:?}", update_interval_duration);
    log!("Max matches: {}", max_matches);

    if is_prod {
        leptos_options.site_addr = SocketAddr::from(([0, 0, 0, 0], 443));
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");
    } else {
        leptos_options.site_addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    }

    let site_address = leptos_options.site_addr;

    let db = init_database().await;
    let riot_api = Arc::new(init_riot_api());
    let live_game_cache = Arc::new(LiveGameCache::new(std::time::Duration::from_secs(60)));
    let summoner_updated_sender = Arc::new(DashMap::new());

    let mut task_director = TaskDirector::default();
    task_director.add_task(LiveGameCacheCleanupTask::new(
        Arc::clone(&live_game_cache),
        tokio::time::Duration::from_secs(30),
    ));

    // download and update of match details are done in fast bg task. to not get concurrent mass insert/update
    task_director.add_task(UpdateMatchesTask::new(
        db.clone(),
        Arc::clone(&riot_api),
        update_interval_duration,
        Arc::clone(&summoner_updated_sender),
    ));

    // cleanup sse_broadcast_match_updated subscriptions
    task_director.add_task(SummonerUpdatedSenderCleanupTask::new(
        Arc::clone(&summoner_updated_sender),
        tokio::time::Duration::from_secs(10),
    ));

    if is_prod {
        task_director.add_task(UpdateProPlayerTask::new(db.clone(), riot_api.clone(), 2,lol_pro_task_on_startup));
        task_director.add_task(GenerateSiteMapTask::new(db.clone(), 3, site_map_task_on_startup));
    }
    tokio::spawn(async move {
        task_director.run().await;
    });

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        riot_api,
        db,
        live_game_cache,
        max_matches,
        summoner_updated_sender,
    };

    let routes = generate_route_list(App);
    // build our application with a route
    let app = Router::<AppState>::new()
        .nest(
            "/assets",
            MemoryServe::new(load_assets!("../target/site/assets"))
                .enable_brotli(!cfg!(debug_assertions))
                .cache_control(CacheControl::Custom("public, max-age=31536000"))
                .into_router(),
        )
        .nest(
            "/pkg",
            MemoryServe::new(load_assets!("../target/site/pkg"))
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
        .route(
            "/sse/match_updated/:summoner_id",
            get(sse_broadcast_match_updated),
        )
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
    // see common.rs for hydration function instead
}
