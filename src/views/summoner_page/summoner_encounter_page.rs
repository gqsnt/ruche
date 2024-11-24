use crate::backend::server_fns::get_encounter::get_encounter;
use crate::consts::champion::Champion;
use crate::consts::item::Item;
use crate::consts::perk::Perk;
use crate::consts::profile_icon::ProfileIcon;
use crate::consts::summoner_spell::SummonerSpell;
use crate::consts::HasStaticAsset;
use crate::utils::{string_to_fixed_array, summoner_url, FixedToString, RiotMatchId};
use crate::views::components::pagination::Pagination;
use crate::views::summoner_page::match_details::MatchDetails;
use crate::views::summoner_page::Summoner;
use crate::views::{BackEndMatchFiltersSearch};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::{component, IntoView};
use leptos_router::hooks::{query_signal_with_options, use_query_map};
use leptos_router::NavigateOptions;
use leptos::server_fn::rkyv::{Deserialize, Serialize, Archive};
use crate::consts::platform_route::PlatformRoute;
use crate::consts::queue::Queue;

#[component]
pub fn SummonerEncounterPage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();

    let queries = use_query_map();
    let match_filters_updated = expect_context::<RwSignal<BackEndMatchFiltersSearch>>();
    let (is_with, set_is_with) = signal(true);

    let encounter_slug = move || queries.read().get("encounter_slug").unwrap_or_default();
    let encounter_platform = move || queries.read().get("encounter_platform").unwrap_or_default();

    let (page_number, set_page_number) = query_signal_with_options::<u16>(
        "page",
        NavigateOptions {
            scroll: false,
            replace: true,
            ..Default::default()
        },
    );

    let encounter_resource = leptos_server::Resource::new_rkyv(
        move || (summoner(), match_filters_updated.get(), page_number(), encounter_slug(), encounter_platform(), is_with.get()),
        |(summoner, filters, page_number, encounter_slug, encounter_platform, is_with)| async move {
            //println!("{:?} {:?} {:?}", filters, summoner, page_number);
            get_encounter(is_with, summoner.id,page_number.unwrap_or(1),  Some(filters), PlatformRoute::from(encounter_platform.as_str()), string_to_fixed_array::<22>(encounter_slug.as_str())).await
        },
    );

    let (reset_page_number, set_reset_page_number) = signal::<bool>(false);
    Effect::new(move |_| {
        if reset_page_number() {
            set_page_number(None);
            set_reset_page_number(false);
        }
    });

    view! {
        <div class="flex my-card space-x-2 my-2">
            <button

                class="w-[22rem] "
                class=("active-tab", move || is_with())
                class=("default-tab", move || !is_with())
                on:click=move |_| set_is_with(true)
            >
                With
            </button>
            <button
                class="w-[22rem] "
                class=("active-tab", move || !is_with())
                class=("default-tab", move || is_with())
                on:click=move |_| set_is_with(false)
            >
                VS
            </button>
        </div>
        <div class="w-[768px]">
            <Suspense fallback=move || {
                view! { <div class="text-center">Loading Encounter</div> }
            }>
                {move || Suspend::new(async move {
                    match encounter_resource.await {
                        Ok(encounter_result) => {
                            let total_pages = encounter_result.total_pages;
                            let current_page = page_number().unwrap_or(1);
                            if total_pages == 0 || total_pages < current_page {
                                set_reset_page_number(true);
                            }
                            if encounter_result.matches.is_empty() {
                                Ok(
                                    Either::Left(
                                        view! {
                                            <div class="text-center">No Encounter Matches Found</div>
                                        },
                                    ),
                                )
                            } else {
                                Ok(
                                    Either::Right(
                                        view! {
                                            <div class="flex w-full">
                                                <div class="flex w-full my-card justify-between">
                                                    <SummonerEncounterStat
                                                        summoner=encounter_result.summoner.clone()
                                                        stats=encounter_result.summoner_stats.clone()
                                                        is_self=true
                                                    />
                                                    <SummonerEncounterStat
                                                        summoner=encounter_result.encounter.clone()
                                                        stats=encounter_result.encounter_stats.clone()
                                                        is_self=false
                                                    />

                                                </div>
                                            </div>
                                            <div class="flex flex-col space-y-2 mt-2">
                                                <For
                                                    each=move || encounter_result.matches.clone()
                                                    key=|match_| match_.match_id
                                                    let:match_
                                                >
                                                    <SummonerEncounterMatchComponent match_=match_ summoner />
                                                </For>
                                            </div>

                                            <Show when=move || (encounter_result.total_pages > 1)>
                                                <Pagination max_page=encounter_result.total_pages />
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
    }
}


#[component]
pub fn SummonerEncounterMatchComponent(match_: SummonerEncounterMatch, summoner: ReadSignal<Summoner>) -> impl IntoView {
    let (show_details, set_show_details) = signal(false);
    view! {
        <div class="flex flex-col">
            <div class="flex  my-card w-[768px]">
                <div class="flex flex-col  gap-2">
                    <div class="flex flex-col items-start w-[108px]">
                        <div class="uppercase font-bold text-ellipsis max-w-[90%] overflow-hidden whitespace-nowrap">
                            {match_.queue.to_str()}
                        </div>
                        <div>{match_.match_ended_since.clone()}</div>
                    </div>
                    <hr class="w-1/2" />
                    <div class="flex flex-col items-start w-[108px]">
                        <div>{match_.match_duration.clone()}</div>
                    </div>
                </div>
                <div class="flex w-full">
                    <SummonerEncounterParticipantComponent
                        encounter_participant=match_.participant
                        is_self=true
                    />
                    <SummonerEncounterParticipantComponent
                        encounter_participant=match_.encounter
                        is_self=false
                    />
                </div>
                <div class="w-[40px] flex relative flex-col">
                    <button
                        aria-label="Show Details"
                        class="px-1 flex flex-col items-center justify-end h-full"
                        on:click=move |_| set_show_details(!show_details())
                    >
                        <span class="w-[24px] h-[24px]">
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

#[component]
pub fn SummonerEncounterParticipantComponent(encounter_participant: SummonerEncounterParticipant, is_self: bool) -> impl IntoView {
    view! {
        <div
            class="flex flex-col h-full gap-0.5 justify-start w-full px-2 "
            class=("bg-red-900", move || !encounter_participant.won)
            class=("bg-blue-900", move || encounter_participant.won)
            class=("rounded-r-lg", move || !is_self)
            class=("border-l-2", move || !is_self)
            class=("border-gray-800", move || !is_self)
            class=("rounded-l-lg", move || is_self)
        >

            <div class="flex items-center gap-2.5 " class=("flex-row-reverse", move || !is_self)>
                <div class="relative flex">
                    <img
                        width="48"
                        height="48"
                        alt=Champion::from(encounter_participant.champion_id).to_str()
                        src=Champion::get_static_asset_url(encounter_participant.champion_id)
                        class="w-12 h-12 rounded-full"
                    />
                    <span
                        class="absolute right-0 bottom-0 flex w-[20px] h-[20px] justify-center items-center bg-gray-800 text-white rounded-full"
                        style="font-size:11px"
                    >
                        {encounter_participant.champ_level}
                    </span>
                </div>
                <div class="gap-0.5 flex">
                    <div class="flex flex-col gap-0.5">
                        <div class="relative rounded">
                            <img
                                width="22"
                                height="22"
                                alt=SummonerSpell::from(encounter_participant.summoner_spell1_id)
                                    .to_string()
                                src=SummonerSpell::get_static_asset_url(
                                    encounter_participant.summoner_spell1_id,
                                )
                                class="w-[22px] w-[22px]"
                            />
                        </div>
                        <div class="relative rounded">
                            <img
                                width="22"
                                height="22"
                                alt=SummonerSpell::from(encounter_participant.summoner_spell2_id)
                                    .to_string()
                                src=SummonerSpell::get_static_asset_url(
                                    encounter_participant.summoner_spell2_id,
                                )
                                class="w-[22px] w-[22px]"
                            />
                        </div>
                    </div>
                    <div class="flex flex-col gap-0.5">
                        <Show when=move || encounter_participant.perk_primary_selection_id != 0>
                            <div class="relative rounded-full">
                                <img
                                    width="22"
                                    height="22"
                                    alt=Perk::from(encounter_participant.perk_primary_selection_id)
                                        .to_string()
                                    src=Perk::get_static_asset_url(
                                        encounter_participant.perk_primary_selection_id,
                                    )
                                    class="w-[22px] w-[22px]"
                                />
                            </div>
                        </Show>
                        <Show when=move || encounter_participant.perk_sub_style_id != 0>
                            <div class="relative rounded-full">
                                <img
                                    width="22"
                                    height="22"
                                    alt=Perk::from(encounter_participant.perk_sub_style_id)
                                        .to_string()
                                    src=Perk::get_static_asset_url(
                                        encounter_participant.perk_sub_style_id,
                                    )
                                    class="w-[22px] w-[22px]"
                                />
                            </div>
                        </Show>
                    </div>
                </div>
                <div class="flex flex-col w-[85px] items-start gap-1">
                    <div class="text-base">
                        <span class="text-white">{encounter_participant.kills}</span>
                        /
                        <span class="text-red-300">{encounter_participant.deaths}</span>
                        /
                        <span class="text-white">{encounter_participant.assists}</span>
                    </div>
                    <div>{format!("{:.2}", encounter_participant.kda)}:1 KDA</div>
                </div>
                <div
                    class="flex flex-col h-[58px]  "
                    class=("border-l-2", move || is_self)
                    class=("pl-2", move || is_self)
                    class=("border-r-2", move || !is_self)
                    class=("pr-2", move || !is_self)
                    class=("border-red-500", move || !encounter_participant.won)
                    class=("border-blue-500", move || encounter_participant.won)
                >
                    <div class="text-red-300 text-sm">
                        P/Kill {format!("{:.2}", encounter_participant.kill_participation)}%
                    </div>
                </div>
            </div>
            <div class="flex gap-0.5 " class=("flex-row-reverse", move || !is_self)>
                <Show when=move || encounter_participant.item0_id != 0>
                    <div class="relative rounded">
                        <img
                            alt=format!("Item {}", encounter_participant.item0_id)
                            width="22"
                            height="22"
                            src=Item::get_static_asset_url_u32(encounter_participant.item0_id)
                            class="w-[22px] w-[22px]"
                        />
                    </div>
                </Show>
                <Show when=move || encounter_participant.item1_id != 0>
                    <div class="relative rounded">
                        <img
                            alt=format!("Item {}", encounter_participant.item1_id)
                            width="22"
                            height="22"
                            src=Item::get_static_asset_url_u32(encounter_participant.item1_id)
                            class="w-[22px] w-[22px]"
                        />
                    </div>
                </Show>
                <Show when=move || encounter_participant.item2_id != 0>
                    <div class="relative rounded">
                        <img
                            alt=format!("Item {}", encounter_participant.item2_id)
                            width="22"
                            height="22"
                            src=Item::get_static_asset_url_u32(encounter_participant.item2_id)
                            class="w-[22px] w-[22px]"
                        />
                    </div>
                </Show>
                <Show when=move || encounter_participant.item3_id != 0>
                    <div class="relative rounded">
                        <img
                            alt=format!("Item {}", encounter_participant.item3_id)
                            width="22"
                            height="22"
                            src=Item::get_static_asset_url_u32(encounter_participant.item3_id)
                            class="w-[22px] w-[22px]"
                        />
                    </div>
                </Show>
                <Show when=move || encounter_participant.item4_id != 0>
                    <div class="relative rounded">
                        <img
                            alt=format!("Item {}", encounter_participant.item4_id)
                            width="22"
                            height="22"
                            src=Item::get_static_asset_url_u32(encounter_participant.item4_id)
                            class="w-[22px] w-[22px]"
                        />
                    </div>
                </Show>
                <Show when=move || encounter_participant.item5_id != 0>
                    <div class="relative rounded">
                        <img
                            alt=format!("Item {}", encounter_participant.item5_id)
                            width="22"
                            height="22"
                            src=Item::get_static_asset_url_u32(encounter_participant.item5_id)
                            class="w-[22px] w-[22px]"
                        />
                    </div>
                </Show>
                <Show when=move || encounter_participant.item6_id != 0>
                    <div class="relative rounded">
                        <img
                            alt=format!("Item {}", encounter_participant.item6_id)
                            width="22"
                            height="22"
                            src=Item::get_static_asset_url_u32(encounter_participant.item6_id)
                            class="w-[22px] w-[22px]"
                        />
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn SummonerEncounterStat(summoner: Summoner, stats: SummonerEncounterStats, is_self: bool) -> impl IntoView {
    let has_slug = summoner.pro_slug.is_some();
    view! {
        <div class="flex w-1/2 " class=("flex-row-reverse", move || !is_self)>
            <img
                alt="Profile Icon"
                src=ProfileIcon::get_static_asset_url(summoner.profile_icon_id)
                class="w-16 h-16"
            />
            <div
                class="flex flex-col items-start "
                class=("ml-2", move || is_self)
                class=("mr-2", move || !is_self)
            >
                <div>
                    <a href=summoner_url(
                        summoner.platform.to_string(),
                        summoner.game_name.to_string(),
                        summoner.tag_line.to_string(),
                    )>{summoner.game_name.to_string()}# {summoner.tag_line.to_string()}</a>
                </div>
                <div>
                    <span>lvl. {summoner.summoner_level}</span>
                    <Show when=move || has_slug>

                        <a
                            target="_blank"
                            href=format!(
                                "https://lolpros.gg/player/{}",
                                summoner.pro_slug.unwrap().to_str(),
                            )
                            class=" bg-purple-800 rounded px-1 py-0.5 text-center ml-1"
                        >
                            PRO
                        </a>
                    </Show>
                </div>
            </div>
            <div
                class="flex flex-col text-sm "
                class=("ml-2", move || is_self)
                class=("mr-2", move || !is_self)
            >
                <div>
                    {stats.total_wins}W {stats.total_loses}L {stats.total_wins + stats.total_loses}G
                    {format!(
                        "{:.2}%",
                        (stats.total_wins as f32
                            / (stats.total_wins + stats.total_loses).max(1) as f32) * 100.0,
                    )}
                </div>
                <div class="flex flex-col">
                    <div>
                        {format!("{:.2}", stats.avg_kills)}/ {format!("{:.2}", stats.avg_deaths)}/
                        {format!("{:.2}", stats.avg_assists)}
                    </div>
                    <div>
                        {format!("{:.2}", stats.avg_kda)}:1 P/kill
                        {format!("{:.2}", stats.avg_kill_participation)}%
                    </div>
                </div>

            </div>
        </div>
    }
}


#[derive(Clone, Serialize, Deserialize, Archive)]
pub struct SummonerEncounterResult {
    pub total_pages: u16,
    pub matches: Vec<SummonerEncounterMatch>,
    pub summoner_stats: SummonerEncounterStats,
    pub encounter_stats: SummonerEncounterStats,
    pub summoner: Summoner,
    pub encounter: Summoner,
}

#[derive(Clone, Serialize, Deserialize, Archive)]
pub struct SummonerEncounterStats {
    pub total_wins: u16,
    pub total_loses: u16,
    pub avg_kills: f32,
    pub avg_deaths: f32,
    pub avg_assists: f32,
    pub avg_kda: f32,
    pub avg_kill_participation: f32,
}

#[derive(Clone, Serialize, Deserialize, Archive)]
pub struct SummonerEncounterParticipant {
    pub won: bool,
    pub champion_id: u16,
    pub champ_level: u16,
    pub kills: u16,
    pub deaths: u16,
    pub assists: u16,
    pub summoner_spell1_id: u16,
    pub summoner_spell2_id: u16,
    pub perk_primary_selection_id: u16,
    pub perk_sub_style_id: u16,
    pub summoner_id: i32,
    pub item0_id: u32,
    pub item1_id: u32,
    pub item2_id: u32,
    pub item3_id: u32,
    pub item4_id: u32,
    pub item5_id: u32,
    pub item6_id: u32,
    pub kda: f32,
    pub kill_participation: f32,
}

#[derive(Clone, Serialize, Deserialize, Archive)]
pub struct SummonerEncounterMatch {
    pub platform: PlatformRoute,
    pub queue: Queue,
    pub match_id: i32,
    pub riot_match_id: RiotMatchId,
    pub match_ended_since: String,
    pub match_duration: String,
    pub participant: SummonerEncounterParticipant,
    pub encounter: SummonerEncounterParticipant,
}