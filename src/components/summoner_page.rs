use crate::apis::{get_summoner, UpdateSummoner};
use crate::components::match_filters::MatchFilters;
use crate::components::summoner_champions_page::SummonerChampionsPage;
use crate::components::summoner_encounters_page::SummonerEncountersPage;
use crate::components::summoner_live_page::SummonerLivePage;
use crate::components::summoner_matches_page::SummonerMatchesPage;
use leptos::context::provide_context;
use leptos::either::Either;
use leptos::prelude::{log, OnAttribute};
use leptos::prelude::{signal, Children, ElementChild, ElementExt, Show};
use leptos::prelude::{ActionForm, ClassAttribute, Get, Read, ServerAction, Suspend, Transition};
use leptos::server::Resource;
use leptos::{component, view, IntoView};
use leptos_router::hooks::{query_signal, query_signal_with_options, use_location, use_params_map};
use leptos_router::NavigateOptions;

#[component]
pub fn SummonerPage() -> impl IntoView {
    let update_summoner_action = ServerAction::<UpdateSummoner>::new();
    let params = use_params_map();
    let platform_type = move || {
        params.read().get("platform_type").clone().unwrap_or_default()
    };
    let summoner_slug = move || {
        params.read().get("summoner_slug").clone().unwrap_or_default()
    };


    // Update the summoner signal when resource changes
    let summoner_resource = Resource::new_blocking(
        move || (update_summoner_action.version().get(), platform_type(), summoner_slug()),
        |(_, pt, ss)| async move {
            //log!("Client::Fetching summoner: {}", ss);
            get_summoner(pt, ss).await
        },
    );



    let summoner_view = move || {
        Suspend::new(async move {
            match summoner_resource.await {
                Ok(summoner) => {
                    Either::Left({

                        let (summoner_signal, _) = signal(summoner.clone());
                        provide_context(summoner_signal);
                        provide_context(update_summoner_action.version());
                        view! {
                            <div class="flex justify-between">
                                <div class="flex justify-center items-center mt-2">
                                    <img
                                        src=format!(
                                            "/assets/profile_icons/{}.webp",
                                            summoner_signal().profile_icon_id,
                                        )
                                        class="w-16 h-16"
                                    />
                                    <div class="flex flex-col items-start">
                                        <div>
                                            {summoner_signal().game_name}#{summoner_signal().tag_line}
                                        </div>
                                        <div>lvl. {summoner_signal().summoner_level}</div>
                                    </div>
                                    <ActionForm action=update_summoner_action>
                                        <input
                                            type="hidden"
                                            name="id"
                                            value=move || summoner_signal().id
                                        />
                                        <input
                                            type="hidden"
                                            name="puuid"
                                            value=move || summoner_signal().puuid.clone()
                                        />
                                        <input
                                            type="hidden"
                                            name="platform_type"
                                            value=move || summoner_signal().platform.as_region_str()
                                        />
                                        <button class="ml-2 bg-green-500 px-3 py-1" type="submit">
                                            Update
                                        </button>
                                    </ActionForm>
                                </div>
                            </div>
                            <SummonerNav />
                        }
                    })
                }
                Err(_) => { Either::Right(view! {}) }
            }
        })
    };

    view! {
        <Transition fallback=move || {
            view! { <div>"Loading summoner ..."</div> }
        }>{summoner_view}</Transition>
    }
}


#[component]
pub fn SummonerNav() -> impl IntoView {
    let (tab, set_tab) = query_signal_with_options::<String>("tab", NavigateOptions {
        scroll: false,
        replace: true,
        ..Default::default()
    });

    view! {
        <nav>
            <ul class="flex ">
                <li class="-mb-px mr-1">
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
                <li class="-mb-px mr-1">
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
                <li class="-mb-px mr-1">
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
                <li class="-mb-px mr-1">
                    <button
                        on:click=move |e| set_tab(Some(Tabs::Live.to_string()))
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
            </ul>
        </nav>
        <div class="my-4">
            <Show when=move || tab().is_none() || tab() == Some(Tabs::Matches.to_string())>
                <MatchFilters>
                    <SummonerMatchesPage />
                </MatchFilters>
            </Show>
            <Show when=move || tab() == Some(Tabs::Champions.to_string())>
                <SummonerChampionsPage />
            </Show>
            <Show when=move || tab() == Some(Tabs::Encounters.to_string())>
                <SummonerEncountersPage />
            </Show>
            <Show when=move || tab() == Some(Tabs::Live.to_string())>
                <SummonerLivePage />
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
}

impl Tabs {
    pub fn to_string(&self) -> String {
        match self {
            Tabs::Matches => "matches".to_string(),
            Tabs::Champions => "champions".to_string(),
            Tabs::Encounters => "encounters".to_string(),
            Tabs::Live => "live".to_string(),
        }
    }
}

