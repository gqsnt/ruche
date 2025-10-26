use bitcode::{Decode, Encode};
use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_encounters::get_encounters;
use crate::utils::{
    calculate_loss_and_win_rate, format_float_to_2digits, summoner_encounter_url, summoner_url,
};
use crate::views::components::pagination::Pagination;
use crate::views::summoner_page::{SSEMatchUpdateVersion, Summoner};
use crate::views::{
    get_default_navigation_option, BackEndMatchFiltersSearch, ImgSrc, PendingLoading,
};
use common::consts::platform_route::PlatformRoute;
use common::consts::profile_icon::ProfileIcon;
use common::consts::HasStaticSrcAsset;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::components::A;
use leptos_router::hooks::query_signal_with_options;

#[component]
pub fn SummonerEncountersPage() -> impl IntoView {
    let summoner = expect_context::<Summoner>();
    let sse_match_update_version = expect_context::<ReadSignal<Option<SSEMatchUpdateVersion>>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
    let match_filters_updated = expect_context::<RwSignal<BackEndMatchFiltersSearch>>();

    let (page_number, set_page_number) =
        query_signal_with_options::<u16>("page", get_default_navigation_option());
    let (search_summoner, set_search_summoner) =
        query_signal_with_options::<String>("q", get_default_navigation_option());
    let (search_summoner_signal, set_search_summoner_signal) =
        signal(search_summoner.get_untracked().unwrap_or_default());

    let (pending, set_pending) = signal(false);

    let (reset_page_number, set_reset_page_number) = signal(false);
    Effect::new(move |_| {
        if reset_page_number() {
            set_page_number(None);
            set_reset_page_number(false);
        }
    });

    let encounters_resource = Resource::new_bitcode(
        move || {
            (
                sse_match_update_version.get().unwrap_or_default(),
                search_summoner.get(),
                match_filters_updated.get(),
                summoner.id,
                page_number(),
                set_pending,
            )
        },
        |(_, search_summoner, filters, summoner_id, page_number, set_pending_value)| async move {
            //println!("{:?} {:?} {:?}", filters, summoner.unwrap(), page_number);
            let r = get_encounters(
                summoner_id,
                page_number.unwrap_or(1),
                search_summoner,
                Some(filters),
            )
            .await;
            set_pending_value(false);
            r
        },
    );

    meta_store.title().set(format!(
        "{}#{} | Encounters | Ruche",
        summoner.game_name.as_str(),
        summoner.tag_line.as_str()
    ));
    meta_store.description().set(format!("Discover the top champions played by {}#{}. Access in-depth statistics, win rates, and performance insights on Ruche, powered by Rust for optimal performance.", summoner.game_name.as_str(), summoner.tag_line.as_str()));
    meta_store
        .url()
        .set(format!("{}/encounters", summoner.to_route_path()));

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
                    class="my-button flex items-center"
                    on:click=move |_| {
                        set_pending(true);
                        set_search_summoner(Some(search_summoner_signal.get()));
                    }
                >
                    <PendingLoading pending>Search</PendingLoading>
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
            <Transition fallback=move || {
                view! { <div class="text-center">Loading Encounters</div> }
            }>
                {move || {
                    let summoner_clone = summoner.clone();
                    Suspend::new(async move {
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

                                                                    let summoner_encounter_url = summoner_encounter_url(
                                                                                        summoner_clone.platform.code(),
                                                                                        summoner_clone.game_name.as_str(),
                                                                                        summoner_clone.tag_line.as_str(),
                                                                                        encounter.platform.code(),
                                                                                        encounter.game_name.to_string().as_str(),
                                                                                        encounter.tag_line.as_str(),
                                                                                    );

                                                                    view! {
                                                                        <tr>
                                                                            <td class="text-left w-[200px]">
                                                                                <div class="flex items-center py-0.5">
                                                                                    <div>
                                                                                        <ImgSrc
                                                                                            alt=profile_icon.to_string()
                                                                                            src=profile_icon.get_static_asset_url()
                                                                                            class="w-8 h-8 rounded".to_string()
                                                                                            height=32
                                                                                            width=32
                                                                                        />
                                                                                    </div>
                                                                                    <div class="ml-2">
                                                                                        <A
                                                                                            href=summoner_url(
                                                                                                encounter.platform.code(),
                                                                                                encounter.game_name.clone().as_str(),
                                                                                                encounter.tag_line.as_str(),
                                                                                            )
                                                                                            attr:class="text-blue-300 hover:underline"
                                                                                        >
                                                                                            {encounter.game_name.to_string()}
                                                                                        </A>
                                                                                    </div>
                                                                                </div>
                                                                            </td>
                                                                            <td class="px-2">
                                                                                {format!(
                                                                                    "{}W {}L {}G {}%",
                                                                                    encounter.with_win_count,
                                                                                    with_losses as u16,
                                                                                    encounter.with_match_count,
                                                                                    format_float_to_2digits(with_winrate),
                                                                                )}
                                                                            </td>
                                                                            <td class="px-2">
                                                                                {format!(
                                                                                    "{}W {}L {}G {}%",
                                                                                    encounter.vs_win_count,
                                                                                    vs_losses as u16,
                                                                                    vs_match_count,
                                                                                    format_float_to_2digits(vs_winrate),
                                                                                )}
                                                                            </td>
                                                                            <td class="px-2 ">{encounter.match_count}</td>
                                                                            <td class="px-2">
                                                                                <A
                                                                                    attr:class="my-button font-bold"
                                                                                    href=summoner_encounter_url
                                                                                >
                                                                                    >
                                                                                </A>
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
                    })
                }}
            </Transition>

        </div>
    }
}

#[derive(Clone, Default, Encode,Decode)]
pub struct SummonerEncountersResult {
    pub total_pages: u16,
    pub encounters: Vec<SummonerEncountersSummoner>,
}

#[derive(Clone,  Encode,Decode)]
pub struct SummonerEncountersSummoner {
    pub id: i32,
    pub match_count: u16,
    pub with_match_count: u16,
    pub with_win_count: u16,
    pub vs_win_count: u16,
    pub profile_icon_id: u16,
    pub game_name: String,
    pub tag_line: String,
    pub platform: PlatformRoute,
}
