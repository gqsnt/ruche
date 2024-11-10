use leptos_broken_gg::backend::generate_sitemap::generate_site_map;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use tower_http::compression::{CompressionLayer, DefaultPredicate};
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use leptos_broken_gg::app::*;
    use std::sync::Arc;
    use leptos_broken_gg::{init_database, init_riot_api, AppState};
    use dotenv::dotenv;
    use leptos::logging::log;
    use leptos::html::tr;
    use tower_http::compression::Predicate;
    use tower_http::compression::predicate::{NotForContentType, SizeAbove};
    use memory_serve::{load_assets, CacheControl, MemoryServe};
    use tower::ServiceBuilder;
    use leptos_broken_gg::live_game_cache;
    use tokio::task;
    use leptos_broken_gg::backend;
    use leptos_broken_gg::backend::updates::update_matches_task::update_matches_task;
    use leptos_broken_gg::live_game_cache::cache_cleanup_task;

    dotenv().ok();
    let conf = get_configuration(None).unwrap();
    let mut leptos_options = conf.leptos_options;
    let _ = leptos_options.site_root.clone();
    backend::lol_static::init_static_data().await;
    let db = init_database().await;
    let riot_api = Arc::new(init_riot_api());

    // live game caching
    let expiration_duration = std::time::Duration::from_secs(60);
    let cleanup_interval = std::time::Duration::from_secs(30);
    let live_game_cache = Arc::new(live_game_cache::LiveGameCache::new(expiration_duration));
    let cache_for_cleanup = Arc::clone(&live_game_cache);

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        riot_api: riot_api.clone(),
        db: db.clone(),
        live_game_cache,
    };

    // generate sitemap
    generate_site_map(&db.clone()).await.unwrap();

    // thread to update matches data and add summoners related.
    // because of mass update/inserts and limiting usage of account_v1 request.
    // we dont want n concurrent thread updating matches and summoners
    tokio::spawn(async move {
        update_matches_task(db, riot_api).await;
    });

    // thread to cleanup live game cache data
    task::spawn(async move {
        cache_cleanup_task(cache_for_cleanup, cleanup_interval).await;
    });

    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    // build our application with a route
    let app = Router::<AppState>::new()

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
        .fallback(leptos_axum::file_and_error_handler::<LeptosOptions, _>(shell))
        .layer(
            CompressionLayer::new()
                .br(true)
                .deflate(true)
                .gzip(true)
                .zstd(true)
                .compress_when(SizeAbove::new(32)
                    .and(NotForContentType::GRPC)
                    .and(NotForContentType::SSE)),
        )
        .fallback(leptos_axum::file_and_error_handler::<LeptosOptions, _>(shell))
        .merge(
            MemoryServe::new(load_assets!("target/site/assets"))
                .enable_brotli(!cfg!(debug_assertions))
                .cache_control(CacheControl::Long)
                .into_router()
        )
        .with_state(app_state);
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
