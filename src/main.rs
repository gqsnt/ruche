


#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use leptos_broken_gg::app::*;
    use leptos_broken_gg::fileserv::file_and_error_handler;
    use std::sync::Arc;
    use leptos_broken_gg::{init_database, init_riot_api, AppState};
    use leptos_broken_gg::lol_static;
    use dotenv::dotenv;

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    dotenv().ok();
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    lol_static::init_static_data().await;
    let app_state = AppState{
        leptos_options: leptos_options.clone(),
        riot_api: Arc::new(init_riot_api()),
        db: init_database().await,
    };
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
            App
        )
        .fallback(file_and_error_handler)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
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
