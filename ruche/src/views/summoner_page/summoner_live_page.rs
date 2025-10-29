use bitcode::{Decode, Encode};
use crate::app::{to_summoner_identifier_memo, SummonerRouteParams};
use crate::backend::server_fns::get_live_game::get_live_game;
use crate::utils::{
    calculate_and_format_kda, calculate_loss_and_win_rate, format_float_to_2digits,
    summoner_encounter_url, summoner_url, ProPlayerSlug, RiotMatchId,
};
use crate::views::summoner_page::{SSEInLiveGame, SSEMatchUpdateVersion};
use crate::views::{ImgChampion, ImgPerk, ImgSummonerSpell, PendingLoading, ProPlayerSlugView};
use common::consts::champion::Champion;
use common::consts::map::Map;
use common::consts::perk::Perk;
use common::consts::platform_route::PlatformRoute;
use common::consts::queue::Queue;
use common::consts::summoner_spell::SummonerSpell;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos::prelude::codee::binary::BitcodeCodec;
use leptos_router::components::A;
use leptos_router::{lazy_route, LazyRoute};
use leptos_router::hooks::use_params;

pub struct SummonerLiveRoute{
    live_game_resource: Resource<Result<Option<LiveGame>, ServerFnError>, BitcodeCodec>,
    refresh_signal:RwSignal<i32>,
    pending:RwSignal<bool>,
}

#[lazy_route]
impl LazyRoute for SummonerLiveRoute {
    fn data() -> Self {
        let summoner_route_params = use_params::<SummonerRouteParams>();
        let summoner_identifier_memo = to_summoner_identifier_memo(
            summoner_route_params
        );
        let sse_match_update_version = expect_context::<RwSignal<Option<SSEMatchUpdateVersion>>>();
        let sse_in_live_game = expect_context::<RwSignal<SSEInLiveGame>>();
        let pending = RwSignal::new(false);
        let refresh_signal = RwSignal::new(0);
        let live_game_resource = Resource::new_bitcode(
            move || {
                (
                    sse_in_live_game.get(),
                    sse_match_update_version.get().unwrap_or_default(),
                    refresh_signal.get(),
                    summoner_identifier_memo.get(),
                    pending.write_only(),
                )
            },
            |(_, _, refresh_version, summoner_identifier, set_pending_value)| async move {
                let r = get_live_game(
                    summoner_identifier,
                    refresh_version > 0,
                )
                    .await;
                set_pending_value.set(false);
                r
            },
        );
    Self{
        live_game_resource,
        refresh_signal,
        pending,
    }
}

    fn view(this: Self) -> AnyView {
        let SummonerLiveRoute{live_game_resource, refresh_signal, pending} = this;
        
        //        let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
// batch(|| {
//         meta_store.title().set(format!(
//             "{}#{} | Live Game | Ruche",
//             summoner.read().game_name.as_str(),
//             summoner.read().tag_line.as_str()
//         ));
//         meta_store.description().set(format!("Watch {}#{}'s live game now on Ruche. Get real-time updates and analytics with our ultra-fast, Rust-based League of Legends companion.", summoner.read().game_name.as_str(), summoner.read().tag_line.as_str()));
//         meta_store
//             .url()
//             .set(format!("{}/live", summoner.read().to_route_path()));
//         });
        view! {
            <div class="w-[768px] my-2">
                <div class="flex justify-start mb-2">
                    <button
                        class="my-button flex items-center"
                        on:click=move |_| {
                            pending.set(true);
                            refresh_signal.set(refresh_signal.get() + 1);
                        }
                    >
                        <PendingLoading pending>Refresh</PendingLoading>
                    </button>
                </div>
                <Transition
                    fallback=move || {
                        view! { <div class="text-center">Not In Live Game</div> }
                    }
                    set_pending= pending.write_only()
                >
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
                                                <div>{result.queue.label()}</div>
                                                <div>{result.game_map.label()}</div>
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
                            _ => {
                                Either::Left(
                                    view! { <div class="text-center">Not In Live Game</div> },
                                )
                            }
                        }
                    })}
                </Transition>
            </div>
        }.into_any()

    }

}




#[component]
pub fn MatchLiveTable(team_id: i32, participants: Vec<LiveGameParticipant>) -> impl IntoView {
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
                        let champion = Champion::try_from(participant.champion_id)
                            .unwrap_or_default();
                        let summoner_spell1 = SummonerSpell::try_from(
                                participant.summoner_spell1_id,
                            )
                            .unwrap_or_default();
                        let summoner_spell2 = SummonerSpell::try_from(
                                participant.summoner_spell2_id,
                            )
                            .unwrap_or_default();
                        let perk_primary_selection = Perk::try_from(
                                participant.perk_primary_selection_id,
                            )
                            .unwrap_or_default();
                        let perk_sub_style = Perk::try_from(participant.perk_sub_style_id)
                            .unwrap_or_default();

                        view! {
                            <tr>
                                <td
                                    class="border-l-2 pl-2.5 py-1 "
                                    class=("border-red-500", !is_blue_team())
                                    class=("border-blue-500", is_blue_team())
                                >
                                    <ImgChampion
                                        champion
                                        class="self-scale-66 rounded-full block sprite-inner"
                                            .to_string()
                                        parent_class="w-8 h-8 sprite-wrapper".to_string()
                                    />
                                </td>
                                <td class="py-1">
                                    <ImgSummonerSpell
                                        summoner_spell=summoner_spell1
                                        class="self-scale-57 rounded sprite-inner".to_string()
                                        parent_class="w-4 h-4 sprite-wrapper".to_string()
                                    />
                                    <ImgSummonerSpell
                                        summoner_spell=summoner_spell2
                                        class="self-scale-57 rounded sprite-inner".to_string()
                                        parent_class="w-4 h-4 sprite-wrapper".to_string()
                                    />
                                </td>
                                <td class="py-1">
                                    <ImgPerk
                                        perk=perk_primary_selection
                                        parent_class="w-4 h-4 sprite-wrapper".to_string()
                                        class="self-scale-57 rounded sprite-inner".to_string()
                                    />
                                    <ImgPerk
                                        perk=perk_sub_style
                                        parent_class="w-4 h-4 sprite-wrapper".to_string()
                                        class="self-scale-57 rounded sprite-inner".to_string()
                                    />
                                </td>
                                <td class="pl-[5px] py-1 text-ellipsis overflow-hidden text-left">
                                    <div class="flex items-center gap-1">
                                        {(participant.encounter_count > 0)
                                            .then(|| {
                                                view! {
                                                    <A
                                                        href=summoner_encounter_url(
                                                            participant.platform.code(),
                                                            participant.game_name.as_str(),
                                                            participant.tag_line.as_str(),
                                                            false
                                                        )
                                                        attr:class="text-xs bg-green-800 rounded px-0.5 text-center"
                                                    >
                                                        {participant.encounter_count}
                                                    </A>
                                                }
                                            })}
                                        <ProPlayerSlugView
                                            pro_player_slug=participant.pro_player_slug
                                            small=true
                                        />
                                        <A href=summoner_url(
                                            participant.platform.code(),
                                            participant.game_name.as_str(),
                                            participant.tag_line.as_str(),
                                        )>
                                            {format!(
                                                "{}#{}",
                                                participant.game_name.as_str(),
                                                participant.tag_line.as_str(),
                                            )}
                                        </A>
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
                                                        {format!(
                                                            "{}/{}/{}",
                                                            format_float_to_2digits(champion_stats.avg_kills),
                                                            format_float_to_2digits(champion_stats.avg_deaths),
                                                            format_float_to_2digits(champion_stats.avg_assists),
                                                        )}
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
#[derive(Clone,  Encode,Decode)]
pub struct LiveGame {
    pub game_length: u16,
    pub game_map: Map,
    pub queue: Queue,
    pub game_id: RiotMatchId,
    pub participants: Vec<LiveGameParticipant>,
}

#[derive(Clone, Encode,Decode)]
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
    pub game_name: String,
    pub tag_line: String,
    pub pro_player_slug: Option<ProPlayerSlug>,
    pub ranked_stats: Option<LiveGameParticipantRankedStats>,
    pub champion_stats: Option<LiveGameParticipantChampionStats>,
}

#[derive(Clone, Default, Encode,Decode)]
pub struct LiveGameParticipantRankedStats {
    pub total_ranked: u16,
    pub total_ranked_wins: u16,
}

#[derive(Clone, Default,  Encode,Decode)]
pub struct LiveGameParticipantChampionStats {
    pub total_champion_played: u16,
    pub total_champion_wins: u16,
    pub avg_kills: f32,
    pub avg_deaths: f32,
    pub avg_assists: f32,
}
