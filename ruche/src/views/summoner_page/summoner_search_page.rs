use crate::app::SummonerSearchQuery;
use crate::backend::server_fns::search_summoner::SearchSummoner;
use crate::views::PendingLoading;
use common::consts::platform_route::{PlatformRoute, PLATFORM_ROUTE_OPTIONS};
use leptos::ev::SubmitEvent;
use leptos::html::{Input, Select};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::hooks::use_query;

#[component]
pub fn SummonerSearch(is_summoner_page: bool) -> impl IntoView {
    let search_summoner = ServerAction::<SearchSummoner>::new();
    let pending = RwSignal::new(false);

    let summoner_search_query = use_query::<SummonerSearchQuery>();
    let game_name = move || {
        summoner_search_query
            .read()
            .as_ref()
            .ok()
            .and_then(|q| q.game_name.clone())
            .unwrap_or_default()
    };
    let tag_line = move || {
        summoner_search_query
            .read()
            .as_ref()
            .ok()
            .and_then(|q| q.tag_line.clone())
            .unwrap_or_default()
    };
    let platform_type = move || {
        summoner_search_query
            .read()
            .as_ref()
            .ok()
            .and_then(|p| p.platform_route)
            .unwrap_or_default()
            .code()
    };

    let game_name_node = NodeRef::<Input>::new();
    let tag_line_node = NodeRef::<Input>::new();
    let platform_type_node = NodeRef::<Select>::new();

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        pending.set(true);
        search_summoner.dispatch(SearchSummoner {
            platform_route: PlatformRoute::try_from(
                platform_type_node
                    .get()
                    .expect("platform_type not valid")
                    .value()
                    .as_str(),
            )
            .unwrap_or_default(),
            game_name: game_name_node.get().expect("game_name not valid").value(),
            tag_line: tag_line_node.get().expect("tag_line not valid").value(),
        });
    };

    Effect::new(move |_| {
        let _ = search_summoner.version().get();
        pending.set(false);
    });

    view! {
        <div class=" w-full flex my-2 justify-center">
            <form
                on:submit=on_submit
                class=("justify-center", !is_summoner_page)
                class="flex space-x-2 items-center w-[768px]"
            >
                {is_summoner_page
                    .then(|| {
                        view! {
                            <img
                                src="/assets/favicon.ico"
                                height="38"
                                class="mr-2 h-[38px] w-[38px]"
                                alt="logo"
                            />
                        }
                    })}
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
                    aria-label="Platform Route"
                    node_ref=platform_type_node
                    name="platform_route"
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
