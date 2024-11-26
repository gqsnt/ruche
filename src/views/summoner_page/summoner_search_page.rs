use crate::backend::server_fns::search_summoner::SearchSummoner;
use crate::consts::platform_route::{PlatformRoute, PLATFORM_ROUTE_OPTIONS};
use crate::utils::{GameName, TagLine};
use leptos::html::{Input, Select};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::hooks::{use_params_map, use_query_map};

#[component]
pub fn SummonerSearchPage() -> impl IntoView {
    let query = use_query_map();
    let params = use_params_map();
    let search_summoner = ServerAction::<SearchSummoner>::new();

    let game_name = move || query.read().get("game_name").unwrap_or_default();
    let tag_line = move || query.read().get("tag_line").unwrap_or_default();
    let platform_type = move || params.read().get("platform_type").unwrap_or_default();

    let game_name_node = NodeRef::<Input>::new();
    let tag_line_node = NodeRef::<Input>::new();
    let platform_type_node = NodeRef::<Select>::new();

    view! {
        <div class="w-full flex justify-center">
            <form on:submit:target=move |ev| {
                ev.prevent_default();
                search_summoner
                    .dispatch(SearchSummoner {
                        platform_route: PlatformRoute::from(
                            platform_type_node
                                .get()
                                .expect("platform_type not valid")
                                .value()
                                .as_str(),
                        ),
                        game_name: GameName::new(
                            game_name_node.get().expect("game_name not valid").value().as_str(),
                        ),
                        tag_line: TagLine::new(
                            tag_line_node.get().expect("tag_line not valid").value().as_str(),
                        ),
                    });
            }>
                <div class="my-2 flex space-x-2 items-center max-w-[768px]">
                    <input
                        class="my-input"
                        type="text"
                        node_ref=game_name_node
                        placeholder="Game Name"
                        value=game_name
                        name="game_name"
                    />
                    <input
                        class="my-input"
                        type="text"
                        node_ref=tag_line_node
                        placeholder="Tag Line"
                        value=tag_line
                        name="tag_line"
                    />
                    <select
                        class="my-select"
                        aria-label="Platform Type"
                        node_ref=platform_type_node
                        name="platform_type"
                        prop:value=platform_type
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
            </form>
        </div>
    }
}
