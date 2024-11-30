use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_matches::get_matches;
use common::consts::champion::Champion;
use common::consts::item::Item;
use common::consts::perk::Perk;
use common::consts::platform_route::PlatformRoute;
use common::consts::queue::Queue;
use common::consts::summoner_spell::SummonerSpell;
use common::consts::{HasStaticBgAsset};
use crate::utils::{
    calculate_and_format_kda, calculate_loss_and_win_rate, format_duration,
    format_float_to_2digits, summoner_encounter_url, summoner_url, DurationSince, ProPlayerSlug,
    RiotMatchId,
};
use crate::views::components::pagination::Pagination;
use crate::views::summoner_page::match_details::MatchDetails;
use crate::views::summoner_page::Summoner;
use crate::views::{BackEndMatchFiltersSearch, ImgBg, ImgOptBg};
use leptos::either::Either;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::server_fn::rkyv::{Archive, Deserialize, Serialize};
use leptos::{component, view, IntoView};
use leptos_router::hooks::query_signal_with_options;
use leptos_router::NavigateOptions;

#[component]
pub fn SummonerMatchesPage() -> impl IntoView {
    let summoner = expect_context::<Summoner>();
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
        summoner.game_name.as_str(),
        summoner.tag_line.as_str()
    ));
    meta_store.description().set(format!("Explore {}#{}'s match history on Broken.gg. Analyze detailed League Of Legends stats, KDA ratios, and performance metrics on our high-speed, resource-efficient platform.", summoner.game_name.as_str(), summoner.tag_line.as_str()));
    meta_store.url().set(summoner.to_route_path());
    view! {
        <div class="w-[768px] inline-block align-top justify-center">
            <div class="">
                <Transition fallback=move || {
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
                                                        <MatchCard match_=match_ />
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
                </Transition>
            </div>
        </div>
    }
}

#[component]
pub fn MatchCard(match_: SummonerMatch) -> impl IntoView {
    let summoner = expect_context::<Summoner>();
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
    let items = vec![
        match_.item0_id,
        match_.item1_id,
        match_.item2_id,
        match_.item3_id,
        match_.item4_id,
        match_.item5_id,
        match_.item6_id,
    ].iter()
        .filter_map(|id| Item::try_from(*id).ok())
        .collect::<Vec<_>>();

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
                    <MatchInfoCard
                        won=match_.won
                        queue=match_.queue
                        match_ended_since=match_.match_ended_since
                        match_duration=match_.match_duration
                    />
                    <MatchSummonerCard
                        champion=champion
                        champ_level=match_.champ_level
                        summoner_spell1=summoner_spell1
                        summoner_spell2=summoner_spell2
                        primary_perk_selection=primary_perk_selection
                        sub_perk_style=sub_perk_style
                        kills=match_.kills
                        deaths=match_.deaths
                        assists=match_.assists
                        won=match_.won
                        kill_participation=match_.kill_participation
                        items=items
                    />

                    <div
                        class="flex gap-x-2 gap-y-0.5 w-[266px] max-h-[89px]"
                        style="flex-flow:column wrap"
                    >
                        <For
                            each=move || match_.participants.clone()
                            key=|match_| match_.summoner_id
                            let:participant
                        >
                            {
                                let participant: SummonerMatchParticipant = participant;
                                let is_pro_player = participant.pro_player_slug.is_some();
                                let champion = Champion::from(participant.champion_id);
                                let p_gn_clone = participant.game_name.clone();
                                let p_tl_clone = participant.tag_line.clone();
                                let s_gn_clone = summoner.game_name.clone();
                                let s_tl_clone = summoner.tag_line.clone();
                                view! {
                                    <div class="flex items-center gap-1 w-[130px]">
                                        <div class="sprite-wrapper  w-4 h-4">
                                            <ImgBg
                                                alt=champion.to_str().to_string()
                                                class=format!(
                                                    "rounded scale-33 sprite-inner {}",
                                                    champion.get_class_name(),
                                                )
                                            />
                                        </div>

                                        <Show when=move || (participant.encounter_count > 1)>
                                            <a
                                                href=summoner_encounter_url(
                                                    summoner.platform.as_ref(),
                                                    s_gn_clone.as_str(),
                                                    s_tl_clone.as_str(),
                                                    participant.platform.as_ref(),
                                                    p_gn_clone.as_str(),
                                                    p_tl_clone.as_str(),
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
                                                    participant.pro_player_slug.unwrap().as_ref(),
                                                )
                                                class="text-xs bg-purple-800 rounded px-0.5 text-center"
                                            >
                                                pro
                                            </a>
                                        </Show>
                                        <a
                                            target="_blank"
                                            href=summoner_url(
                                                participant.platform.as_ref(),
                                                participant.game_name.as_str(),
                                                participant.tag_line.as_str(),
                                            )
                                            class:text-white=move || {
                                                participant.summoner_id == match_.summoner_id
                                            }
                                            class="text-ellipsis overflow-hidden whitespace-nowrap "
                                        >
                                            {participant.game_name.clone()}
                                        </a>
                                    </div>
                                }
                            }
                        </For>
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
                    riot_match_id=match_.riot_match_id
                    platform=match_.platform
                />
            </Show>
        </div>
    }
}


#[component]
pub fn MatchSummonerCard(
    champion:Champion,
    champ_level:u16,
    summoner_spell1:SummonerSpell,
    summoner_spell2:SummonerSpell,
    primary_perk_selection:Perk,
    sub_perk_style:Perk,
    kills:u16,
    deaths:u16,
    assists:u16,
    won:bool,
    kill_participation:u16,
    items:Vec<Item>,
    #[prop(optional)]
    encounter_is_self:Option<bool>
) -> impl IntoView{
    let has_encounter = move|| encounter_is_self.is_some();
    let is_self = move|| has_encounter() && encounter_is_self.unwrap();
    let is_encounter = move|| has_encounter() && !encounter_is_self.unwrap();
    view! {
        <div
            class="flex flex-col h-full w-[305px] gap-0.5 justify-center "
            class=("px-2", has_encounter)
            class=("bg-red-900", move || !won)
            class=("bg-blue-900", move || won)
            class=("rounded-r-lg", is_encounter)
            class=("border-l-2", is_encounter)
            class=("border-gray-800", is_encounter)
            class=("rounded-l-lg", is_self)
        >
            <div class="flex items-center gap-2.5" class=("flex-row-reverse", is_encounter)>
                <div class="relative flex">
                    <ImgBg
                        alt=champion.to_str().to_string()
                        class=format!("rounded-full {}", champion.get_class_name())
                    />
                    <span
                        class="absolute right-0 bottom-0 flex w-[20px] h-[20px] justify-center items-center bg-gray-800 text-white rounded-full"
                        style="font-size:11px"
                    >
                        {champ_level}
                    </span>
                </div>
                <div class="gap-0.5 flex">
                    <div class="flex flex-col gap-0.5 items-center">

                        <ImgBg
                            alt=summoner_spell1.to_string()
                            class=format!("rounded {}", summoner_spell1.get_class_name())
                        />
                        <ImgBg
                            alt=summoner_spell2.to_string()
                            class=format!("rounded {}", summoner_spell2.get_class_name())
                        />
                    </div>
                    <div class="flex flex-col gap-0.5 items-center">
                        <div class="w-[22px] h-[22px] sprite-wrapper">
                            <ImgOptBg
                                when=move || primary_perk_selection != Perk::UNKNOWN
                                alt=primary_perk_selection.to_string()
                                class=format!(
                                    "scale-78 sprite-inner rounded-full {}",
                                    primary_perk_selection.get_class_name(),
                                )
                            />
                        </div>
                        <div class="w-[22px] h-[22px] sprite-wrapper">
                            <ImgOptBg
                                when=move || sub_perk_style != Perk::UNKNOWN
                                alt=sub_perk_style.to_string()
                                class=format!(
                                    "scale-78 sprite-inner rounded-full {}",
                                    sub_perk_style.get_class_name(),
                                )
                            />
                        </div>

                    </div>
                </div>
                <div class="flex flex-col w-[85px] items-start gap-1">
                    <div class="text-base">
                        <span class="text-white">{kills}</span>
                        /
                        <span class="text-red-300">{deaths}</span>
                        /
                        <span class="text-white">{assists}</span>
                    </div>
                    <div>{calculate_and_format_kda(kills, deaths, assists)}:1 KDA</div>
                </div>
                <div
                    class="flex flex-col h-[58px] "
                    class=("border-l-2", move || !is_encounter())
                    class=("pl-2", move || !is_encounter())
                    class=("border-r-2", is_encounter)
                    class=("pr-2", is_encounter)
                    class=("border-red-500", move || !won)
                    class=("border-blue-500", move || won)
                >
                    <div class="text-red-300">P/Kill {kill_participation}%</div>
                </div>
            </div>
            <div class="flex gap-0.5" class=("flex-row-reverse", is_encounter)>
                {items
                    .iter()
                    .map(|item| {
                        view! {
                            <ImgBg
                                alt=item.to_string()
                                class=format!("rounded {}", item.get_class_name())
                            />
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
pub fn MatchInfoCard(
    #[prop(optional)]
    won:Option<bool>,
    queue:Queue,
    match_ended_since:DurationSince,
    match_duration:Option<i32>,

) -> impl IntoView{
    view! {
        <div class="flex flex-col w-[108px] gap-2">
            <div class="flex flex-col items-start w-[108px]">
                <div
                    class:text-red-300=move || won.is_some() && !won.unwrap()
                    class:text-blue-300=move || won.is_some() && won.unwrap()
                    class=" uppercase font-bold text-ellipsis max-w-[90%] overflow-hidden whitespace-nowrap"
                >
                    {queue.to_str()}
                </div>
                <div>{match_ended_since.to_string()}</div>
            </div>
            <hr
                class:border-red-500=move || won.is_some() && !won.unwrap()
                class:border-blue-500=move || won.is_some() && won.unwrap()
                class="w-1/2"
            />
            <div class="flex flex-col items-start w-[108px]">
                <Show when=move || won.is_some()>
                    <div>{if won.unwrap() { "Victory" } else { "Defeat" }}</div>
                </Show>
                <div>{format_duration(match_duration)}</div>
            </div>
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
    pub match_duration: Option<i32>,
    pub item0_id: u32,
    pub item1_id: u32,
    pub item2_id: u32,
    pub item3_id: u32,
    pub item4_id: u32,
    pub item5_id: u32,
    pub item6_id: u32,
    pub champ_level: u16,
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
    pub game_name: String,
    pub pro_player_slug: Option<ProPlayerSlug>,
    pub tag_line: String,
    pub platform: PlatformRoute,
}
