use crate::views::summoner_page::{SSEInLiveGame, Summoner};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::components::A;
use leptos_router::hooks::{ use_location, use_params_map, use_query_map};

#[component]
pub fn SummonerNav() -> impl IntoView {
    let sse_in_live_game = expect_context::<ReadSignal<SSEInLiveGame>>();
    let summoner = expect_context::<Summoner>();
    let location = use_location();
    let query = use_query_map();
    let params = use_params_map();

    let base = Memo::new(move |_|summoner.to_route_path()); // e.g. /platform/EUW/summoners/slug

    // Build ?filters[...] query string to preserve across tabs
    let filters_qs = Memo::new(move |_| {
        let q = query.read();
        let mut parts = Vec::new();
        for (k, v) in q.clone().into_iter() {
            if k.starts_with("filters[") { parts.push(format!("{}={}", k, v)); }
        }
        if parts.is_empty() { String::new() } else { format!("?{}", parts.join("&")) }
    });

    // Active helpers
    let path = move || location.pathname.get();
    let is_matches   = move || path() == base.get() || path() == format!("{}/", base.read());
    let is_champions = move || path().starts_with(&format!("{}/champions", base.read()));
    let is_encounters= move || path().starts_with(&format!("{}/encounters", base.read()));
    let is_live      = move || path().starts_with(&format!("{}/live", base.read()));
    let is_encounter = move || params.read().get("encounter_slug").is_some();

    let tab_class = move |active: bool| if active { "active-tab" } else { "default-tab" };

    view! {
        <div class="flex justify-center">
            <nav class="w-[768px]">
                <ul class="flex justify-start space-x-2">
                    <li>
                        <A
                            href=move || format!("{}{}", base.read(), filters_qs())
                            attr:class=move || tab_class(is_matches())
                        >
                            "Matches"
                        </A>
                    </li>
                    <li>
                        <A
                            href=move || format!("{}/champions{}", base.read(), filters_qs())
                            attr:class=move || tab_class(is_champions())
                        >
                            "Champions"
                        </A>
                    </li>
                    <li>
                        <A
                            href=move || format!("{}/encounters{}", base.read(), filters_qs())
                            attr:class=move || tab_class(is_encounters())
                        >
                            "Encounters"
                        </A>
                    </li>
                    <li>
                        <A
                            href=move || format!("{}/live", base.read())
                            attr:class=move || {
                                if is_live() {
                                    "active-tab"
                                } else if sse_in_live_game().0.is_some() {
                                    "default-ig-tab"
                                } else {
                                    "default-tab"
                                }
                            }
                        >
                            "Live"
                        </A>
                    </li>
                    <li>
                        <button class=move || {
                            if is_encounter() { "active-tab" } else { "disabled-tab" }
                        }>"Encounter"</button>
                    </li>
                </ul>
            </nav>
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Tabs {
    #[default]
    Matches,
    Champions,
    Encounters,
    Live,
    Encounter,
}

impl std::fmt::Display for Tabs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tabs::Matches => write!(f, "matches"),
            Tabs::Champions => write!(f, "champions"),
            Tabs::Encounters => write!(f, "encounters"),
            Tabs::Live => write!(f, "live"),
            Tabs::Encounter => write!(f, "encounter"),
        }
    }
}
