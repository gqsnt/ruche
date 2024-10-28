use crate::error_template::AppError;
use crate::models::entities::lol_match_participant::{LolMatchDefaultParticipantMatchesPage, LolMatchParticipant};
#[cfg(feature = "ssr")]
use crate::AppState;
use itertools::partition;
use leptos::html::{form, A};
use leptos::{component, create_blocking_resource, create_effect, create_local_resource, create_resource, create_signal, expect_context, server, use_context, view, CollectView, For, IntoView, RwSignal, ServerFnError, Show, Signal, SignalGet, SignalWith, Suspense, Transition};
use leptos_router::use_query_map;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[component]
pub fn SummonerMatchesPage() -> impl IntoView {
    let query = use_query_map();
    let get_page_number = move || {
        query.with(|q| {
            q.get("page")
                .and_then(|p| p.parse::<i64>().ok())
                .unwrap_or(0)
        })
    };

    let (internal_matches, set_internal_matches) = create_signal(vec![]);
    let (internal_pages, set_internal_pages) = create_signal(vec![]);

    let summoner_id = use_context::<RwSignal<Option<i32>>>().expect("summoner_id not found");
    let matches = create_resource(
        move || (summoner_id.get(), get_page_number()),
        |(summoner_id, page_number)| async move {
            let summoner_id = summoner_id.unwrap();
            println!("CLIENT:summoner_id: {}, page_number: {}", summoner_id, page_number);
            get_summoner_matches(summoner_id, page_number).await
        });

    view! {
        <div class="w-[740px] ">
            <Transition fallback=move|| view! { <p>"Loading..."</p>}>
                {move||{
                    match matches.get(){
                        Some(Ok(data)) => {
                                set_internal_matches(data.matches);
                                let current_page = get_page_number();
                                let mut inner_pages: Vec<PageItem> = vec![];

                                // First page
                                // Previous page
                                if current_page > 0{
                                    if current_page > 1{
                                        inner_pages.push(PageItem {
                                            label: "First".to_string(),
                                            page: 0,
                                            disabled: current_page == 0,
                                            is_current: false,
                                        });
                                    }
                                    inner_pages.push(PageItem {
                                        label: "Prev".to_string(),
                                        page: current_page - 1,
                                        disabled: current_page == 0,
                                        is_current: false,
                                    });
                                }


                                // Page numbers (-2, -1, current, +1, +2)
                                let start_page = if current_page > 2 { current_page - 2 } else { 0 };
                                let end_page = if current_page + 2 < data.total_pages { current_page + 2 } else { data.total_pages - 1 };

                                for page in start_page..=end_page {
                                    inner_pages.push(PageItem {
                                        label: format!("{}", page),
                                        page,
                                        disabled: false,
                                        is_current: page == current_page,
                                    });
                                }

                                if current_page < data.total_pages-1{
                                    inner_pages.push(PageItem {
                                        label: "Next".to_string(),
                                        page: current_page + 1,
                                        disabled: current_page >= data.total_pages - 1,
                                        is_current: false,
                                    });
                                    if current_page < data.total_pages-2{
                                        inner_pages.push(PageItem {
                                            label: "Last".to_string(),
                                            page: data.total_pages - 1,
                                            disabled: current_page >= data.total_pages - 1,
                                            is_current: false,
                                        });
                                    }
                                }
                                set_internal_pages(inner_pages);
                                view!{
                                    <div class="space-y-2 mx-2 text-gray-400">
                                        <For
                                            each=internal_matches
                                            key=|state|state.match_id
                                            children = move |child|{view!{
                                               <MatchCard match_ =child/>
                                            }}
                                        />
                                        <nav aria-label="matches navigation" >
                                             <ul class="inline-flex -space-x-px text-sm mb-6" >
                                                <For
                                                    each=internal_pages
                                                    key=|state| state.label.clone()
                                                    children=move |item| {
                                                        view! {
                                                            <li>
                                                                <a
                                                                    href={format!("?page={}", item.page)}
                                                                    aria-disabled=item.disabled
                                                                    class=("default-pagination", !item.is_current)
                                                                    class=("selected-pagination", item.is_current)
                                                                    >
                                                                    { item.label.clone() }
                                                                </a>
                                                            </li>
                                                        }
                                                    }
                                                />
                                            </ul>
                                        </nav>
                                    </div>
                                }.into_view()
                        }
                    _ => ().into_view()
                    }
                }}
            </Transition>
        </div>
    }
}

#[component]
pub fn MatchCard(match_: LolMatchDefaultParticipantMatchesPage) -> impl IntoView {
    let (participants, _) = create_signal(match_.participants);
    view! {
        <div  class="min-h-24 w-full flex rounded text-xs" >
            <div class=("bg-rose-500", !match_.won) class=("bg-blue-500", match_.won) class="min-w-1.5 w-1.5"></div>
            <div class=("bg-rose-800", !match_.won) class=("bg-blue-800", match_.won) class="flex gap-2 py-2 px-4 w-full items-center">
                <div class="flex flex-col w-[108px] gap-2">
                    <div class="flex flex-col items-start">
                        <div class=("text-rose-500", !match_.won) class=("text-blue-500", match_.won) class="uppercase font-bold text-ellipsis max-w-[90%] overflow-hidden whitespace-nowrap">{match_.queue}</div>
                        <div>{match_.match_ended_since}</div>
                    </div>
                    <hr class=("border-rose-600", !match_.won) class=("border-blue-600", match_.won) class="w-1/2"/>
                    <div class="flex flex-col items-start">
                        <div>{if match_.won {"Victory"} else {"Defeat"}}</div>
                        <div>{match_.match_duration}</div>
                    </div>
                </div>
                <div class="flex flex-col h-full w-[378px]  gap-0.5 justify-start">
                    <div class="flex items-center gap-2.5">
                        <div class="relative flex">
                            <img width="48" height="48" src={match_.champion_img_url.clone()} class="w-12 h-12 rounded-full"/>
                            <span class="absolute right-0 bottom-0 flex w-[20px] h-[20px] justify-center items-center bg-gray-700 text-white rounded-full" style="font-size:11px">{match_.champ_level}</span>
                        </div>
                        <div class="gap-0.5 flex">
                            <div class="flex flex-col gap-0.5">
                                <div class="relative rounded">
                                    <img width="22" height="22" src={match_.summoner_spell_img_url1.clone()} class="w-[22px] w-[22px]"/>
                                </div>
                                <div class="relative rounded">
                                    <img width="22" height="22" src={match_.summoner_spell_img_url2.clone()} class="w-[22px] w-[22px]"/>
                                </div>
                            </div>
                            <div class="flex flex-col gap-0.5">
                                <div class="relative rounded-full">
                                    <img width="22" height="22" src={match_.perk_primary_selection_url.clone()} class="w-[22px] w-[22px]"/>
                                </div>
                                <div class="relative rounded-full">
                                    <img width="22" height="22" src={match_.perk_sub_style_img_url.clone()} class="w-[22px] w-[22px]"/>
                                </div>
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
                            <div>
                                {match_.kda}:1 KDA
                            </div>
                        </div>
                        <div class=("border-rose-600", !match_.won) class=("border-blue-600", match_.won) class="flex flex-col h-[58px] pl-2 border-l-2">
                            <div class="text-rose-500">P/Kill {(match_.kill_participation * 100.0).round()}%</div>
                        </div>
                    </div>
                    <div class="flex gap-0.5">
                        <Show when=move ||match_.item0_id != 0 fallback=|| view!{}>
                            <div class="relative rounded">
                                <img width="20" height="22" src={match_.item0_img_url.clone()} class="w-[22px] w-[22px]"/>
                            </div>
                        </Show>
                        <Show when=move ||match_.item1_id != 0 fallback=|| view!{}>
                            <div class="relative rounded">
                                <img width="20" height="22" src={match_.item1_img_url.clone()} class="w-[22px] w-[22px]"/>
                            </div>
                        </Show>
                        <Show when=move ||match_.item2_id != 0 fallback=|| view!{}>
                            <div class="relative rounded">
                                <img width="20" height="22" src={match_.item2_img_url.clone()} class="w-[22px] w-[22px]"/>
                            </div>
                        </Show>
                        <Show when=move ||match_.item3_id != 0 fallback=|| view!{}>
                            <div class="relative rounded">
                                <img width="20" height="22" src={match_.item3_img_url.clone()} class="w-[22px] w-[22px]"/>
                            </div>
                        </Show>
                        <Show when=move ||match_.item4_id != 0 fallback=|| view!{}>
                            <div class="relative rounded">
                                <img width="20" height="22" src={match_.item4_img_url.clone()} class="w-[22px] w-[22px]"/>
                            </div>
                        </Show>
                        <Show when=move ||match_.item5_id != 0 fallback=|| view!{}>
                            <div class="relative rounded">
                                <img width="20" height="22" src={match_.item5_img_url.clone()} class="w-[22px] w-[22px]"/>
                            </div>
                        </Show>
                        <Show when=move ||match_.item6_id != 0 fallback=|| view!{}>
                            <div class="relative rounded">
                                <img width="20" height="22" src={match_.item6_img_url.clone()} class="w-[22px] w-[22px]"/>
                            </div>
                        </Show>
                    </div>
                </div>
                <div class="flex gap-x-2 gap-y-0.5 w-[168px] max-h-[89px]" style="flex-flow:column wrap">
                    <For
                        each=participants
                        key=|participant|(participant.lol_match_id, participant.summoner_id)
                        children = move |child|view!{
                            <Show
                                when=move ||{child.team_id == 100}
                                fallback= || view!{}
                                 >
                                    <div class="flex items-center gap-1">
                                        <img width="16" height="16" src={child.champion_img_url.clone()} class="w-4 h-4 rounded"/>
                                        <a href={format!("/{}/summoners/{}-{}/matches", child.summoner_platform, child.summoner_name, child.summoner_tag_line)} class=("text-white", child.summoner_id == match_.summoner_id) class="text-ellipsis overflow-hidden whitespace-nowrap max-w-[60px]">{child.summoner_name.clone()}</a>
                                    </div>
                            </Show>
                        }
                    />
                    <For
                        each=participants
                        key=|participant|(participant.lol_match_id, participant.summoner_id)
                        children = move |child|view!{
                            <Show
                                when=move ||{child.team_id == 200}
                                fallback= || view!{}
                                 >
                                    <div class="flex items-center gap-1">
                                        <img width="16" height="16" src={child.champion_img_url.clone()} class="w-4 h-4 rounded"/>
                                        <a href={format!("/{}/summoners/{}-{}/matches", child.summoner_platform, child.summoner_name, child.summoner_tag_line)}  class=("text-white", child.summoner_id == match_.summoner_id) class="text-ellipsis overflow-hidden whitespace-nowrap max-w-[60px]">{child.summoner_name.clone()}</a>
                                    </div>
                            </Show>
                        }
                    />
                </div>
            </div>
            <div class="w-[40px] flex relative flex-col">
                <button class=("bg-rose-600", !match_.won) class=("bg-blue-600", match_.won) class="p-2 flex flex-col items-center justify-end h-full">
                    <span class="w-[24px] h-[24px]" class=("text-rose-500", !match_.won) class=("text-blue-500", match_.won)>
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                            <g fill="currentColor" fill-rule="evenodd">
                                <g fill="currentColor" fill-rule="nonzero">
                                    <g fill="currentColor">
                                        <path d="M12 13.2L16.5 9 18 10.4 12 16 6 10.4 7.5 9z" transform="translate(-64 -228) translate(64 228)" fill="currentColor"></path>
                                    </g>
                                </g>
                            </g>
                        </svg>
                    </span>
                </button>
            </div>
        </div>
    }
}


#[server(GetSummonerMatches, "/api")]
async fn get_summoner_matches(summoner_id: i32, page_number: i64) -> Result<GetSummonerMatchesResult, ServerFnError> {
    println!("SERVER:summoner_id: {}, page_number: {}", summoner_id, page_number);
    let state = expect_context::<AppState>();
    let db = state.db.clone();
    let (matches, total_pages) = LolMatchParticipant::get_match_participant_for_matches_page(&db, summoner_id, page_number).await.unwrap();
    Ok(GetSummonerMatchesResult { matches, total_pages })
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetSummonerMatchesResult {
    pub matches: Vec<LolMatchDefaultParticipantMatchesPage>,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PageItem {
    label: String,
    page: i64,
    disabled: bool,
    is_current: bool,
}