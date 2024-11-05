

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
    use leptos_broken_gg::lol_static;
    use leptos::logging::log;
    use leptos::html::tr;
    use tower_http::compression::Predicate;
    use tower_http::compression::predicate::{NotForContentType, SizeAbove};
    use leptos_broken_gg::{serve_with_tsl, server_locally};
    use memory_serve::{load_assets, CacheControl, MemoryServe};
    use tower::ServiceBuilder;
    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    dotenv().ok();
    let conf = get_configuration(None).unwrap();
    let mut leptos_options = conf.leptos_options;
    let _ = leptos_options.site_root.clone();
    lol_static::init_static_data().await;

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        riot_api: Arc::new(init_riot_api()),
        db: init_database().await,
    };
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    // build our application with a route
    let app = Router::<AppState>::new()
        .merge(
            MemoryServe::new(load_assets!("target/site"))
                .enable_brotli(!cfg!(debug_assertions))
                .cache_control(CacheControl::Long)
                .into_router()
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
