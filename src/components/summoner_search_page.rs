use crate::apis::FindSummoner;
use crate::consts::PlatformRoute;
use leptos::form::ActionForm;
use leptos::prelude::AriaAttributes;
use leptos::prelude::{ClassAttribute, ElementChild};
use leptos::prelude::{PropAttribute, Read};
use leptos::server::ServerAction;
use leptos::{component, view, IntoView};
use leptos_router::hooks::{use_params_map, use_query_map};
use strum::IntoEnumIterator;

#[component]
pub fn SummonerSearchPage() -> impl IntoView {
    let query = use_query_map();
    let params = use_params_map();
    let find_summoner = ServerAction::<FindSummoner>::new();


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
        <ActionForm action=find_summoner>
            <div class="my-2 flex space-x-2 items-center">
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
                    {PlatformRoute::iter()
                        .map(|pt| {
                            view! {
                                <option
                                    value=pt.as_region_str()
                                    selected=move || {
                                        platform_type().eq(&pt.as_region_str().to_string())
                                    }
                                >
                                    {pt.as_region_str()}
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
    }
}


