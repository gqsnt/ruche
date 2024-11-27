use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_live_game::get_live_game;
use crate::consts::champion::Champion;
use crate::consts::map::Map;
use crate::consts::perk::Perk;
use crate::consts::platform_route::PlatformRoute;
use crate::consts::queue::Queue;
use crate::consts::summoner_spell::SummonerSpell;
use crate::consts::HasStaticAsset;
use crate::utils::{
    calculate_and_format_kda, calculate_loss_and_win_rate, format_float_to_2digits,
    summoner_encounter_url, summoner_url, ProPlayerSlug, Puuid, RiotMatchId,
};
use crate::views::summoner_page::Summoner;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::server_fn::rkyv::{Archive, Deserialize, Serialize};
use leptos::{component, view, IntoView};

#[component]
pub fn SummonerLivePage() -> impl IntoView {
    let summoner = expect_context::<Summoner>();
    let summoner_update_version = expect_context::<ReadSignal<Option<u16>>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

    let (refresh_signal, set_refresh_signal) = signal(0);

    let live_game_resource = Resource::new_rkyv(
        move || {
            (
                summoner_update_version.get().unwrap_or_default(),
                refresh_signal.get(),
                summoner.id,
                summoner.platform.to_string(),
            )
        },
        |(_, _, id, platform_type)| async move {
            get_live_game(id, PlatformRoute::from(platform_type.as_str())).await
        },
    );

    meta_store.title().set(format!(
        "{}#{} | Live Game | Broken.gg",
        summoner.game_name.as_str(),
        summoner.tag_line.as_str()
    ));
    meta_store.description().set(format!("Watch {}#{}'s live game now on Broken.gg. Get real-time updates and analytics with our ultra-fast, Rust-based League of Legends companion.", summoner.game_name.as_str(), summoner.tag_line.as_str()));
    meta_store
        .url()
        .set(format!("{}?tab=live", summoner.to_route_path()));
    view! {
        <div class="w-[768px]">
            <div class="flex justify-start mb-2">
                <button
                    class="my-button"
                    on:click=move |_| { set_refresh_signal(refresh_signal() + 1) }
                >
                    Refresh
                </button>
            </div>
            <Suspense fallback=move || {
                view! { <div class="text-center">Not In Live Game</div> }
            }>
                {move || Suspend::new(async move {
                    match live_game_resource.await {
                        Ok(Some(result)) => {
                            let first_team = result
                                .participants
                                .iter()
                                .filter(|participant| participant.team_id == 100)
                                .cloned()
                                .collect::<Vec<_>>();
                            let second_team = result
                                .participants
                                .iter()
                                .filter(|participant| participant.team_id == 200)
                                .cloned()
                                .collect::<Vec<_>>();
                            Either::Right(
                                view! {
                                    <div class="flex flex-col space-y-2">
                                        <div class="flex space-x-2">
                                            <div>{result.queue.to_str()}</div>
                                            <div>{result.game_map.get_static_name()}</div>
                                            <div>
                                                {format!(
                                                    "{:02}:{:02}",
                                                    result.game_length / 60,
                                                    result.game_length % 60,
                                                )}
                                            </div>
                                        </div>
                                        <MatchLiveTable team_id=100 participants=first_team />
                                        <MatchLiveTable team_id=200 participants=second_team />

                                    </div>
                                },
                            )
                        }
                        _ => Either::Left(view! { <div class="text-center">Not In Live Game</div> }),
                    }
                })}
            </Suspense>
        </div>
    }
}

#[component]
pub fn MatchLiveTable(team_id: i32, participants: Vec<LiveGameParticipant>) -> impl IntoView {
    let summoner = expect_context::<Summoner>();
    let is_blue_team = || team_id == 100;

    view! {
        <table class="table-fixed text-xs w-full">
            <colgroup>
                <col width="44" />
                <col width="16" />
                <col width="15" />
                <col />
                <col width="32" />
                <col width="132" />
                <col width="124" />
                <col width="100" />
                <col width="100" />
            </colgroup>
            <thead>
                <tr>
                    <th
                        colspan="3"
                        class=("text-blue-800", is_blue_team())
                        class=("text-red-800", !is_blue_team())
                    >
                        {if is_blue_team() { "Blue Team" } else { "Red Team" }}
                    </th>
                    <th class="text-left"></th>
                    <th></th>
                    <th>S2024</th>
                    <th>Ranked Stats</th>
                    <th colspan="2">Champion Stats</th>

                </tr>
            </thead>
            <tbody>
                {participants
                    .into_iter()
                    .map(|participant| {
                        let is_pro_player = participant.pro_player_slug.is_some();
                        let champion = Champion::from(participant.champion_id);
                        let summoner_spell1 = SummonerSpell::from(participant.summoner_spell1_id);
                        let summoner_spell2 = SummonerSpell::from(participant.summoner_spell2_id);
                        let perk_primary_selection = Perk::from(
                            participant.perk_primary_selection_id,
                        );
                        let perk_sub_style = Perk::from(participant.perk_sub_style_id);
                        let participant_game_name_clone = participant.game_name.clone();
                        let participant_tag_line_clone = participant.tag_line.clone();
                        let summoner_game_name_clone = summoner.game_name.clone();
                        let summoner_tag_line_clone = summoner.tag_line.clone();

                        view! {
                            <tr>
                                <td
                                    class="border-l-2 pl-2.5 py-1 "
                                    class=("border-red-500", !is_blue_team())
                                    class=("border-blue-500", is_blue_team())
                                >

                                    <div class="relative w-8">
                                        <img
                                            width="32"
                                            height="32"
                                            alt=champion.to_str()
                                            src=champion.get_static_asset_url()
                                            class="w-8 h-8 rounded-full block"
                                        />
                                    </div>
                                </td>
                                <td class="py-1">
                                    <div class="relative">
                                        <img
                                            width="16"
                                            height="16"
                                            alt=summoner_spell1.to_string()
                                            src=summoner_spell1.get_static_asset_url()
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                    <div class="relative">
                                        <img
                                            width="16"
                                            height="16"
                                            alt=summoner_spell2.to_string()
                                            src=summoner_spell2.get_static_asset_url()
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                </td>
                                <td class="py-1">
                                    <div class="relative">
                                        <img
                                            alt=perk_primary_selection.to_string()
                                            width="16"
                                            height="16"
                                            src=perk_primary_selection.get_static_asset_url()
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                    <div class="relative">
                                        <img
                                            width="16"
                                            height="16"
                                            alt=perk_sub_style.to_string()
                                            src=perk_sub_style.get_static_asset_url()
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                </td>
                                <td class="pl-[5px] py-1 text-ellipsis overflow-hidden text-left">
                                    <div class="flex items-center gap-1">
                                        <Show when=move || (participant.encounter_count > 0)>
                                            <a
                                                href=summoner_encounter_url(
                                                    summoner.platform.as_ref(),
                                                    summoner_game_name_clone.as_str(),
                                                    summoner_tag_line_clone.as_str(),
                                                    participant.platform.as_ref(),
                                                    participant_game_name_clone.as_str(),
                                                    participant_tag_line_clone.as_str(),
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
                                        >
                                            {participant.game_name.clone()}
                                            #
                                            {participant.tag_line.clone()}
                                        </a>
                                    </div>
                                    <span class="text-[11px]">
                                        Lvl. {participant.summoner_level}
                                    </span>
                                </td>
                                <td></td>
                                <td></td>
                                <td class="py-1">
                                    {match participant.ranked_stats {
                                        Some(ranked_stats) => {
                                            let (losses, win_rate) = calculate_loss_and_win_rate(
                                                ranked_stats.total_ranked,
                                                ranked_stats.total_ranked_wins,
                                            );
                                            Either::Left(
                                                view! {
                                                    <div>
                                                        {format_float_to_2digits(win_rate)}%
                                                        {ranked_stats.total_ranked}G
                                                    </div>
                                                    <div>
                                                        {ranked_stats.total_ranked_wins}W {losses as u16}L
                                                    </div>
                                                },
                                            )
                                        }
                                        None => {
                                            Either::Right(

                                                view! { <div>-</div> },
                                            )
                                        }
                                    }}
                                </td>
                                <td class="py-1">
                                    {match &participant.champion_stats {
                                        Some(champion_stats) => {
                                            let (losses, win_rate) = calculate_loss_and_win_rate(
                                                champion_stats.total_champion_played,
                                                champion_stats.total_champion_wins,
                                            );
                                            Either::Left(
                                                view! {
                                                    <div>
                                                        {format_float_to_2digits(win_rate)}%
                                                        {champion_stats.total_champion_played}G
                                                    </div>
                                                    <div>
                                                        {champion_stats.total_champion_wins}W {losses as u16}L
                                                    </div>
                                                },
                                            )
                                        }
                                        None => {
                                            Either::Right(

                                                view! { <div>-</div> },
                                            )
                                        }
                                    }}
                                </td>
                                <td class="py-1">
                                    {match &participant.champion_stats {
                                        Some(champion_stats) => {
                                            Either::Left(
                                                view! {
                                                    <div>
                                                        {calculate_and_format_kda(
                                                            champion_stats.avg_kills,
                                                            champion_stats.avg_deaths,
                                                            champion_stats.avg_assists,
                                                        )}:1
                                                    </div>
                                                    <div>
                                                        {format_float_to_2digits(champion_stats.avg_kills)}/
                                                        {format_float_to_2digits(champion_stats.avg_deaths)}/
                                                        {format_float_to_2digits(champion_stats.avg_assists)}
                                                    </div>
                                                },
                                            )
                                        }
                                        None => Either::Right(view! { <div>-</div> }),
                                    }}
                                </td>
                            </tr>
                        }
                    })
                    .collect::<Vec<_>>()}
            </tbody>
        </table>
    }
}
#[derive(Clone, Serialize, Deserialize, Archive)]
pub struct LiveGame {
    pub game_length: u16,
    pub game_map: Map,
    pub queue: Queue,
    pub game_id: RiotMatchId,
    pub participants: Vec<LiveGameParticipant>,
}

#[derive(Clone, Serialize, Deserialize, Archive)]
pub struct LiveGameParticipant {
    pub summoner_id: i32,
    pub champion_id: u16,
    pub team_id: u16,
    pub encounter_count: u16,
    pub summoner_spell1_id: u16,
    pub summoner_spell2_id: u16,
    pub perk_primary_selection_id: u16,
    pub perk_sub_style_id: u16,
    pub summoner_level: u16,
    pub platform: PlatformRoute,
    pub puuid: Puuid,
    pub game_name: String,
    pub tag_line: String,
    pub pro_player_slug: Option<ProPlayerSlug>,
    pub ranked_stats: Option<LiveGameParticipantRankedStats>,
    pub champion_stats: Option<LiveGameParticipantChampionStats>,
}

#[derive(Clone, Serialize, Deserialize, Default, Archive)]
pub struct LiveGameParticipantRankedStats {
    pub total_ranked: u16,
    pub total_ranked_wins: u16,
}

#[derive(Clone, Serialize, Deserialize, Default, Archive)]
pub struct LiveGameParticipantChampionStats {
    pub total_champion_played: u16,
    pub total_champion_wins: u16,
    pub avg_kills: f32,
    pub avg_deaths: f32,
    pub avg_assists: f32,
}
