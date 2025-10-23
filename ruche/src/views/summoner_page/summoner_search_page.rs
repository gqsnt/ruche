use crate::backend::server_fns::search_summoner::SearchSummoner;
use crate::views::PendingLoading;
use common::consts::platform_route::{PlatformRoute, PLATFORM_ROUTE_OPTIONS};
use leptos::ev::SubmitEvent;
use leptos::html::{Input, Select};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::hooks::{use_params_map, use_query_map};

#[component]
pub fn SummonerSearchPage(is_summoner_page: Signal<bool>) -> impl IntoView {
    let query = use_query_map();
    let params = use_params_map();
    let search_summoner = ServerAction::<SearchSummoner>::new();
    let (pending, set_pending) = signal(false);

    let game_name = move || query.read().get("game_name").unwrap_or_default();
    let tag_line = move || query.read().get("tag_line").unwrap_or_default();
    let platform_type = move || {
        params
            .read()
            .get("platform_type")
            .map(|k|PlatformRoute::from_code(&k).unwrap().code())
            .unwrap_or(PlatformRoute::EUW.code())
    };

    let game_name_node = NodeRef::<Input>::new();
    let tag_line_node = NodeRef::<Input>::new();
    let platform_type_node = NodeRef::<Select>::new();

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_pending(true);
        search_summoner.dispatch(SearchSummoner {
            platform_route: PlatformRoute::try_from(
                platform_type_node
                    .get()
                    .expect("platform_type not valid")
                    .value()
                    .as_str(),
            ).unwrap_or_default(),
            game_name: game_name_node.get().expect("game_name not valid").value(),
            tag_line: tag_line_node.get().expect("tag_line not valid").value(),
        });
    };

    Effect::new(move |_| {
        let _ = search_summoner.version().get();
        set_pending(false);
    });

    view! {
        <div class=" w-full flex my-2 justify-center">
            <form
                on:submit=on_submit
                class=("justify-center", move || !is_summoner_page())
                class="flex space-x-2 items-center w-[768px]"
            >
                {move || {
                    is_summoner_page()
                        .then(|| {
                            view! {
                                <img
                                    src="/assets/favicon.ico"
                                    height="38"
                                    class="mr-2 h-[38px] w-[38px]"
                                    alt="logo"
                                />
                            }
                        })
                }}
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
                                    value=pt.code()
                                    selected=move || { platform_type().eq(pt.code()) }
                                >
                                    {pt.code()}
                                </option>
                            }
                        })
                        .collect::<Vec<_>>()}
                </select>
                <button class="my-button flex items-center" type="submit">
                    <PendingLoading pending>Search</PendingLoading>
                </button>
            </form>
        </div>
    }
}
