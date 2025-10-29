use crate::app::{to_summoner_identifier_memo, SummonerRouteParams};
use crate::backend::server_fns::get_encounters::get_encounters;
use crate::utils::{calculate_loss_and_win_rate, format_float_to_2digits, summoner_encounter_url, summoner_url};
use crate::views::components::pagination::Pagination;
use crate::views::summoner_page::SSEMatchUpdateVersion;
use crate::views::{BackEndMatchFiltersSearch, BackEndMatchFiltersSearchStoreFields, ImgSrc, PendingLoading};
use bitcode::{Decode, Encode};
use common::consts::platform_route::PlatformRoute;
use common::consts::profile_icon::ProfileIcon;
use common::consts::HasStaticSrcAsset;
use leptos::either::Either;
use leptos::prelude::codee::binary::BitcodeCodec;
use leptos::prelude::*;
use leptos::view;
use leptos_router::components::A;
use leptos_router::hooks::use_params;
use leptos_router::{lazy_route, LazyRoute};
use reactive_stores::Store;

pub struct SummonerEncountersRoute {
    encounters_resource: Resource<Result<SummonerEncountersResult, ServerFnError>, BitcodeCodec>,
    pending: RwSignal<bool>,
    search_summoner: RwSignal<Option<String>>,
    match_filters: Store<BackEndMatchFiltersSearch>,
}

#[lazy_route]
impl LazyRoute for SummonerEncountersRoute {
    fn data() -> Self {
        let summoner_route_params = use_params::<SummonerRouteParams>();
        let summoner_identifier_memo = to_summoner_identifier_memo(
            summoner_route_params
        );
        let sse_match_update_version = expect_context::<RwSignal<Option<SSEMatchUpdateVersion>>>();
        let match_filters = expect_context::<Store<BackEndMatchFiltersSearch>>();

        let search_summoner = RwSignal::new(None::<String>);
        let pending = RwSignal::new(false);
        let encounters_resource = Resource::new_bitcode(
            move || {
                (
                    sse_match_update_version.get().unwrap_or_default(),
                    search_summoner.get(),
                    match_filters.get(),
                    summoner_identifier_memo.get(),
                    pending,
                )
            },
            |(_, search_summoner, filters, summoner_identifier, pending)| async move {
                //println!("{:?} {:?} {:?}", filters, summoner.unwrap(), page_number);
                let r = get_encounters(
                    summoner_identifier,
                    search_summoner,
                    Some(filters),
                )
                    .await;
                pending.set(false);
                r
            },
        );
        Self {
            encounters_resource,
            pending,
            search_summoner,
            match_filters,
        }
    }

    fn view(this: Self) -> AnyView {
        let SummonerEncountersRoute { encounters_resource, pending, search_summoner, match_filters } = this;


        let to_opt_string = |v: String| if v.is_empty() { None } else { Some(v) };

        let (local_search_summoner, set_local_search_summoner) = signal("".to_string());


        //         let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
        //
        //         batch(|| {
        //         meta_store.title().set(format!(
        //             "{}#{} | Encounters | Ruche",
        //             summoner.read().game_name.as_str(),
        //             summoner.read().tag_line.as_str()
        //         ));
        //         meta_store.description().set(format!("Discover the top champions played by {}#{}. Access in-depth statistics, win rates, and performance insights on Ruche, powered by Rust for optimal performance.", summoner.read().game_name.as_str(), summoner.read().tag_line.as_str()));
        //         meta_store
        //             .url()
        //             .set(format!("{}/encounters", summoner.read().to_route_path()));
        // });
        view! {
            <div>
                <div class="my-card flex space-x-2 my-2 w-fit">
                    <input
                        type="text"
                        class="my-input"
                        placeholder="Search for a summoner"
                        prop:value=move || local_search_summoner()
                        on:input=move |e| { set_local_search_summoner(event_target_value(&e)) }
                    />
                    <button
                        class="my-button flex items-center"
                        on:click=move |_| {
                            pending.set(true);
                            match_filters.page().set(None);
                            search_summoner.set(to_opt_string(local_search_summoner.get()));
                        }
                    >
                        <PendingLoading pending>Search</PendingLoading>
                    </button>
                    <button
                        class="my-button bg-red-700 hover:bg-red-800 text-gray-200"
                        on:click=move |_| {
                            search_summoner.set(None);
                            set_local_search_summoner(String::new());
                            match_filters.page().set(None);
                        }
                    >
                        Clear
                    </button>
                </div>
                <Transition fallback=move || {
                    view! { <div class="text-center">Loading Encounters</div> }
                }>
                    {move || {
                        Suspend::new(async move {
                            match encounters_resource.await {
                                Ok(encounters_result) => {
                                    let total_pages = encounters_result.total_pages;
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
                                                                            encounter.platform.code(),
                                                                            &encounter.game_name,
                                                                            &encounter.tag_line,
                                                                            false,
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
        }.into_any()
    }
}


#[derive(Clone, Default, Encode, Decode)]
pub struct SummonerEncountersResult {
    pub total_pages: u16,
    pub encounters: Vec<SummonerEncountersSummoner>,
}

#[derive(Clone, Encode, Decode)]
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
