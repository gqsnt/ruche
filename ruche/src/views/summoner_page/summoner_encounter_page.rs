use crate::app::{
    to_encounter_identifier_memo, to_summoner_identifier_memo, EncounterRouteParams, MetaStore,
    MetaStoreStoreFields, SummonerIdentifier, SummonerRouteParams,
};
use crate::backend::server_fns::get_encounter::get_encounter;
use crate::utils::{
    calculate_and_format_kda, calculate_loss_and_win_rate, format_float_to_2digits,
    items_from_slice, DurationSince, RiotMatchId,
};
use crate::views::components::pagination::Pagination;
use crate::views::summoner_page::match_details::MatchDetails;
use crate::views::summoner_page::summoner_matches_page::{MatchInfoCard, MatchSummonerCard};
use crate::views::summoner_page::{SSEMatchUpdateVersion, Summoner, SummonerInfo};
use crate::views::BackEndMatchFiltersSearch;
use bitcode::{Decode, Encode};
use common::consts::champion::Champion;
use common::consts::perk::Perk;
use common::consts::platform_route::PlatformRoute;
use common::consts::queue::Queue;
use common::consts::summoner_spell::SummonerSpell;
use leptos::either::Either;
use leptos::prelude::codee::binary::BitcodeCodec;
use leptos::prelude::*;
use leptos::{component, IntoView};
use leptos_router::hooks::use_params;
use leptos_router::{lazy_route, LazyRoute};
use reactive_stores::Store;

pub struct SummonerEncounterRoute {
    encounter_resource: Resource<Result<SummonerEncounterResult, ServerFnError>, BitcodeCodec>,
    is_with: RwSignal<bool>,
    summoner_identifier_memo: Memo<SummonerIdentifier>,
    encounter_identifier_memo: Memo<SummonerIdentifier>,
}

#[lazy_route]
impl LazyRoute for SummonerEncounterRoute {
    fn data() -> Self {
        let is_with = RwSignal::new(true);
        let summoner_route_params = use_params::<SummonerRouteParams>();
        let summoner_identifier_memo = to_summoner_identifier_memo(summoner_route_params);

        let encounter_route_params = use_params::<EncounterRouteParams>();
        let encounter_identifier_memo = to_encounter_identifier_memo(encounter_route_params);

        let sse_match_update_version = expect_context::<RwSignal<Option<SSEMatchUpdateVersion>>>();
        let match_filters = expect_context::<Store<BackEndMatchFiltersSearch>>();

        let encounter_resource = leptos::server::Resource::new_bitcode(
            move || {
                (
                    sse_match_update_version.get().unwrap_or_default(),
                    summoner_identifier_memo.get(),
                    encounter_identifier_memo.get(),
                    match_filters.get(),
                    is_with.get(),
                )
            },
            |(_, summoner, encounter, filters, is_with)| async move {
                //println!("{:?} {:?} {:?}", filters, summoner, page_number);
                get_encounter(summoner, encounter, is_with, Some(filters)).await
            },
        );
        Self {
            encounter_resource,
            is_with,
            summoner_identifier_memo,
            encounter_identifier_memo,
        }
    }

    fn view(this: Self) -> AnyView {
        let SummonerEncounterRoute {
            encounter_resource,
            is_with,
            summoner_identifier_memo,
            encounter_identifier_memo,
        } = this;
        let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

        batch(|| {
            let me = summoner_identifier_memo.read();
            let opp = encounter_identifier_memo.read();

            // Build canonical URL that matches router:
            // /summoners/:platform/:me_slug/encounter/:opp_platform/:opp_slug
            let me_slug = format!("{}-{}", me.game_name, me.tag_line);
            let opp_slug = format!("{}-{}", opp.game_name, opp.tag_line);
            let canonical = format!(
                "/summoners/{}/{}/encounter/{}/{}",
                me.platform_route.code(),
                me_slug,
                opp.platform_route.code(),
                opp_slug
            );

            meta_store.title().set(format!("{}#{} vs {}#{} — Head-to-Head | Ruche", me.game_name, me.tag_line, opp.game_name, opp.tag_line));
            meta_store.description().set(format!(
                "Head-to-head: {}#{} vs {}#{}. Wins/losses together and against, shared matches, and detailed performance metrics. Updated in near real time.",
                me.game_name, me.tag_line, opp.game_name, opp.tag_line
            ));
            meta_store.url().set(canonical);
        });

        view! {
            <div class="flex my-card justify-center space-x-2 my-2">
                <button

                    class="w-[22rem] "
                    class=("active-tab", move || is_with.get())
                    class=("default-tab", move || !is_with.get())
                    on:click=move |_| is_with.set(true)
                >
                    With
                </button>
                <button
                    class="w-[22rem] "
                    class=("active-tab", move || !is_with.get())
                    class=("default-tab", move || is_with.get())
                    on:click=move |_| is_with.set(false)
                >
                    VS
                </button>
            </div>
            <div class="w-[768px]">
                <Transition fallback=move || {
                    view! { <div class="text-center">Loading Encounter</div> }
                }>
                    {move || Suspend::new(async move {
                        match encounter_resource.await {
                            Ok(encounter_result) => {
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
                                                <div class="flex flex-col">
                                                    <div class="flex w-full my-card justify-between mb-1">
                                                        <SummonerEncounterInfo
                                                            summoner=encounter_result.summoner
                                                            is_self=true
                                                        />
                                                        <SummonerEncounterInfo
                                                            summoner=encounter_result.encounter
                                                            is_self=false
                                                        />
                                                    </div>
                                                    <div class="flex w-full my-card justify-between">
                                                        <SummonerEncounterStats
                                                            stats=encounter_result.summoner_stats.clone()
                                                            is_self=true
                                                        />
                                                        <SummonerEncounterStats
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
                                                        <SummonerEncounterMatchComponent match_=match_ />
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
                </Transition>
            </div>
        }.into_any()
    }
}

#[component]
pub fn SummonerEncounterMatchComponent(match_: SummonerEncounterMatch) -> impl IntoView {
    let (show_details, set_show_details) = signal(false);

    let self_items = items_from_slice(&[
        match_.participant.item0_id,
        match_.participant.item1_id,
        match_.participant.item2_id,
        match_.participant.item3_id,
        match_.participant.item4_id,
        match_.participant.item5_id,
        match_.participant.item6_id,
    ]);
    let encounter_items = items_from_slice(&[
        match_.encounter.item0_id,
        match_.encounter.item1_id,
        match_.encounter.item2_id,
        match_.encounter.item3_id,
        match_.encounter.item4_id,
        match_.encounter.item5_id,
        match_.encounter.item6_id,
    ]);
    let (champion, encounter_champion) = (
        Champion::try_from(match_.participant.champion_id).unwrap_or_default(),
        Champion::try_from(match_.encounter.champion_id).unwrap_or_default(),
    );
    let (summoner_spell1, summoner_spell2) = (
        SummonerSpell::try_from(match_.participant.summoner_spell1_id).unwrap_or_default(),
        SummonerSpell::try_from(match_.participant.summoner_spell2_id).unwrap_or_default(),
    );
    let (encounter_summoner_spell1, encounter_summoner_spell2) = (
        SummonerSpell::try_from(match_.encounter.summoner_spell1_id).unwrap_or_default(),
        SummonerSpell::try_from(match_.encounter.summoner_spell2_id).unwrap_or_default(),
    );
    let (primary_perk_selection, encounter_primary_perk_selection) = (
        Perk::try_from(match_.participant.perk_primary_selection_id).unwrap_or_default(),
        Perk::try_from(match_.encounter.perk_primary_selection_id).unwrap_or_default(),
    );
    let (sub_perk_style, encounter_sub_perk_style) = (
        Perk::try_from(match_.participant.perk_sub_style_id).unwrap_or_default(),
        Perk::try_from(match_.encounter.perk_sub_style_id).unwrap_or_default(),
    );

    view! {
        <div class="flex flex-col">
            <div class="flex  my-card w-[768px]">
                <MatchInfoCard
                    queue=match_.queue
                    match_ended_since=match_.match_ended_since
                    match_duration=match_.match_duration
                />
                <div class="flex w-full">
                    <MatchSummonerCard
                        items=self_items
                        kills=match_.participant.kills
                        deaths=match_.participant.deaths
                        assists=match_.participant.assists
                        kill_participation=match_.participant.kill_participation
                        won=match_.participant.won
                        champ_level=match_.participant.champ_level
                        champion=champion
                        summoner_spell1=summoner_spell1
                        summoner_spell2=summoner_spell2
                        primary_perk_selection=primary_perk_selection
                        sub_perk_style=sub_perk_style
                        encounter_is_self=true
                    />
                    <MatchSummonerCard
                        items=encounter_items
                        kills=match_.encounter.kills
                        deaths=match_.encounter.deaths
                        assists=match_.encounter.assists
                        kill_participation=match_.encounter.kill_participation
                        won=match_.encounter.won
                        champ_level=match_.encounter.champ_level
                        champion=encounter_champion
                        summoner_spell1=encounter_summoner_spell1
                        summoner_spell2=encounter_summoner_spell2
                        primary_perk_selection=encounter_primary_perk_selection
                        sub_perk_style=encounter_sub_perk_style
                        encounter_is_self=false
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
                    riot_match_id=match_.riot_match_id
                    platform=match_.platform
                    in_encounter=true
                />
            </Show>
        </div>
    }
}

#[component]
pub fn SummonerEncounterInfo(summoner: Summoner, is_self: bool) -> impl IntoView {
    let (summoner_level, _) = signal(summoner.summoner_level);
    let (profile_icon_id, _) = signal(summoner.profile_icon_id);
    view! {
        <div class="flex w-1/2 " class=("flex-row-reverse", move || !is_self)>
            <SummonerInfo
                game_name=summoner.game_name
                tag_line=summoner.tag_line
                platform=summoner.platform
                pro_slug=summoner.pro_slug
                level_signal=summoner_level
                profile_icon_signal=profile_icon_id
                is_self=is_self
            />
        </div>
    }
}

#[component]
pub fn SummonerEncounterStats(stats: SummonerEncounterStats, is_self: bool) -> impl IntoView {
    let (losses, winrate) = calculate_loss_and_win_rate(stats.total_wins, stats.total_matches);
    view! {
        <div
            class="flex flex-col text-sm w-[40%] "
            class=("ml-2", move || is_self)
            class=("mr-2", move || !is_self)
            class=("text-right", move || !is_self)
            class=("text-left", move || is_self)
        >
            <div>
                {stats.total_wins}W {losses as u16}L {stats.total_matches}G
                {format_float_to_2digits(winrate.round())}%
            </div>
            <div class="flex flex-col">
                <div>
                    {format!(
                        "{}/{}/{}",
                        format_float_to_2digits(stats.avg_kills),
                        format_float_to_2digits(stats.avg_deaths),
                        format_float_to_2digits(stats.avg_assists),
                    )}
                </div>
                <div>
                    {calculate_and_format_kda(stats.avg_kills, stats.avg_deaths, stats.avg_assists)}
                    :1 P/kill {stats.avg_kill_participation}%
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Encode, Decode)]
pub struct SummonerEncounterResult {
    pub total_pages: u16,
    pub matches: Vec<SummonerEncounterMatch>,
    pub summoner_stats: SummonerEncounterStats,
    pub encounter_stats: SummonerEncounterStats,
    pub summoner: Summoner,
    pub encounter: Summoner,
}

#[derive(Clone, Encode, Decode, Default)]
pub struct SummonerEncounterStats {
    pub avg_kills: f32,
    pub avg_deaths: f32,
    pub avg_assists: f32,
    pub avg_kill_participation: u16,
    pub total_wins: u16,
    pub total_matches: u16,
}

#[derive(Clone, Encode, Decode)]
pub struct SummonerEncounterParticipant {
    pub summoner_id: i32,
    pub item0_id: u32,
    pub item1_id: u32,
    pub item2_id: u32,
    pub item3_id: u32,
    pub item4_id: u32,
    pub item5_id: u32,
    pub item6_id: u32,
    pub kill_participation: u16,
    pub champion_id: u16,
    pub champ_level: u16,
    pub kills: u16,
    pub deaths: u16,
    pub assists: u16,
    pub summoner_spell1_id: u16,
    pub summoner_spell2_id: u16,
    pub perk_primary_selection_id: u16,
    pub perk_sub_style_id: u16,
    pub won: bool,
}

#[derive(Clone, Encode, Decode)]
pub struct SummonerEncounterMatch {
    pub match_id: i32,
    pub match_duration: Option<i32>,
    pub platform: PlatformRoute,
    pub queue: Queue,
    pub riot_match_id: RiotMatchId,
    pub match_ended_since: DurationSince,
    pub participant: SummonerEncounterParticipant,
    pub encounter: SummonerEncounterParticipant,
}
