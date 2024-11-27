use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_encounters::get_encounters;
use crate::consts::platform_route::PlatformRoute;
use crate::consts::profile_icon::ProfileIcon;
use crate::consts::HasStaticAsset;
use crate::utils::{
    calculate_loss_and_win_rate, format_float_to_2digits, summoner_encounter_url, summoner_url,
    GameName, TagLine,
};
use crate::views::components::pagination::Pagination;
use crate::views::summoner_page::Summoner;
use crate::views::BackEndMatchFiltersSearch;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::server_fn::rkyv::{Archive, Deserialize, Serialize};
use leptos::{component, view, IntoView};
use leptos_router::hooks::query_signal_with_options;
use leptos_router::NavigateOptions;

#[component]
pub fn SummonerEncountersPage(summoner: Summoner) -> impl IntoView {
    let summoner_update_version = expect_context::<ReadSignal<Option<u16>>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
    let match_filters_updated = expect_context::<RwSignal<BackEndMatchFiltersSearch>>();

    let (page_number, set_page_number) = query_signal_with_options::<u16>(
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
    let (search_summoner_signal, set_search_summoner_signal) =
        signal(search_summoner.get().unwrap_or_default());

    let (reset_page_number, set_reset_page_number) = signal::<bool>(false);
    Effect::new(move |_| {
        if reset_page_number() {
            set_page_number(None);
            set_reset_page_number(false);
        }
    });

    let encounters_resource = Resource::new_rkyv(
        move || {
            (
                summoner_update_version.get().unwrap_or_default(),
                search_summoner.get(),
                match_filters_updated.get(),
                summoner.id,
                page_number(),
            )
        },
        |(_, search_summoner, filters, summoner_id, page_number)| async move {
            //println!("{:?} {:?} {:?}", filters, summoner.unwrap(), page_number);
            get_encounters(
                summoner_id,
                page_number.unwrap_or(1),
                search_summoner.map(|r| GameName::new(r.as_str())),
                Some(filters),
            )
            .await
        },
    );

    meta_store.title().set(format!(
        "{}#{} | Encounters | Broken.gg",
        summoner.game_name.to_str(),
        summoner.tag_line.to_str()
    ));
    meta_store.description().set(format!("Discover the top champions played by {}#{}. Access in-depth statistics, win rates, and performance insights on Broken.gg, powered by Rust for optimal performance.", summoner.game_name.to_str(), summoner.tag_line.to_str()));
    meta_store
        .url()
        .set(format!("{}?tab=encounters", summoner.to_route_path()));
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
                            if total_pages == 0 || total_pages < current_page {
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
                                                                let encounter_platform = encounter.platform.to_string();
                                                                let encounter_game_name = encounter.game_name.to_string();
                                                                let encounter_tag_line = encounter.tag_line.to_string();
                                                                let profile_icon = ProfileIcon(encounter.profile_icon_id);
                                                                let vs_match_count = encounter.match_count
                                                                    - encounter.with_match_count;
                                                                let (vs_losses, vs_winrate) = calculate_loss_and_win_rate(
                                                                    encounter.vs_win_count,
                                                                    vs_match_count,
                                                                );
                                                                let (with_losses, with_winrate) = calculate_loss_and_win_rate(
                                                                    encounter.with_win_count,
                                                                    encounter.with_match_count,
                                                                );
                                                                view! {
                                                                    <tr>
                                                                        <td class="text-left w-[200px]">
                                                                            <div class="flex items-center py-0.5">
                                                                                <div>
                                                                                    <img
                                                                                        alt=profile_icon.to_string()
                                                                                        src=profile_icon.get_static_asset_url()
                                                                                        class="w-8 h-8 rounded"
                                                                                        height="32"
                                                                                        width="32"
                                                                                    />
                                                                                </div>
                                                                                <div class="ml-2">
                                                                                    <a
                                                                                        href=summoner_url(
                                                                                            encounter_platform.clone(),
                                                                                            encounter_game_name.clone(),
                                                                                            encounter_tag_line.clone(),
                                                                                        )
                                                                                        class="text-blue-300 hover:underline"
                                                                                    >
                                                                                        {encounter.game_name.to_string()}
                                                                                    </a>
                                                                                </div>
                                                                            </div>
                                                                        </td>
                                                                        <td class="px-2">
                                                                            {encounter.with_win_count}W {with_losses as u16}L
                                                                            <span class="mx-1">{encounter.with_match_count}G</span>
                                                                            {format_float_to_2digits(with_winrate)}%
                                                                        </td>
                                                                        <td class="px-2">
                                                                            {encounter.vs_win_count}W {vs_losses as u16}L
                                                                            <span class="mx-1">{vs_match_count}G</span>
                                                                            {format_float_to_2digits(vs_winrate)}%
                                                                        </td>
                                                                        <td class="px-2 ">{encounter.match_count}</td>
                                                                        <td class="px-2">
                                                                            <a
                                                                                class="my-button font-bold"
                                                                                href=summoner_encounter_url(
                                                                                    summoner.platform.to_string(),
                                                                                    summoner.game_name.to_string(),
                                                                                    summoner.tag_line.to_string(),
                                                                                    encounter_platform.clone(),
                                                                                    encounter_game_name.clone(),
                                                                                    encounter_tag_line.clone(),
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
                                                <Pagination max_page=total_pages />
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
    pub total_pages: u16,
    pub encounters: Vec<SummonerEncountersSummoner>,
}

#[derive(Clone, Serialize, Deserialize, Archive)]
pub struct SummonerEncountersSummoner {
    pub id: i32,
    pub match_count: u16,
    pub with_match_count: u16,
    pub with_win_count: u16,
    pub vs_win_count: u16,
    pub profile_icon_id: u16,
    pub game_name: GameName,
    pub tag_line: TagLine,
    pub platform: PlatformRoute,
}
