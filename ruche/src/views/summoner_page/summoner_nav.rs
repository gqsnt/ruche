use crate::views::components::match_filters::MatchFilters;
use crate::views::summoner_page::summoner_champions_page::SummonerChampionsPage;
use crate::views::summoner_page::summoner_encounter_page::SummonerEncounterPage;
use crate::views::summoner_page::summoner_encounters_page::SummonerEncountersPage;
use crate::views::summoner_page::summoner_live_page::SummonerLivePage;
use crate::views::summoner_page::summoner_matches_page::SummonerMatchesPage;

use crate::views::get_default_navigation_option;
use crate::views::summoner_page::SSEInLiveGame;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::hooks::query_signal_with_options;

#[component]
pub fn SummonerNav() -> impl IntoView {
    let sse_in_live_game = expect_context::<ReadSignal<SSEInLiveGame>>();

    let (tab, set_tab) =
        query_signal_with_options::<String>("tab", get_default_navigation_option());

    let (_, set_page_number) =
        query_signal_with_options::<u16>("page", get_default_navigation_option());

    let (_, set_encounter_slug) =
        query_signal_with_options::<String>("encounter_slug", get_default_navigation_option());

    let (_, set_encounter_platform) =
        query_signal_with_options::<String>("encounter_platform", get_default_navigation_option());
    let (_, set_encounter_search) =
        query_signal_with_options::<String>("q", get_default_navigation_option());

    let switch_tab = move |tab: Tabs| {
        set_page_number(None);
        set_encounter_slug(None);
        set_encounter_platform(None);
        set_encounter_search(None);
        set_tab(Some(tab.to_string()));
    };

    let tab_class = move |is_default: bool, tab_: Tabs| {
        if is_default || tab() == Some(tab_.to_string()) {
            "active-tab"
        } else {
            "default-tab"
        }
    };

    view! {
        <div class="flex justify-center">
            <nav class="w-[768px]">
                <ul class="flex justify-start space-x-2">
                    <li class="-mb-px">
                        <button
                            on:click=move |_| switch_tab(Tabs::Matches)
                            class=move || tab_class(tab().is_none(), Tabs::Matches)
                        >
                            Matches
                        </button>
                    </li>
                    <li class="-mb-px">
                        <button
                            on:click=move |_| switch_tab(Tabs::Champions)
                            class=move || tab_class(false, Tabs::Champions)
                        >
                            Champions
                        </button>
                    </li>
                    <li class="-mb-px">
                        <button
                            on:click=move |_| switch_tab(Tabs::Encounters)
                            class=move || tab_class(false, Tabs::Encounters)
                        >
                            Encounters
                        </button>
                    </li>
                    <li class="-mb-px">
                        <button
                            on:click=move |_| switch_tab(Tabs::Live)
                            class=move || {
                                if tab() == Some(Tabs::Live.to_string()) {
                                    "active-tab"
                                } else {
                                    if sse_in_live_game().0.is_some() {
                                        "default-ig-tab"
                                    } else {
                                        "default-tab"
                                    }
                                }
                            }
                        >
                            Live
                        </button>
                    </li>
                    <li class="-mb-px text-center">
                        <button class=move || {
                            if tab() == Some(Tabs::Encounter.to_string()) {
                                "active-tab"
                            } else {
                                "disabled-tab"
                            }
                        }>Encounter</button>
                    </li>
                </ul>
            </nav>
        </div>

        <div class="my-4 ">
            <Show when=move || tab().is_none() || tab() == Some(Tabs::Matches.to_string())>
                <MatchFilters>
                    <SummonerMatchesPage />
                </MatchFilters>
            </Show>
            <Show when=move || tab() == Some(Tabs::Champions.to_string())>
                <MatchFilters>
                    <SummonerChampionsPage />
                </MatchFilters>
            </Show>
            <Show when=move || tab() == Some(Tabs::Encounters.to_string())>
                <MatchFilters>
                    <SummonerEncountersPage />
                </MatchFilters>
            </Show>
            <Show when=move || tab() == Some(Tabs::Live.to_string())>
                <SummonerLivePage />
            </Show>
            <Show when=move || tab() == Some(Tabs::Encounter.to_string())>
                <MatchFilters>
                    <SummonerEncounterPage />
                </MatchFilters>
            </Show>

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
