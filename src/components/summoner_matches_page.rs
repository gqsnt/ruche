use crate::models::entities::lol_match_participant::LolMatchDefaultParticipantMatchesPage;

use crate::apis::{get_summoner_matches, MatchFiltersSearch};
use crate::components::match_details::MatchDetails;
use crate::components::pagination::Pagination;
use crate::models::entities::summoner::Summoner;
use leptos::either::Either;
use leptos::prelude::ElementChild;
use leptos::prelude::{expect_context, ClassAttribute, Get, ReadSignal, Resource, RwSignal, Show, StyleAttribute, Suspend, Suspense};
use leptos::prelude::{signal, CustomAttribute, Effect, OnAttribute};
use leptos::{component, view, IntoView};
use leptos_router::hooks::query_signal_with_options;
use leptos_router::NavigateOptions;
use serde::{Deserialize, Serialize};

#[component]
pub fn SummonerMatchesPage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();

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
            get_summoner_matches(summoner.id, page_number.unwrap_or(1), Some(filters)).await
        },
    );


    view! {
        <div class="flex">
            <div class="w-[740px] inline-block align-top justify-center">
                <Suspense fallback=move || {
                    view! { <p>"Loading..."</p> }
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
                                            view! { <div class="text-center">No matches found</div> },
                                        ),
                                    )
                                } else {
                                    Ok(
                                        Either::Right(
                                            view! {
                                                <div class="my-2 flex">
                                                    <div class="flex flex-col">
                                                        <div>{matches_result.matches_result_info.total_matches}G {matches_result.matches_result_info.total_wins}W {matches_result.matches_result_info.total_losses}L</div>
                                                        <div>{ matches_result.matches_result_info.total_wins / matches_result.matches_result_info.total_matches.min(1) }</div>
                                                    </div>

                                                </div>
                                                <div class="text-gray-400 space-y-2">
                                                    {matches_result
                                                        .matches
                                                        .into_iter()
                                                        .map(|match_| {
                                                            view! { <MatchCard match_=match_ /> }
                                                        })
                                                        .collect::<Vec<_>>()}
                                                </div>
                                                <Show when=move || (total_pages > 1)>
                                                    <Pagination max_page=(move || total_pages as usize)() />
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
pub fn MatchCard(match_: LolMatchDefaultParticipantMatchesPage) -> impl IntoView {
    let (show_details, set_show_details) = signal(false);

    view! {
        <div class="flex flex-col">
            <div class="min-h-24 w-full flex rounded text-xs">
                <div
                    class=("bg-rose-500", !match_.won)
                    class=("bg-blue-500", match_.won)
                    class="min-w-1.5 w-1.5"
                ></div>
                <div
                    class=("bg-rose-800", !match_.won)
                    class=("bg-blue-800", match_.won)
                    class="flex gap-2 py-0 px-3 w-full items-center"
                >
                    <div class="flex flex-col w-[108px] gap-2">
                        <div class="flex flex-col items-start">
                            <div
                                class=("text-rose-500", !match_.won)
                                class=("text-blue-500", match_.won)
                                class="uppercase font-bold text-ellipsis max-w-[90%] overflow-hidden whitespace-nowrap"
                            >
                                {match_.queue.clone()}
                            </div>
                            <div>{match_.match_ended_since.clone()}</div>
                        </div>
                        <hr
                            class=("border-rose-600", !match_.won)
                            class=("border-blue-600", match_.won)
                            class="w-1/2"
                        />
                        <div class="flex flex-col items-start">
                            <div>{if match_.won { "Victory" } else { "Defeat" }}</div>
                            <div>{match_.match_duration.clone()}</div>
                        </div>
                    </div>
                    <div class="flex flex-col h-full w-[378px]  gap-0.5 justify-start">
                        <div class="flex items-center gap-2.5">
                            <div class="relative flex">
                                <img
                                    width="48"
                                    height="48"
                                    src=format!("/assets/champions/{}.webp", match_.champion_id)
                                    class="w-12 h-12 rounded-full"
                                />
                                <span
                                    class="absolute right-0 bottom-0 flex w-[20px] h-[20px] justify-center items-center bg-gray-700 text-white rounded-full"
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
                                            src=format!(
                                                "/assets/summoner_spells/{}.webp",
                                                match_.summoner_spell1_id,
                                            )
                                            class="w-[22px] w-[22px]"
                                        />
                                    </div>
                                    <div class="relative rounded">
                                        <img
                                            width="22"
                                            height="22"
                                            src=format!(
                                                "/assets/summoner_spells/{}.webp",
                                                match_.summoner_spell2_id,
                                            )
                                            class="w-[22px] w-[22px]"
                                        />
                                    </div>
                                </div>
                                <div class="flex flex-col gap-0.5">
                                    <Show
                                        when=move || match_.perk_primary_selection_id != 0
                                        fallback=|| view! {}
                                    >
                                        <div class="relative rounded-full">
                                            <img
                                                width="22"
                                                height="22"
                                                src=format!(
                                                    "/assets/perks/{}.png",
                                                    match_.perk_primary_selection_id,
                                                )
                                                class="w-[22px] w-[22px]"
                                            />
                                        </div>
                                    </Show>
                                    <Show
                                        when=move || match_.perk_sub_style_id != 0
                                        fallback=|| view! {}
                                    >
                                        <div class="relative rounded-full">
                                            <img
                                                width="22"
                                                height="22"
                                                src=format!(
                                                    "/assets/perks/{}.png",
                                                    match_.perk_sub_style_id,
                                                )
                                                class="w-[22px] w-[22px]"
                                            />
                                        </div>
                                    </Show>
                                </div>
                            </div>
                            <div class="flex flex-col w-[108px] items-start gap-1">
                                <div class="text-base">
                                    <span class="text-white">{match_.kills}</span>
                                    /
                                    <span class="text-rose-400">{match_.deaths}</span>
                                    /
                                    <span class="text-white">{match_.assists}</span>
                                </div>
                                <div>{match_.kda}:1 KDA</div>
                            </div>
                            <div
                                class=("border-rose-600", !match_.won)
                                class=("border-blue-600", match_.won)
                                class="flex flex-col h-[58px] pl-2 border-l-2"
                            >
                                <div class="text-rose-500">P/Kill {match_.kill_participation}%</div>
                            </div>
                        </div>
                        <div class="flex gap-0.5">
                            <Show when=move || match_.item0_id != 0 fallback=|| view! {}>
                                <div class="relative rounded">
                                    <img
                                        width="22"
                                        height="22"
                                        src=format!("/assets/items/{}.webp", match_.item0_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item1_id != 0 fallback=|| view! {}>
                                <div class="relative rounded">
                                    <img
                                        width="22"
                                        height="22"
                                        src=format!("/assets/items/{}.webp", match_.item1_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item2_id != 0 fallback=|| view! {}>
                                <div class="relative rounded">
                                    <img
                                        width="22"
                                        height="22"
                                        src=format!("/assets/items/{}.webp", match_.item2_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item3_id != 0 fallback=|| view! {}>
                                <div class="relative rounded">
                                    <img
                                        width="22"
                                        height="22"
                                        src=format!("/assets/items/{}.webp", match_.item3_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item4_id != 0 fallback=|| view! {}>
                                <div class="relative rounded">
                                    <img
                                        width="22"
                                        height="22"
                                        src=format!("/assets/items/{}.webp", match_.item4_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item5_id != 0 fallback=|| view! {}>
                                <div class="relative rounded">
                                    <img
                                        width="22"
                                        height="22"
                                        src=format!("/assets/items/{}.webp", match_.item5_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                            <Show when=move || match_.item6_id != 0 fallback=|| view! {}>
                                <div class="relative rounded">
                                    <img
                                        width="22"
                                        height="22"
                                        src=format!("/assets/items/{}.webp", match_.item6_id)
                                        class="w-[22px] w-[22px]"
                                    />
                                </div>
                            </Show>
                        </div>
                    </div>
                    <div
                        class="flex gap-x-2 gap-y-0.5 w-[168px] max-h-[89px]"
                        style="flex-flow:column wrap"
                    >
                        {match_
                            .participants
                            .into_iter()
                            .map(|participant| {
                                view! {
                                    <div class="flex items-center gap-1">
                                        <img
                                            width="16"
                                            height="16"
                                            src=format!(
                                                "/assets/champions/{}.webp",
                                                participant.champion_id,
                                            )
                                            class="w-4 h-4 rounded"
                                        />
                                        <a
                                            target="_blank"
                                            href=format!(
                                                "/{}/summoners/{}-{}",
                                                participant.summoner_platform,
                                                participant.summoner_name,
                                                participant.summoner_tag_line,
                                            )
                                            class=(
                                                "text-white",
                                                participant.summoner_id == match_.summoner_id,
                                            )
                                            class="text-ellipsis overflow-hidden whitespace-nowrap max-w-[60px]"
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
                        class=("bg-rose-600", !match_.won)
                        class=("bg-blue-600", match_.won)
                        class="p-2 flex flex-col items-center justify-end h-full"
                        on:click=move |_| set_show_details(!show_details())
                    >
                        <span
                            class="w-[24px] h-[24px]"
                            class=("text-rose-500", !match_.won)
                            class=("text-blue-500", match_.won)
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
                    summoner_id=match_.summoner_id
                    riot_match_id=match_.riot_match_id.clone()
                    platform=match_.platform.clone()
                />
            </Show>
        </div>
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetSummonerMatchesResult {
    pub matches: Vec<LolMatchDefaultParticipantMatchesPage>,
    pub total_pages: i64,
    pub matches_result_info:MatchesResultInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MatchesResultInfo {
    pub total_matches: i64,
    pub total_wins: i64,
    pub total_losses: i64,
    pub avg_kills: f64,
    pub avg_deaths: f64,
    pub avg_assists: f64,
    pub avg_kda: f64,
    pub avg_kill_participation: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PageItem {
    label: String,
    page: i64,
    disabled: bool,
    is_current: bool,
}