use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_encounters::get_encounters;
use crate::consts::profile_icon::ProfileIcon;
use crate::consts::HasStaticAsset;
use crate::utils::{summoner_encounter_url, summoner_url};
use crate::views::components::pagination::Pagination;
use crate::views::summoner_page::Summoner;
use crate::views::{BackEndMatchFiltersSearch};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::server_fn::rkyv::{Deserialize, Serialize, Archive};
use leptos::{component, view, IntoView};
use leptos_router::hooks::query_signal_with_options;
use leptos_router::NavigateOptions;

#[component]
pub fn SummonerEncountersPage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
    let match_filters_updated = expect_context::<RwSignal<BackEndMatchFiltersSearch>>();

    let (page_number, set_page_number) = query_signal_with_options::<i32>(
        "page",
        NavigateOptions {
            scroll: false,
            replace: true,
            ..Default::default()
        },
    );

    let (search_summoner, set_search_summoner) = query_signal_with_options::<String>(
        "q",
        NavigateOptions {
            scroll: false,
            replace: true,
            ..Default::default()
        },
    );
    let (search_summoner_signal, set_search_summoner_signal) = signal(search_summoner.get().unwrap_or_default());


    let (reset_page_number, set_reset_page_number) = signal::<bool>(false);
    Effect::new(move |_| {
        if reset_page_number() {
            set_page_number(None);
            set_reset_page_number(false);
        }
    });

    let encounters_resource = Resource::new_rkyv(
        move || (search_summoner.get(), match_filters_updated.get(), summoner(), page_number()),
        |(search_summoner, filters, summoner, page_number)| async move {
            //println!("{:?} {:?} {:?}", filters, summoner, page_number);
            get_encounters(summoner.id, page_number.unwrap_or(1) as u16, Some(filters), search_summoner).await
        },
    );

    meta_store.title().set(format!("{}#{} | Encounters | Broken.gg", summoner().game_name, summoner().tag_line));
    meta_store.description().set(format!("Discover the top champions played by {}#{}. Access in-depth statistics, win rates, and performance insights on Broken.gg, powered by Rust for optimal performance.", summoner().game_name, summoner().tag_line));
    meta_store.url().set(format!("{}?tab=encounters", summoner().to_route_path()));
    view! {
        <div>
            <div class="my-card flex space-x-2 my-2 w-fit">
                <input
                    type="text"
                    class="my-input"
                    placeholder="Search for a summoner"
                    prop:value=move || search_summoner_signal.get()
                    on:input=move |e| { set_search_summoner_signal(event_target_value(&e)) }
                />
                <button
                    class="my-button"
                    on:click=move |_| {
                        set_search_summoner(Some(search_summoner_signal.get()));
                    }
                >
                    Search
                </button>
                <button
                    class="my-button bg-red-700 hover:bg-red-800 text-gray-200"
                    on:click=move |_| {
                        set_search_summoner(None);
                        set_search_summoner_signal(String::new());
                    }
                >
                    Clear
                </button>
            </div>
            <Suspense fallback=move || {
                view! { <div class="text-center">Loading Encounters</div> }
            }>
                {move || Suspend::new(async move {
                    match encounters_resource.await {
                        Ok(encounters_result) => {
                            let total_pages = encounters_result.total_pages;
                            let current_page = page_number().unwrap_or(1);
                            if total_pages == 0 || (total_pages as i32) < current_page {
                                set_reset_page_number(true);
                            }
                            if !encounters_result.encounters.is_empty() {
                                Ok(
                                    Either::Left(
                                        view! {
                                            <div class="flex my-card w-fit">
                                                <table class="text-gray-200 space-y-2">
                                                    <thead>
                                                        <tr>
                                                            <th class="text-left px-2">Summoner</th>
                                                            <th class="px-2">With</th>
                                                            <th class=" px-2">Vs</th>
                                                            <th class=" px-2">Total</th>
                                                            <th class=" px-2"></th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        <For
                                                            each=move || encounters_result.encounters.clone()
                                                            key=|encounter| encounter.id
                                                            let:encounter
                                                        >
                                                            {
                                                                let encounter: SummonerEncountersSummoner = encounter;
                                                                let encounter_platform = encounter.platform.clone();
                                                                let encounter_game_name = encounter.game_name.clone();
                                                                let encounter_tag_line = encounter.tag_line.clone();
                                                                view! {
                                                                    <tr>
                                                                        <td class="text-left w-[200px]">
                                                                            <div class="flex items-center py-0.5">
                                                                                <div>
                                                                                    <img
                                                                                        alt="Profile Icon"
                                                                                        src=ProfileIcon::get_static_asset_url(
                                                                                            encounter.profile_icon_id,
                                                                                        )
                                                                                        class="w-8 h-8 rounded"
                                                                                        height="32"
                                                                                        width="32"
                                                                                    />
                                                                                </div>
                                                                                <div class="ml-2">
                                                                                    <a
                                                                                        href=summoner_url(
                                                                                            encounter.platform.clone().as_str(),
                                                                                            encounter.game_name.clone().as_str(),
                                                                                            encounter.tag_line.clone().as_str(),
                                                                                        )
                                                                                        class="text-blue-300 hover:underline"
                                                                                    >
                                                                                        {encounter.game_name.clone()}
                                                                                    </a>
                                                                                </div>
                                                                            </div>
                                                                        </td>
                                                                        <td class="px-2">
                                                                            {encounter.with_win_count}W
                                                                            {encounter.with_match_count - encounter.with_win_count}L
                                                                            <span class="mx-1">{encounter.with_match_count}G</span>
                                                                            {format!(
                                                                                "{}%",
                                                                                ((encounter.with_win_count as f64
                                                                                    / encounter.with_match_count as f64) * 100.0) as i32,
                                                                            )}
                                                                        </td>
                                                                        <td class="px-2">
                                                                            {encounter.vs_win_count}W
                                                                            {encounter.vs_match_count - encounter.vs_win_count}L
                                                                            <span class="mx-1">{encounter.vs_match_count}G</span>
                                                                            {format!(
                                                                                "{}%",
                                                                                ((encounter.vs_win_count as f64
                                                                                    / encounter.vs_match_count as f64) * 100.0) as i32,
                                                                            )}
                                                                        </td>
                                                                        <td class="px-2 ">{encounter.match_count}</td>
                                                                        <td class="px-2">
                                                                            <a
                                                                                class="my-button font-bold"
                                                                                href=summoner_encounter_url(
                                                                                    summoner().platform.to_string().as_str(),
                                                                                    summoner().game_name.as_str(),
                                                                                    summoner().tag_line.as_str(),
                                                                                    encounter_platform.as_str(),
                                                                                    encounter_game_name.as_str(),
                                                                                    encounter_tag_line.as_str(),
                                                                                )
                                                                            >
                                                                                >
                                                                            </a>
                                                                        </td>
                                                                    </tr>
                                                                }
                                                            }
                                                        </For>
                                                    </tbody>
                                                </table>
                                            </div>

                                            <Show when=move || (total_pages > 1)>
                                                <Pagination max_page=total_pages as usize />
                                            </Show>
                                        },
                                    ),
                                )
                            } else {
                                Ok(
                                    Either::Right(
                                        view! { <div class="text-center">No Encounters Found</div> },
                                    ),
                                )
                            }
                        }
                        Err(e) => Err(e),
                    }
                })}
            </Suspense>

        </div>
    }
}


#[derive(Clone, Serialize, Deserialize, Default, Archive)]
pub struct SummonerEncountersResult {
    pub encounters: Vec<SummonerEncountersSummoner>,
    pub total_pages: i64,
}


#[derive(Clone, Serialize, Deserialize, Archive)]
pub struct SummonerEncountersSummoner {
    pub id: i32,
    pub match_count: i64,
    pub with_match_count: i64,
    pub with_win_count: i64,
    pub vs_match_count: i64,
    pub vs_win_count: i64,
    pub game_name: String,
    pub tag_line: String,
    pub platform: String,
    pub profile_icon_id: u16,
}
