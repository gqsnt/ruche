use crate::components::platform_type_page::PlatformTypePage;
use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use crate::components::summoner_champions_page::SummonerChampionsPage;
use crate::components::summoner_encounters_page::SummonerEncountersPage;
use crate::components::summoner_live_page::SummonerLivePage;
use crate::components::summoner_matches_page::SummonerMatchesPage;
use crate::components::summoner_page::SummonerPage;
use crate::components::summoner_search_page::SummonerSearchPage;
use crate::models::types::PlatformType;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    let platform_type = create_rw_signal(PlatformType::EUW1);
    provide_context(platform_type);

    let summoner_id = create_rw_signal(None::<i32>);
    provide_context(summoner_id);
    view! {


        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos-broken-gg.css"/>

        // sets the document title
        <Title text="Broken.gg"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
        }>
            <main >
                <Routes>
                    <Route path="" view=HomePage>
                        <Route path="" view=move ||view!{}/>
                        <Route path="/:platform_type" view=PlatformTypePage>
                            <Route path="" view=move ||view!{}/>
                            <Route path="/summoners/:summoner_slug" view=SummonerPage ssr=SsrMode::PartiallyBlocked>
                                <Route path="" view=move ||view!{}/>
                                <Route path="/matches" view=SummonerMatchesPage ssr=SsrMode::PartiallyBlocked/>
                                <Route path="/champions" view=SummonerChampionsPage />
                                <Route path="/encounters" view=SummonerEncountersPage />
                                <Route path="/live" view=SummonerLivePage />
                            </Route>
                        </Route>
                    </Route>
                </Routes>
            </main>
        </Router>
    }
}


/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <main class="my-0 mx-auto max-w-3xl text-center">
               <A href="/" class="p-6 text-4xl">"Welcome to Broken.gg"</A>
            <SummonerSearchPage/>
            <Outlet/>
        </main>
    }
}


