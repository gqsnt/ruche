use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_matches::get_matches;
use crate::consts::champion::Champion;
use crate::consts::item::Item;
use crate::consts::perk::Perk;
use crate::consts::summoner_spell::SummonerSpell;
use crate::consts::HasStaticAsset;
use crate::utils::{summoner_encounter_url, summoner_url};
use crate::views::components::pagination::Pagination;
use crate::views::summoner_page::match_details::MatchDetails;
use crate::views::summoner_page::Summoner;
use crate::views::MatchFiltersSearch;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::server_fn::serde::{Deserialize, Serialize};
use leptos::{component, view, IntoView};
use leptos_router::hooks::{query_signal_with_options};
use leptos_router::NavigateOptions;

#[component]
pub fn SummonerMatchesPage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

    let match_filters_updated = expect_context::<RwSignal<MatchFiltersSearch>>();
    let (page_number, set_page_number) = query_signal_with_options::<i32>(
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


    let matches_resource = Resource::new(
        move || (match_filters_updated.get(), summoner(), page_number()),
        |(filters, summoner, page_number)| async move {
            //println!("{:?} {:?} {:?}", filters, summoner, page_number);
            get_matches(summoner.id, page_number.unwrap_or(1), Some(filters)).await
        },
    );

    meta_store.title().set(format!("{}#{} | Matches | Broken.gg", summoner().game_name, summoner().tag_line));
    meta_store.description().set(format!("Explore {}#{}'s match history on Broken.gg. Analyze detailed League Of Legends stats, KDA ratios, and performance metrics on our high-speed, resource-efficient platform.", summoner().game_name, summoner().tag_line));
    meta_store.url().set( summoner().to_route_path());
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
                                if total_pages == 0 || (total_pages as i32) < current_page {
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
                                        Either::Right(
                                            view! {
                                                <div class="my-2 flex my-card w-fit">
                                                    <div class="flex flex-col">
                                                        <div>
                                                            {matches_result.matches_result_info.total_matches}G
                                                            {matches_result.matches_result_info.total_wins}W
                                                            {matches_result.matches_result_info.total_losses}L
                                                        </div>
                                                        <div>
                                                            {matches_result.matches_result_info.total_wins * 100
                                                                / matches_result.matches_result_info.total_matches.max(1)}%
                                                        </div>
                                                    </div>
                                                    <div class="flex flex-col ml-2">
                                                        <div>
                                                            {matches_result.matches_result_info.avg_kills}/
                                                            {matches_result.matches_result_info.avg_deaths}/
                                                            {matches_result.matches_result_info.avg_assists}
                                                        </div>
                                                        <div>{matches_result.matches_result_info.avg_kda}:1</div>
                                                        <div>
                                                            P/kill
                                                            {matches_result.matches_result_info.avg_kill_participation}%
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class="text-gray-200 space-y-2">
                                                    <For
                                                        each=move || matches_result.matches.clone()
                                                        key=|match_| match_.match_id
                                                        let:match_
                                                    >
                                                        <MatchCard match_=match_ summoner/>
                                                    </For>
                                                </div>
                                                <Show when=move || (total_pages > 1)>
                                                    <Pagination max_page=total_pages as usize />
                                                </Show>
                                            },
                                        ),
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
pub fn MatchCard(match_: SummonerMatch, summoner:ReadSignal<Summoner>) -> impl IntoView {
    let (show_details, set_show_details) = signal(false);

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
                                {match_.queue.clone()}
                            </div>
                            <div>{match_.match_ended_since.clone()}</div>
                        </div>
                        <hr
                            class:border-red-500=move || !match_.won
                            class:border-blue-500=move || match_.won
                            class="w-1/2"
                        />
                        <div class="flex flex-col items-start">
                            <div>{if match_.won { "Victory" } else { "Defeat" }}</div>
                            <div>{match_.match_duration.clone()}</div>
                        </div>
                    </div>
                    <div class="flex flex-col h-full w-[308px]  gap-0.5 justify-start">
                        <div class="flex items-center gap-2.5">
                            <div class="relative flex">
                                <img
                                    width="48"
                                    height="48"
                                    alt=Champion::from(match_.champion_id).to_str()
                                    src=Champion::get_static_asset_url(match_.champion_id)
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
                                            alt=SummonerSpell::from(match_.summoner_spell1_id)
                                                .to_string()
                                            src=SummonerSpell::get_static_asset_url(
                                                match_.summoner_spell1_id,
                                            )
                                            class="w-[22px] w-[22px]"
                                        />
                                    </div>
                                    <div class="relative rounded">
                                        <img
                                            width="22"
                                            height="22"
                                            alt=SummonerSpell::from(match_.summoner_spell2_id)
                                                .to_string()
                                            src=SummonerSpell::get_static_asset_url(
                                                match_.summoner_spell2_id,
                                            )
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
                                                alt=Perk::from(match_.perk_primary_selection_id).to_string()
                                                src=Perk::get_static_asset_url(
                                                    match_.perk_primary_selection_id,
                                                )
                                                class="w-[22px] w-[22px]"
                                            />
                                        </div>
                                    </Show>
                                    <Show when=move || match_.perk_sub_style_id != 0>
                                        <div class="relative rounded-full">
                                            <img
                                                width="22"
                                                height="22"
                                                alt=Perk::from(match_.perk_sub_style_id).to_string()
                                                src=Perk::get_static_asset_url(match_.perk_sub_style_id)
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
                                <div>{match_.kda}:1 KDA</div>
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
                            <Show when=move || match_.item0_id != 0>
                                <div class="relative rounded">
                                    <img
                                        alt=format!("Item {}", match_.item0_id)
                                        width="22"
                                        height="22"
                                        src=Item::get_static_asset_url_u32(match_.item0_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item1_id != 0>
                                <div class="relative rounded">
                                    <img
                                        alt=format!("Item {}", match_.item1_id)
                                        width="22"
                                        height="22"
                                        src=Item::get_static_asset_url_u32(match_.item1_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item2_id != 0>
                                <div class="relative rounded">
                                    <img
                                        alt=format!("Item {}", match_.item2_id)
                                        width="22"
                                        height="22"
                                        src=Item::get_static_asset_url_u32(match_.item2_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item3_id != 0>
                                <div class="relative rounded">
                                    <img
                                        alt=format!("Item {}", match_.item3_id)
                                        width="22"
                                        height="22"
                                        src=Item::get_static_asset_url_u32(match_.item3_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item4_id != 0>
                                <div class="relative rounded">
                                    <img
                                        alt=format!("Item {}", match_.item4_id)
                                        width="22"
                                        height="22"
                                        src=Item::get_static_asset_url_u32(match_.item4_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item5_id != 0>
                                <div class="relative rounded">
                                    <img
                                        alt=format!("Item {}", match_.item5_id)
                                        width="22"
                                        height="22"
                                        src=Item::get_static_asset_url_u32(match_.item5_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item6_id != 0>
                                <div class="relative rounded">
                                    <img
                                        alt=format!("Item {}", match_.item6_id)
                                        width="22"
                                        height="22"
                                        src=Item::get_static_asset_url_u32(match_.item6_id)
                                        class="w-[22px] w-[22px]"
                                    />
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
                                let participant_platform = participant.summoner_platform.clone();
                                let participant_name = participant.summoner_name.clone();
                                let participant_tag_line = participant.summoner_tag_line.clone();
                                view! {
                                    <div class="flex items-center gap-1 w-[130px]">
                                        <img
                                            width="16"
                                            height="16"
                                            alt=Champion::from(participant.champion_id).to_str()
                                            src=Champion::get_static_asset_url(participant.champion_id)
                                            class="w-4 h-4 rounded"
                                        />
                                        <Show when=move || (participant.encounter_count > 1)>
                                            <a
                                                href=summoner_encounter_url(
                                                    summoner().platform.to_string().as_str(),
                                                    summoner().game_name.as_str(),
                                                    summoner().tag_line.as_str(),
                                                    participant_platform.as_str(),
                                                    participant_name.as_str(),
                                                    participant_tag_line.as_str()
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
                                                    participant.pro_player_slug.clone().unwrap(),
                                                )
                                                class="text-xs bg-purple-800 rounded px-0.5 text-center"
                                            >
                                                pro
                                            </a>
                                        </Show>
                                        <a
                                            target="_blank"
                                            href=summoner_url(
                                                participant.summoner_platform.clone().as_str(),
                                                participant.summoner_name.clone().as_str(),
                                                participant.summoner_tag_line.clone().as_str(),
                                            )
                                            class:text-white=move || {
                                                participant.summoner_id == match_.summoner_id
                                            }
                                            class="text-ellipsis overflow-hidden whitespace-nowrap "
                                        >
                                            {participant.summoner_name.clone()}
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
                    riot_match_id=match_.riot_match_id.clone()
                    platform=match_.platform.clone()
                />
            </Show>
        </div>
    }
}


#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GetSummonerMatchesResult {
    pub matches: Vec<SummonerMatch>,
    pub total_pages: i64,
    pub matches_result_info: MatchesResultInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MatchesResultInfo {
    pub total_matches: i32,
    pub total_wins: i32,
    pub total_losses: i32,
    pub avg_kills: f64,
    pub avg_deaths: f64,
    pub avg_assists: f64,
    pub avg_kda: f64,
    pub avg_kill_participation: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummonerMatch {
    pub summoner_id: i32,
    pub match_id: i32,
    pub riot_match_id: String,
    pub match_ended_since: String,
    pub match_duration: String,
    pub queue: String,
    pub platform: String,
    pub champion_id: u16,
    pub won: bool,
    pub champ_level: i32,
    pub kda: f64,
    pub kill_participation: f64,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub summoner_spell1_id: u16,
    pub summoner_spell2_id: u16,
    pub perk_primary_selection_id: u16,
    pub perk_sub_style_id: u16,
    pub item0_id: u32,
    pub item1_id: u32,
    pub item2_id: u32,
    pub item3_id: u32,
    pub item4_id: u32,
    pub item5_id: u32,
    pub item6_id: u32,
    pub participants: Vec<SummonerMatchParticipant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummonerMatchParticipant {
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub summoner_name: String,
    pub summoner_tag_line: String,
    pub summoner_platform: String,
    pub champion_id: u16,
    pub team_id: i32,
    pub pro_player_slug: Option<String>,
    pub encounter_count:i32,
}
