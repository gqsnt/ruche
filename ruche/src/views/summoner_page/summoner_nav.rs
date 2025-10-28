use crate::views::summoner_page::{SSEInLiveGame, Summoner};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::components::A;
use leptos_router::hooks::{  use_params_map, use_query_map};

#[component]
pub fn SummonerNav() -> impl IntoView {
    let sse_in_live_game = expect_context::<ReadSignal<SSEInLiveGame>>();
    let summoner = expect_context::<ReadSignal<Summoner>>();
    let query = use_query_map();
    let params = use_params_map();

    let base = Memo::new(move |_|summoner.read().to_route_path()); // e.g. /platform/EUW/summoners/slug

    // Build ?filters[...] query string to preserve across tabs
    let filters_qs = Memo::new(move |_| {
        let q = query.read();
        let mut parts = Vec::new();
        for (k, v) in q.clone().into_iter() {
            if k.starts_with("filters[") { parts.push(format!("{}={}", k, v)); }
        }
        if parts.is_empty() { String::new() } else { format!("?{}", parts.join("&")) }
    });

    let encounter_href = Memo::new(move |_| {
        let p = params.read();
        match (p.get("encounter_platform_route"), p.get("encounter_slug")) {
            (Some(platform), Some(slug)) => {
                Some(format!("{}/encounter/{}/{}", base.read(), platform, slug))
            }
            _ => None,
        }
    });
    view! {
        <div class="flex justify-center">
            <nav class="w-[768px] summoner-nav" aria-label="Summoner navigation">
                <ul class="flex justify-start space-x-2">
                    <li>
                        <A
                            href=move || format!("{}{}", base.read(), filters_qs())
                            exact=true
                            attr:class="tab"
                        >
                            "Matches"
                        </A>
                    </li>
                    <li>
                        <A
                            href=move || format!("{}/champions{}", base.read(), filters_qs())
                            attr:class="tab"
                        >
                            "Champions"
                        </A>
                    </li>
                    <li>
                        <A
                            href=move || format!("{}/encounters{}", base.read(), filters_qs())
                            attr:class="tab"
                        >
                            "Encounters"
                        </A>
                    </li>
                    <li>
                        <A
                            href=move || format!("{}/live", base.read())
                            // highlight via CSS when data-live="1"
                            attr:data-live=move || sse_in_live_game().0.map(|_| "1")
                            attr:class="tab"
                        >
                            "Live"
                        </A>
                    </li>
                    <li>
                        {move || match encounter_href.get() {
                            Some(href) => {
                                view! {
                                    <A href=href attr:class="tab">
                                        "Encounter"
                                    </A>
                                }
                                    .into_any()
                            }
                            None => {
                                view! {
                                    <span class="tab" aria-disabled="true">
                                        "Encounter"
                                    </span>
                                }
                                    .into_any()
                            }
                        }}
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
