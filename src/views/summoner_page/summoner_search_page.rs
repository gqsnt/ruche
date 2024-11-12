use crate::backend::server_fns::search_summoner::SearchSummoner;
use crate::consts::platform_route::PLATFORM_ROUTE_OPTIONS;
use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::hooks::{use_params_map, use_query_map};

#[component]
pub fn SummonerSearchPage() -> impl IntoView {
    let query = use_query_map();
    let params = use_params_map();
    let find_summoner = ServerAction::<SearchSummoner>::new();


    let game_name = move || {
        query.read().get("game_name").unwrap_or_default()
    };
    let tag_line = move || {
        query.read().get("tag_line").unwrap_or_default()
    };
    let platform_type = move || {
        params.read().get("platform_type").unwrap_or_default()
    };


    view! {
        <div class="w-full flex justify-center">
            <ActionForm action=find_summoner>
                <div class="my-2 flex space-x-2 items-center max-w-[768px]">
                    <input
                        class="my-input"
                        type="text"
                        placeholder="Game Name"
                        value=move || game_name()
                        name="game_name"
                    />
                    <input
                        class="my-input"
                        type="text"
                        placeholder="Tag Line"
                        value=move || tag_line()
                        name="tag_line"
                    />
                    <select
                        class="my-select"
                        aria-label="Platform Type"
                        name="platform_type"
                        prop:value=move || platform_type()
                    >
                        {PLATFORM_ROUTE_OPTIONS
                            .iter()
                            .map(|pt| {
                                view! {
                                    <option
                                        value=pt.to_string()
                                        selected=move || { platform_type().eq(&pt.to_string()) }
                                    >
                                        {pt.to_string()}
                                    </option>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </select>
                    <button class="my-button" type="submit">
                        "Search"
                    </button>
                </div>
            </ActionForm>
        </div>
    }
}


