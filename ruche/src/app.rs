use bitcode::{Decode, Encode};
use crate::views::summoner_search_page::SummonerSearchPage;
use crate::views::summoner_page::{SSEInLiveGame, SSEMatchUpdateVersion, SummonerPageRoute};
use leptos::config::LeptosOptions;

use leptos::prelude::*;
use leptos_router::params::{Params, ParamsError};
use leptos_meta::{provide_meta_context, Link, Meta, MetaTags, Stylesheet, Title};
use leptos_router::components::{ParentRoute, Redirect};
use leptos_router::{components::{Route, Router, Routes}, path, Lazy};
use reactive_stores::Store;
use serde::{Deserialize, Serialize};
use common::consts::platform_route::PlatformRoute;
use crate::utils::parse_summoner_slug;
use crate::views::BackEndMatchFiltersSearch;
use crate::views::summoner_page::summoner_champions_page::SummonerChampionsRoute;
use crate::views::summoner_page::summoner_encounter_page::{ SummonerEncounterRoute};
use crate::views::summoner_page::summoner_encounters_page::SummonerEncountersRoute;
use crate::views::summoner_page::summoner_live_page::SummonerLiveRoute;
use crate::views::summoner_page::summoner_matches_page::SummonerMatchesRoute;

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

    let sse_match_update_version = RwSignal::new(
        None::<SSEMatchUpdateVersion>,
    );
    let sse_in_live_game = RwSignal::new(
        SSEInLiveGame::default(),
    );

    let filters = Store::new(BackEndMatchFiltersSearch::default());
    provide_context(sse_match_update_version);
    provide_context(sse_in_live_game);
    provide_context(filters);

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
                        view=SummonerSearchPage
                    />

                      <ParentRoute
                        path=path!("summoners/:platform_route/:summoner_slug")
                        view={Lazy::<SummonerPageRoute>::new()}
                      >
                        <Route path=path!("") view=|| {view! { <Redirect path="matches" /> }} />
                        // index → Matches
                        <Route path=path!("matches") view={Lazy::<SummonerMatchesRoute>::new()} />
                        <Route path=path!("champions") view={Lazy::<SummonerChampionsRoute>::new()} />
                        <Route path=path!("encounters")view={Lazy::<SummonerEncountersRoute>::new()}/>
                        <Route path=path!("live") view={Lazy::<SummonerLiveRoute>::new()} />
                        <Route
                          path=path!("encounter/:encounter_platform_route/:encounter_slug")
                          view={Lazy::<SummonerEncounterRoute>::new()}
                        />
                      </ParentRoute>

                </Routes>
            </main>
        </Router>
    }
}



#[derive(Params, Debug, PartialEq, Clone)]
pub struct SummonerRouteParams {
    pub platform_route:Option<PlatformRoute>,
    pub summoner_slug:Option<String>,

}



pub fn to_summoner_identifier_memo(
    summoner_route_params: Memo<Result<SummonerRouteParams, ParamsError>>
) -> Memo<SummonerIdentifier>{
    Memo::new(move |_| {
        summoner_route_params.get()
            .ok()
            .and_then(|sr| {
                match (sr.summoner_slug, sr.platform_route){
                    (Some(ss), Some(platform_route)) => {
                        let (game_name, tag_line)  = parse_summoner_slug(&ss);
                        Some(
                            SummonerIdentifier{
                                game_name,
                                tag_line,
                                platform_route,
                            }
                        )
                    }
                    _ => None
                }
            })
            .unwrap()
    })
}


#[derive(Encode,Decode, PartialEq, Clone, Hash, Eq, Debug)]
pub struct SummonerIdentifier{
    pub game_name:String,
    pub tag_line:String,
    pub platform_route: PlatformRoute,
}





#[derive(Params, Debug, PartialEq, Clone)]
pub struct EncounterRouteParams{
    pub encounter_platform_route:Option<PlatformRoute>,
    pub encounter_slug:Option<String>,
}


pub fn to_encounter_identifier_memo(
    summoner_route_params: Memo<Result<EncounterRouteParams, ParamsError>>
) -> Memo<SummonerIdentifier>{
    Memo::new(move |_| {
        summoner_route_params.get()
            .ok()
            .and_then(|sr| {
                match (sr.encounter_slug, sr.encounter_platform_route){
                    (Some(ss), Some(platform_route)) => {
                        let (game_name, tag_line)  = parse_summoner_slug(&ss);
                        Some(
                            SummonerIdentifier{
                                game_name,
                                tag_line,
                                platform_route,
                            }
                        )
                    }
                    _ => None
                }
            })
            .unwrap()
    })
}


#[derive(Params, Debug, PartialEq, Clone)]
pub struct SummonerSearchQuery{
    pub game_name:Option<String>,
    pub tag_line:Option<String>,
    pub platform_route: Option<PlatformRoute>,
}




