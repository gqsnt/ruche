use crate::views::platform_type_page::PlatformTypePage;
use crate::views::summoner_page::SummonerPage;
use leptos::config::LeptosOptions;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::*;

use leptos_meta::{provide_meta_context, Link, Meta, MetaTags, Stylesheet, Title};
use leptos_router::components::{ParentRoute, Redirect};
use leptos_router::{components::{Route, Router, Routes}, path, ParamSegment, StaticSegment};
use serde::{Deserialize, Serialize};
use crate::views::summoner_page::summoner_champions_page::SummonerChampionsPage;
use crate::views::summoner_page::summoner_encounter_page::SummonerEncounterPage;
use crate::views::summoner_page::summoner_encounters_page::SummonerEncountersPage;
use crate::views::summoner_page::summoner_live_page::SummonerLivePage;
use crate::views::summoner_page::summoner_matches_page::SummonerMatchesPage;

pub const SITE_URL: &str = "https://ruche.lol";

#[derive(Clone, reactive_stores_macro::Store, Serialize, Deserialize, Default)]
pub struct MetaStore {
    pub title: String,
    pub description: String,
    pub image: String,
    pub url: String,
}

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
    let meta_store = reactive_stores::Store::new(MetaStore::default());
    provide_context(meta_store);

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/ruche.css" />
        // sets the document title
        <Title text=move || meta_store.title().get() />
        <Meta name="color-scheme" content="dark light" />
        <Meta name="og:type" content="website" />
        <Meta name="og:site_name" content="Ruche" />
        <Meta name="robots" content="index,follow" />

        <Meta name="description" content=move || meta_store.description().get() />
        <Meta name="og:title" content=move || meta_store.title().get() />
        <Meta name="og:description" content=move || meta_store.description().get() />
        <Meta name="og:image" content=move || meta_store.image().get() />
        <Meta name="og:url" content=move || meta_store.url().get() />
        <Link rel="canonical" prop:href=move || format!("{}{}", SITE_URL, meta_store.url().get()) />
        <Link rel="icon" type_="image/x-icon" href="/assets/favicon.ico" />

        // content for this welcome page
        <Router>
            <main class="bg-gray-900 flex items-start justify-center min-h-screen w-full text-gray-200">
                <Routes
                    transition=true
                    fallback=|| view! { <div class="text-center">Page Not Found</div> }
                >
                    <Route
                        path=path!("")
                        view=move || view! { <Redirect path="platform/EUW" /> }
                    />
                    // app.rs
                    <ParentRoute
                      path=path!("platform/:platform_route")
                      view=PlatformTypePage
                    >
                      <Route path=path!("") view=move || view! {} />
                    
                      // Turn this into a parent route
                      <ParentRoute
                        path=path!("summoners/:summoner_slug")
                        view=SummonerPage
                      >
                        // index → Matches
                        <Route path=path!("") view=SummonerMatchesPage />
                    
                        <Route path=path!("champions") view=SummonerChampionsPage />
                    
                        <Route path=path!("encounters")view=SummonerEncountersPage/>
                    
                        <Route path=path!("live")
                               view=SummonerLivePage />
                    
                        <Route
                          path=path!("encounter/:encounter_platform/:encounter_slug")
                          view=SummonerEncounterPage
                        />
                      </ParentRoute>
                    </ParentRoute>

                </Routes>
            </main>
        </Router>
    }
}
