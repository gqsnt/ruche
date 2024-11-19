use crate::views::components::match_filters::MatchFilters;
use crate::views::summoner_page::summoner_champions_page::SummonerChampionsPage;
use crate::views::summoner_page::summoner_encounters_page::SummonerEncountersPage;
use crate::views::summoner_page::summoner_live_page::SummonerLivePage;
use crate::views::summoner_page::summoner_matches_page::SummonerMatchesPage;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::hooks::query_signal_with_options;
use leptos_router::NavigateOptions;
use crate::views::summoner_page::summoner_encounter_page::SummonerEncounterPage;

#[component]
pub fn SummonerNav() -> impl IntoView {
    let (tab, set_tab) = query_signal_with_options::<String>("tab", NavigateOptions {
        scroll: false,
        replace: true,
        ..Default::default()
    });

    view! {
        <div class="flex justify-center">
            <nav class="w-[768px]">
                <ul class="flex justify-start space-x-2">
                    <li class="-mb-px">
                        <button
                            on:click=move |_| set_tab(Some(Tabs::Matches.to_string()))
                            class=move || {
                                if tab().is_none() || tab() == Some(Tabs::Matches.to_string()) {
                                    "active-tab"
                                } else {
                                    "default-tab"
                                }
                            }
                        >
                            Matches
                        </button>
                    </li>
                    <li class="-mb-px">
                        <button
                            on:click=move |_| set_tab(Some(Tabs::Champions.to_string()))
                            class=move || {
                                if tab() == Some(Tabs::Champions.to_string()) {
                                    "active-tab"
                                } else {
                                    "default-tab"
                                }
                            }
                        >
                            Champions
                        </button>
                    </li>
                    <li class="-mb-px">
                        <button
                            on:click=move |_| set_tab(Some(Tabs::Encounters.to_string()))
                            class=move || {
                                if tab() == Some(Tabs::Encounters.to_string()) {
                                    "active-tab"
                                } else {
                                    "default-tab"
                                }
                            }
                        >
                            Encounters
                        </button>
                    </li>
                    <li class="-mb-px">
                        <button
                            on:click=move |_| set_tab(Some(Tabs::Live.to_string()))
                            class=move || {
                                if tab() == Some(Tabs::Live.to_string()) {
                                    "active-tab"
                                } else {
                                    "default-tab"
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
                        }>Encouter</button>
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

