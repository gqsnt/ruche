use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_live_game::get_live_game;
use crate::consts::champion::Champion;
use crate::consts::perk::Perk;
use crate::consts::summoner_spell::SummonerSpell;
use crate::consts::HasStaticAsset;
use crate::utils::{format_float_to_2digits, summoner_encounter_url, summoner_url, GameName, ProPlayerSlug, Puuid, RiotMatchId, TagLine};
use crate::views::summoner_page::Summoner;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::server_fn::rkyv::{Deserialize, Serialize, Archive};
use leptos::{component, view, IntoView};
use crate::consts::map::Map;
use crate::consts::platform_route::PlatformRoute;
use crate::consts::queue::Queue;

#[component]
pub fn SummonerLivePage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

    let (refresh_signal, set_refresh_signal) = signal(0);

    let live_game_resource = Resource::new_rkyv(
        move || (refresh_signal.get(), summoner().puuid, summoner().id, summoner().platform.to_string()),
        |(_, puuid, id, platform_type)| async move {
            get_live_game(id, PlatformRoute::from(platform_type.as_str()),puuid).await
        },
    );

    meta_store.title().set(format!("{}#{} | Live Game | Broken.gg", summoner().game_name.to_str(), summoner().tag_line.to_str()));
    meta_store.description().set(format!("Watch {}#{}'s live game now on Broken.gg. Get real-time updates and analytics with our ultra-fast, Rust-based League of Legends companion.", summoner().game_name.to_str(), summoner().tag_line.to_str()));
    meta_store.url().set(format!("{}?tab=live", summoner().to_route_path()));
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
                                        <MatchLiveTable
                                            team_id=100
                                            participants=first_team
                                            summoner=summoner
                                        />
                                        <MatchLiveTable
                                            team_id=200
                                            participants=second_team
                                            summoner=summoner
                                        />

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
pub fn MatchLiveTable(team_id: i32, participants: Vec<LiveGameParticipant>, summoner: ReadSignal<Summoner>) -> impl IntoView {
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
                                            alt=Champion::from(participant.champion_id).to_str()
                                            src=Champion::get_static_asset_url(participant.champion_id)
                                            class="w-8 h-8 rounded-full block"
                                        />
                                    </div>
                                </td>
                                <td class="py-1">
                                    <div class="relative">
                                        <img
                                            width="16"
                                            height="16"
                                            alt=SummonerSpell::from(participant.summoner_spell1_id)
                                                .to_string()
                                            src=SummonerSpell::get_static_asset_url(
                                                participant.summoner_spell1_id,
                                            )
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                    <div class="relative">
                                        <img
                                            width="16"
                                            height="16"
                                            alt=SummonerSpell::from(participant.summoner_spell2_id)
                                                .to_string()
                                            src=SummonerSpell::get_static_asset_url(
                                                participant.summoner_spell2_id,
                                            )
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                </td>
                                <td class="py-1">
                                    <div class="relative">
                                        <img
                                            alt=Perk::from(participant.perk_primary_selection_id)
                                                .to_string()
                                            width="16"
                                            height="16"
                                            src=Perk::get_static_asset_url(
                                                participant.perk_primary_selection_id,
                                            )
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                    <div class="relative">
                                        <img
                                            width="16"
                                            height="16"
                                            alt=Perk::from(participant.perk_sub_style_id).to_string()
                                            src=Perk::get_static_asset_url(
                                                participant.perk_sub_style_id,
                                            )
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                </td>
                                <td class="pl-[5px] py-1 text-ellipsis overflow-hidden text-left">
                                    <div class="flex items-center gap-1">
                                        <Show when=move || (participant.encounter_count > 0)>
                                            <a
                                                href=summoner_encounter_url(
                                                    summoner().platform.to_string(),
                                                    summoner().game_name.to_string(),
                                                    summoner().tag_line.to_string(),
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
                                                    participant.pro_player_slug.unwrap().to_str(),
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
                                        >
                                            {participant.game_name.to_string()}
                                            #
                                            {participant.tag_line.to_string()}
                                        </a>
                                    </div>
                                    <span class="text-[11px]">
                                        Lvl. {participant.summoner_level}
                                    </span>
                                </td>
                                <td></td>
                                <td></td>
                                <td class="py-1">
                                    {match &participant.ranked_stats {
                                        Some(ranked_stats) => {
                                            Either::Left(
                                                view! {
                                                    <div>
                                                        {format_float_to_2digits(ranked_stats.ranked_win_rate)}%
                                                        {ranked_stats.total_ranked}G
                                                    </div>
                                                    <div>
                                                        {ranked_stats.total_ranked_wins}W
                                                        {ranked_stats.total_ranked_losses}L
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
                                                        {format_float_to_2digits(champion_stats.champion_win_rate)}%
                                                        {champion_stats.total_champion_played}G
                                                    </div>
                                                    <div>
                                                        {champion_stats.total_champion_wins}W
                                                        {champion_stats.total_champion_losses}L
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
                                                        {format!(
                                                            "{:.2}",
                                                            (champion_stats.avg_kills + champion_stats.avg_assists)
                                                                / champion_stats.avg_deaths.max(1.0),
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
    pub game_name: GameName,
    pub tag_line: TagLine,
    pub pro_player_slug: Option<ProPlayerSlug>,
    pub ranked_stats: Option<LiveGameParticipantRankedStats>,
    pub champion_stats: Option<LiveGameParticipantChampionStats>,
}


#[derive(Clone, Serialize, Deserialize, Default, Archive)]
pub struct LiveGameParticipantRankedStats {
    pub total_ranked: u16,
    pub total_ranked_wins: u16,
    pub total_ranked_losses: u16,
    pub ranked_win_rate: f32,
}


#[derive(Clone, Serialize, Deserialize, Default, Archive)]
pub struct LiveGameParticipantChampionStats {
    pub total_champion_played: u16,
    pub total_champion_wins: u16,
    pub total_champion_losses: u16,
    pub champion_win_rate: f32,
    pub avg_kills: f32,
    pub avg_deaths: f32,
    pub avg_assists: f32,
}
