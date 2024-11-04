use crate::components::platform_type_page::PlatformTypePage;
use crate::components::summoner_page::SummonerPage;
use leptos::config::LeptosOptions;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Stylesheet, Title};
use leptos_router::components::{ParentRoute, Redirect};
use leptos_router::{components::{Route, Router, Routes}, ParamSegment, StaticSegment};


pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options=options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {


    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos-broken-gg.css" />

        // sets the document title
        <Title text="Broken.gg" />
        <Meta name="color-scheme" content="dark light" />
        // content for this welcome page
        <Router>
            <main>
                <Routes transition=true fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=move || view! { <Redirect path="EUW" /> } />
                    <ParentRoute path=ParamSegment("platform_type") view=PlatformTypePage>
                        <Route path=StaticSegment("") view=move || view! {} />
                        <Route
                            path=(StaticSegment("summoners"), ParamSegment("summoner_slug"))
                            view=SummonerPage
                        />
                    </ParentRoute>
                </Routes>
            </main>
        </Router>
    }
}
