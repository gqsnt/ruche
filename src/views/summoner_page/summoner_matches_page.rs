use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_matches::get_matches;
use crate::consts::champion::Champion;
use crate::consts::item::Item;
use crate::consts::perk::Perk;
use crate::consts::platform_route::PlatformRoute;
use crate::consts::queue::Queue;
use crate::consts::summoner_spell::SummonerSpell;
use crate::consts::HasStaticAsset;
use crate::utils::{
    calculate_and_format_kda, calculate_loss_and_win_rate, format_duration,
    format_float_to_2digits, summoner_encounter_url, summoner_url, DurationSince, GameName,
    ProPlayerSlug, RiotMatchId, TagLine,
};
use crate::views::components::pagination::Pagination;
use crate::views::summoner_page::match_details::MatchDetails;
use crate::views::summoner_page::Summoner;
use crate::views::BackEndMatchFiltersSearch;
use leptos::either::Either;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::server_fn::rkyv::{Archive, Deserialize, Serialize};
use leptos::{component, view, IntoView};
use leptos_router::hooks::query_signal_with_options;
use leptos_router::NavigateOptions;

#[component]
pub fn SummonerMatchesPage(summoner: Summoner) -> impl IntoView {
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

    let (reset_page_number, set_reset_page_number) = signal::<bool>(false);
    Effect::new(move |_| {
        if reset_page_number() {
            set_page_number(None);
            set_reset_page_number(false);
        }
    });

    let matches_resource = Resource::new_rkyv(
        move || {
            (
                summoner_update_version.get().unwrap_or_default(),
                match_filters_updated.get(),
                summoner.id,
                page_number(),
            )
        },
        |(_, filters, id, page_number)| async move {
            get_matches(id, page_number.unwrap_or(1), Some(filters)).await
        },
    );

    meta_store.title().set(format!(
        "{}#{} | Matches | Broken.gg",
        summoner.game_name.to_str(),
        summoner.tag_line.to_str()
    ));
    meta_store.description().set(format!("Explore {}#{}'s match history on Broken.gg. Analyze detailed League Of Legends stats, KDA ratios, and performance metrics on our high-speed, resource-efficient platform.", summoner.game_name.to_str(), summoner.tag_line.to_str()));
    meta_store.url().set(summoner.to_route_path());
    view! {
        <div class="w-[768px] inline-block align-top justify-center">
            <div class="">
                <Suspense fallback=move || {
                    view! { <div class="text-center">Loading Matches</div> }
                }>
                    {move || Suspend::new(async move {
                        match matches_resource.await {
                            Ok(matches_result) => {
                                let total_pages = matches_result.total_pages;
                                let current_page = page_number().unwrap_or(1);
                                if total_pages == 0 || total_pages < current_page {
                                    set_reset_page_number(true);
                                }
                                if matches_result.matches.is_empty() {
                                    Ok(
                                        Either::Left(
                                            view! { <div class="text-center">No Matches Found</div> },
                                        ),
                                    )
                                } else {
                                    Ok(
                                        Either::Right({
                                            let (losses, winrate) = calculate_loss_and_win_rate(
                                                matches_result.matches_result_info.total_wins,
                                                matches_result.matches_result_info.total_matches,
                                            );
                                            view! {
                                                <div class="my-2 flex my-card w-fit">
                                                    <div class="flex flex-col">
                                                        <div>
                                                            {matches_result.matches_result_info.total_matches}G
                                                            {matches_result.matches_result_info.total_wins}W
                                                            {losses as u16}L
                                                        </div>
                                                        <div>{format_float_to_2digits(winrate)}%</div>
                                                    </div>
                                                    <div class="flex flex-col ml-2">
                                                        <div>
                                                            {format!(
                                                                "{:.2}",
                                                                matches_result.matches_result_info.avg_kills,
                                                            )}/
                                                            {format!(
                                                                "{:.2}",
                                                                matches_result.matches_result_info.avg_deaths,
                                                            )}/
                                                            {format!(
                                                                "{:.2}",
                                                                matches_result.matches_result_info.avg_assists,
                                                            )}
                                                        </div>
                                                        <div>
                                                            {calculate_and_format_kda(
                                                                matches_result.matches_result_info.avg_kills,
                                                                matches_result.matches_result_info.avg_deaths,
                                                                matches_result.matches_result_info.avg_assists,
                                                            )}:1
                                                        </div>
                                                        <div>
                                                            P/kill
                                                            {format!(
                                                                "{:.2}",
                                                                matches_result.matches_result_info.avg_kill_participation,
                                                            )}%
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class="text-gray-200 space-y-2">
                                                    <For
                                                        each=move || matches_result.matches.clone()
                                                        key=|match_| match_.match_id
                                                        let:match_
                                                    >
                                                        <MatchCard match_=match_ summoner />
                                                    </For>
                                                </div>
                                                <Show when=move || (total_pages > 1)>
                                                    <Pagination max_page=total_pages />
                                                </Show>
                                            }
                                        }),
                                    )
                                }
                            }
                            Err(e) => Err(e),
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
pub fn MatchCard(match_: SummonerMatch, summoner: Summoner) -> impl IntoView {
    let (show_details, set_show_details) = signal(false);
    let champion = Champion::from(match_.champion_id);
    let summoner_spell1 = SummonerSpell::from(match_.summoner_spell1_id);
    let summoner_spell2 = SummonerSpell::from(match_.summoner_spell2_id);
    let primary_perk_selection = Perk::from(match_.perk_primary_selection_id);
    let sub_perk_style = Perk::from(match_.perk_sub_style_id);
    if primary_perk_selection == Perk::UNKNOWN {
        log!("{:?}", match_.perk_primary_selection_id);
    }
    if sub_perk_style == Perk::UNKNOWN {
        log!("{:?}", match_.perk_sub_style_id);
    }
    let item0 = Item::try_from(match_.item0_id).ok();
    let item1 = Item::try_from(match_.item1_id).ok();
    let item2 = Item::try_from(match_.item2_id).ok();
    let item3 = Item::try_from(match_.item3_id).ok();
    let item4 = Item::try_from(match_.item4_id).ok();
    let item5 = Item::try_from(match_.item5_id).ok();
    let item6 = Item::try_from(match_.item6_id).ok();
    view! {
        <div class="flex flex-col">
            <div class="min-h-24 w-full flex rounded text-xs">
                <div
                    class:bg-red-400=move || !match_.won
                    class:bg-blue-400=move || match_.won
                    class="min-w-1.5 w-1.5"
                ></div>
                <div
                    class:bg-red-900=move || !match_.won
                    class:bg-blue-900=move || match_.won
                    class="flex gap-2 py-0 px-3 w-full items-center"
                >
                    <div class="flex flex-col w-[108px] gap-2">
                        <div class="flex flex-col items-start">
                            <div
                                class:text-red-300=move || !match_.won
                                class:text-blue-300=move || match_.won
                                class="uppercase font-bold text-ellipsis max-w-[90%] overflow-hidden whitespace-nowrap"
                            >
                                {match_.queue.to_str()}
                            </div>
                            <div>{match_.match_ended_since.to_string()}</div>
                        </div>
                        <hr
                            class:border-red-500=move || !match_.won
                            class:border-blue-500=move || match_.won
                            class="w-1/2"
                        />
                        <div class="flex flex-col items-start">
                            <div>{if match_.won { "Victory" } else { "Defeat" }}</div>
                            <div>{format_duration(match_.match_duration)}</div>
                        </div>
                    </div>
                    <div class="flex flex-col h-full w-[308px]  gap-0.5 justify-start">
                        <div class="flex items-center gap-2.5">
                            <div class="relative flex">
                                <img
                                    width="48"
                                    height="48"
                                    alt=champion.to_str()
                                    src=champion.get_static_asset_url()
                                    class="w-12 h-12 rounded-full"
                                />
                                <span
                                    class="absolute right-0 bottom-0 flex w-[20px] h-[20px] justify-center items-center bg-gray-800 text-white rounded-full"
                                    style="font-size:11px"
                                >
                                    {match_.champ_level}
                                </span>
                            </div>
                            <div class="gap-0.5 flex">
                                <div class="flex flex-col gap-0.5">
                                    <div class="relative rounded">
                                        <img
                                            width="22"
                                            height="22"
                                            alt=summoner_spell1.to_string()
                                            src=summoner_spell1.get_static_asset_url()
                                            class="w-[22px] w-[22px]"
                                        />
                                    </div>
                                    <div class="relative rounded">
                                        <img
                                            width="22"
                                            height="22"
                                            alt=summoner_spell2.to_string()
                                            src=summoner_spell2.get_static_asset_url()
                                            class="w-[22px] w-[22px]"
                                        />
                                    </div>
                                </div>
                                <div class="flex flex-col gap-0.5">
                                    <Show when=move || match_.perk_primary_selection_id != 0>
                                        <div class="relative rounded-full">
                                            <img
                                                width="22"
                                                height="22"
                                                alt=primary_perk_selection.to_string()
                                                src=primary_perk_selection.get_static_asset_url()
                                                class="w-[22px] w-[22px]"
                                            />
                                        </div>
                                    </Show>
                                    <Show when=move || match_.perk_sub_style_id != 0>
                                        <div class="relative rounded-full">
                                            <img
                                                width="22"
                                                height="22"
                                                alt=sub_perk_style.to_string()
                                                src=sub_perk_style.get_static_asset_url()
                                                class="w-[22px] w-[22px]"
                                            />
                                        </div>
                                    </Show>
                                </div>
                            </div>
                            <div class="flex flex-col w-[85px] items-start gap-1">
                                <div class="text-base">
                                    <span class="text-white">{match_.kills}</span>
                                    /
                                    <span class="text-red-300">{match_.deaths}</span>
                                    /
                                    <span class="text-white">{match_.assists}</span>
                                </div>
                                <div>
                                    {calculate_and_format_kda(
                                        match_.kills,
                                        match_.deaths,
                                        match_.assists,
                                    )}:1 KDA
                                </div>
                            </div>
                            <div
                                class:border-red-500=move || !match_.won
                                class:border-blue-500=move || match_.won
                                class="flex flex-col h-[58px] pl-2 border-l-2"
                            >
                                <div class="text-red-300">P/Kill {match_.kill_participation}%</div>
                            </div>
                        </div>
                        <div class="flex gap-0.5">
                            <Show when=move || item0.is_some()>
                                <div class="relative rounded">
                                    {
                                        let inner = item0.unwrap();
                                        view! {
                                            <img
                                                alt=inner.to_string()
                                                width="22"
                                                height="22"
                                                src=inner.get_static_asset_url()
                                                class="w-[22px] w-[22px]"
                                            />
                                        }
                                    }

                                </div>
                            </Show>
                            <Show when=move || item1.is_some()>
                                <div class="relative rounded">
                                    {
                                        let inner = item1.unwrap();
                                        view! {
                                            <img
                                                alt=inner.to_string()
                                                width="22"
                                                height="22"
                                                src=inner.get_static_asset_url()
                                                class="w-[22px] w-[22px]"
                                            />
                                        }
                                    }

                                </div>
                            </Show>
                            <Show when=move || item2.is_some()>
                                <div class="relative rounded">
                                    {
                                        let inner = item2.unwrap();
                                        view! {
                                            <img
                                                alt=inner.to_string()
                                                width="22"
                                                height="22"
                                                src=inner.get_static_asset_url()
                                                class="w-[22px] w-[22px]"
                                            />
                                        }
                                    }

                                </div>
                            </Show>
                            <Show when=move || item3.is_some()>
                                <div class="relative rounded">
                                    {
                                        let inner = item3.unwrap();
                                        view! {
                                            <img
                                                alt=inner.to_string()
                                                width="22"
                                                height="22"
                                                src=inner.get_static_asset_url()
                                                class="w-[22px] w-[22px]"
                                            />
                                        }
                                    }

                                </div>
                            </Show>
                            <Show when=move || item4.is_some()>
                                <div class="relative rounded">
                                    {
                                        let inner = item4.unwrap();
                                        view! {
                                            <img
                                                alt=inner.to_string()
                                                width="22"
                                                height="22"
                                                src=inner.get_static_asset_url()
                                                class="w-[22px] w-[22px]"
                                            />
                                        }
                                    }

                                </div>
                            </Show>
                            <Show when=move || item5.is_some()>
                                <div class="relative rounded">
                                    {
                                        let inner = item5.unwrap();
                                        view! {
                                            <img
                                                alt=inner.to_string()
                                                width="22"
                                                height="22"
                                                src=inner.get_static_asset_url()
                                                class="w-[22px] w-[22px]"
                                            />
                                        }
                                    }

                                </div>
                            </Show>
                            <Show when=move || item6.is_some()>
                                <div class="relative rounded">
                                    {
                                        let inner = item6.unwrap();
                                        view! {
                                            <img
                                                alt=inner.to_string()
                                                width="22"
                                                height="22"
                                                src=inner.get_static_asset_url()
                                                class="w-[22px] w-[22px]"
                                            />
                                        }
                                    }

                                </div>
                            </Show>
                        </div>
                    </div>
                    <div
                        class="flex gap-x-2 gap-y-0.5 w-[266px] max-h-[89px]"
                        style="flex-flow:column wrap"
                    >
                        {match_
                            .participants
                            .into_iter()
                            .map(|participant| {
                                let is_pro_player = participant.pro_player_slug.is_some();
                                let champion = Champion::from(participant.champion_id);
                                view! {
                                    <div class="flex items-center gap-1 w-[130px]">
                                        <img
                                            width="16"
                                            height="16"
                                            alt=champion.to_str()
                                            src=champion.get_static_asset_url()
                                            class="w-4 h-4 rounded"
                                        />
                                        <Show when=move || (participant.encounter_count > 1)>
                                            <a
                                                href=summoner_encounter_url(
                                                    summoner.platform.to_string(),
                                                    summoner.game_name.to_string(),
                                                    summoner.tag_line.to_string(),
                                                    participant.platform.to_string(),
                                                    participant.game_name.to_string(),
                                                    participant.tag_line.to_string(),
                                                )
                                                class="text-xs bg-green-800 rounded px-0.5 text-center"
                                            >
                                                {participant.encounter_count}
                                            </a>
                                        </Show>
                                        <Show when=move || is_pro_player>
                                            <a
                                                target="_blank"
                                                href=format!(
                                                    "https://lolpros.gg/player/{}",
                                                    participant.pro_player_slug.unwrap().to_string().as_str(),
                                                )
                                                class="text-xs bg-purple-800 rounded px-0.5 text-center"
                                            >
                                                pro
                                            </a>
                                        </Show>
                                        <a
                                            target="_blank"
                                            href=summoner_url(
                                                participant.platform.to_string(),
                                                participant.game_name.to_string(),
                                                participant.tag_line.to_string(),
                                            )
                                            class:text-white=move || {
                                                participant.summoner_id == match_.summoner_id
                                            }
                                            class="text-ellipsis overflow-hidden whitespace-nowrap "
                                        >
                                            {participant.game_name.to_string()}
                                        </a>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </div>
                </div>
                <div class="w-[40px] flex relative flex-col">
                    <button
                        aria-label="Show Details"
                        class:bg-red-600=move || !match_.won
                        class:bg-blue-600=move || match_.won
                        class="p-2 flex flex-col items-center justify-end h-full"
                        on:click=move |_| set_show_details(!show_details())
                    >
                        <span
                            class="w-[24px] h-[24px]"
                            class:text-red-300=move || !match_.won
                            class:text-blue-400=move || match_.won
                        >
                            <svg
                                class=move || ("rotate-180", show_details())
                                xmlns="http://www.w3.org/2000/svg"
                                width="24"
                                height="24"
                                viewBox="0 0 24 24"
                                fill="currentColor"
                            >
                                <g fill="currentColor" fill-rule="evenodd">
                                    <g fill="currentColor" fill-rule="nonzero">
                                        <g fill="currentColor">
                                            <path
                                                d="M12 13.2L16.5 9 18 10.4 12 16 6 10.4 7.5 9z"
                                                transform="translate(-64 -228) translate(64 228)"
                                                fill="currentColor"
                                            ></path>
                                        </g>
                                    </g>
                                </g>
                            </svg>
                        </span>
                    </button>
                </div>
            </div>
            <Show when=move || show_details()>
                <MatchDetails
                    match_id=match_.match_id
                    summoner
                    riot_match_id=match_.riot_match_id
                    platform=match_.platform
                />
            </Show>
        </div>
    }
}

#[derive(Clone, Deserialize, Serialize, Default, Archive)]
pub struct GetSummonerMatchesResult {
    pub total_pages: u16,
    pub matches: Vec<SummonerMatch>,
    pub matches_result_info: MatchesResultInfo,
}

#[derive(Clone, Deserialize, Serialize, Default, Archive)]
pub struct MatchesResultInfo {
    pub avg_kills: f32,
    pub avg_deaths: f32,
    pub avg_assists: f32,
    pub avg_kill_participation: u16,
    pub total_matches: u16,
    pub total_wins: u16,
}
#[derive(Clone, Serialize, Deserialize, Archive)]
pub struct SummonerMatch {
    pub participants: Vec<SummonerMatchParticipant>,
    pub summoner_id: i32,
    pub match_id: i32,
    pub champ_level: i32,
    pub match_duration: Option<i32>,
    pub item0_id: u32,
    pub item1_id: u32,
    pub item2_id: u32,
    pub item3_id: u32,
    pub item4_id: u32,
    pub item5_id: u32,
    pub item6_id: u32,
    pub kill_participation: u16,
    pub champion_id: u16,
    pub kills: u16,
    pub deaths: u16,
    pub assists: u16,
    pub summoner_spell1_id: u16,
    pub summoner_spell2_id: u16,
    pub perk_primary_selection_id: u16,
    pub perk_sub_style_id: u16,
    pub riot_match_id: RiotMatchId,
    pub match_ended_since: DurationSince,
    pub won: bool,
    pub queue: Queue,
    pub platform: PlatformRoute,
}

#[derive(Clone, Serialize, Deserialize, Archive)]
pub struct SummonerMatchParticipant {
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub champion_id: u16,
    pub team_id: u16,
    pub encounter_count: u16,
    pub game_name: GameName,
    pub pro_player_slug: Option<ProPlayerSlug>,
    pub tag_line: TagLine,
    pub platform: PlatformRoute,
}
