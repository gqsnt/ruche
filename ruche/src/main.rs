use http::HeaderValue;
use sqlx::PgPool;
use tower::ServiceBuilder;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> ruche::backend::ssr::AppResult<()> {
    use axum::routing::get;
    use axum::Router;
    use dashmap::DashMap;
    use dotenv::dotenv;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use memory_serve::{load_assets, CacheControl, MemoryServe};
    use ruche::app::*;
    use ruche::backend::live_game_cache::LiveGameCache;
    use ruche::backend::task_director::TaskDirector;
    use ruche::backend::tasks::daily_sql_clean::DailySqlCleanTask;
    use ruche::backend::tasks::generate_sitemap::GenerateSiteMapTask;
    use ruche::backend::tasks::handle_live_game_cache::HandleLiveGameCacheTask;
    use ruche::backend::tasks::sse_broadcast_match_updated_cleanup::SummonerUpdatedSenderCleanupTask;
    use ruche::backend::tasks::update_matches::UpdateMatchesTask;
    use ruche::backend::tasks::update_pro_players::UpdateProPlayerTask;
    use ruche::ssr::get_sitemap;
    use ruche::ssr::init_riot_api;
    use ruche::ssr::serve;
    use ruche::ssr::sse_broadcast_match_updated;
    use ruche::ssr::AppState;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tower_http::compression::predicate::NotForContentType;
    use tower_http::compression::predicate::SizeAbove;
    use tower_http::compression::CompressionLayer;
    use tower_http::compression::Predicate;
    use tower_http::CompressionLevel;

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

    let live_game_cache_interval_duration = tokio::time::Duration::from_secs(
        dotenv::var("LIVE_GAME_CACHE_UPDATE_INTERVAL")
            .unwrap_or_else(|_| "30".to_string())
            .parse()?,
    );

    let lol_pro_task_on_startup = dotenv::var("LOL_PRO_TASK_ON_STARTUP")
        .unwrap_or("false".to_string())
        .eq("true");
    let site_map_task_on_startup = dotenv::var("SITE_MAP_TASK_ON_STARTUP")
        .unwrap_or("false".to_string())
        .eq("true");

    log!("Starting Ruche as {}", env_type);
    log!("Update interval duration: {:?}", update_interval_duration);
    log!("Max matches: {}", max_matches);
    log!("LOL Pro Task on Startup: {}", lol_pro_task_on_startup);
    log!("Site Map Task on Startup: {}", site_map_task_on_startup);

    if is_prod {
        leptos_options.site_addr = SocketAddr::from(([0, 0, 0, 0], 443));
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");
    } else {
        leptos_options.site_addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    }
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let site_address = leptos_options.site_addr;
    let alt_svc_value = format!("h3=\":{}\"; ma=2592000; persist=1", site_address.port());

    let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(database_url.as_str())
        .await
        .expect("Could not connect to database");
    let riot_api = Arc::new(init_riot_api());
    let live_game_cache = Arc::new(LiveGameCache::new(std::time::Duration::from_secs(60)));
    let summoner_updated_sender = Arc::new(DashMap::new());
    let mut task_director = TaskDirector::default();
    task_director.add_task(HandleLiveGameCacheTask::new(
        pool.clone(),
        riot_api.clone(),
        live_game_cache.clone(),
        summoner_updated_sender.clone(),
        live_game_cache_interval_duration,
    ));

    // download and update of match details are done in fast bg task. to not get concurrent mass insert/update
    task_director.add_task(UpdateMatchesTask::new(
        pool.clone(),
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
        task_director.add_task(GenerateSiteMapTask::new(
            pool.clone(),
            3,
            site_map_task_on_startup,
        ));
        task_director.add_task(DailySqlCleanTask::new(pool.clone(), 1, true));
        task_director.add_task(UpdateProPlayerTask::new(
            pool.clone(),
            riot_api.clone(),
            2,
            lol_pro_task_on_startup,
        ));
    }
    tokio::spawn(async move {
        task_director.run().await;
    });

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        riot_api,
        db: pool,
        live_game_cache,
        max_matches,
        summoner_updated_sender,
    };

    let routes = generate_route_list(App);

    let middleware = ServiceBuilder::new()
        .layer(
            CompressionLayer::new()
                .br(true)
                .zstd(true)
                .quality(CompressionLevel::Default)
                .compress_when(
                    SizeAbove::new(256)
                        .and(NotForContentType::GRPC)
                        .and(NotForContentType::IMAGES)
                        .and(NotForContentType::SSE)
                        .and(NotForContentType::const_new("text/javascript"))
                        .and(NotForContentType::const_new("application/wasm"))
                        .and(NotForContentType::const_new("text/css")),
                ),
        )
        .layer(
            tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                http::header::ALT_SVC,
                HeaderValue::from_str(&alt_svc_value).unwrap(),
            ),
        );

    // build our application with a route
    let app = Router::<AppState>::new()
        // .nest(
        //     "/assets",
        //     MemoryServe::new(load_assets!("../target/site/assets"))
        //         .enable_brotli(!cfg!(debug_assertions))
        //         .cache_control(CacheControl::Custom("public, max-age=31536000"))
        //         .into_router::<AppState>(),
        // )
        // .nest(
        //     "/pkg",
        //     MemoryServe::new(load_assets!("../target/site/pkg"))
        //         .enable_brotli(!cfg!(debug_assertions))
        //         .cache_control(CacheControl::Custom("public, max-age=31536000"))
        //         .into_router::<AppState>(),
        // )
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
            "/sse/match_updated/{platform_route}/{summoner_id}",
            get(sse_broadcast_match_updated),
        )
        .route("/sitemap-index.xml", get(get_sitemap))
        .fallback(leptos_axum::file_and_error_handler::<LeptosOptions, _>(
            shell,
        ))
        .layer(middleware)
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
