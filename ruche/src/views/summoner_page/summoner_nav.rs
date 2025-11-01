use crate::app::EncounterRouteParams;
use crate::utils::{SSEVersions, SSEVersionsStoreFields};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::components::A;
use leptos_router::hooks::use_params;
use reactive_stores::Store;

#[component]
pub fn SummonerNav() -> impl IntoView {
    let sse_in_live_game = expect_context::<Store<SSEVersions>>();
    let encounter_params = use_params::<EncounterRouteParams>();
    let is_encounter_route = move || {
        encounter_params
            .get()
            .ok()
            .map(|i| i.encounter_platform_route.is_some())
            .unwrap_or_default()
    };
    view! {
        <div class="flex justify-center">
            <nav class="w-[768px] summoner-nav" aria-label="Summoner navigation">
                <ul class="flex justify-start space-x-2">
                    <li>
                        <A href="matches" attr:class="tab">
                            "Matches"
                        </A>
                    </li>
                    <li>
                        <A href="champions" attr:class="tab">
                            "Champions"
                        </A>
                    </li>
                    <li>
                        <A href="encounters" attr:class="tab">
                            "Encounters"
                        </A>
                    </li>
                    <li>
                        <A
                            href="live"
                            // highlight via CSS when data-live="1"
                            attr:data-live=move || sse_in_live_game.live_ver().get().map(|_| "1")
                            attr:class="tab"
                        >
                            "Live"
                        </A>
                    </li>
                    <li>
                        <Show
                            when=move || is_encounter_route()
                            fallback=|| {
                                view! {
                                    <span class="tab" aria-disabled="true">
                                        "Encounter"
                                    </span>
                                }
                            }
                        >
                            <span class="tab" aria-current="page">
                                "Encounter"
                            </span>
                        </Show>
                    </li>
                </ul>
            </nav>
        </div>
    }
}
