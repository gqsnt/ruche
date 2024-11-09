use crate::apis::get_live_game;
use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::models::entities::summoner::{LiveGameResultParticipant, Summoner};
use leptos::prelude::{expect_context, ReadSignal, Set};
use leptos::prelude::{signal, ClassAttribute, ElementChild, Get, OnAttribute, Resource, Suspend, Suspense};
use leptos::{component, view, IntoView};
use leptos::either::Either;
use crate::consts::{Champion, Perk, SummonerSpell};
use crate::models::entities::lol_match_participant::LolMatchParticipantMatchesDetailPage;

#[component]
pub fn SummonerLivePage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

    let (refresh_signal, set_refresh_signal) = signal(0);

    let live_game_resource = Resource::new(
        move || (refresh_signal.get(), summoner().puuid.clone(),summoner().platform.as_region_str().to_string()),
        |(refresh_version, puuid, platform_type)| async move {
            get_live_game(puuid,platform_type).await
        },
    );

    meta_store.title().set(format!("{}#{} | Live Game | Broken.gg", summoner().game_name, summoner().tag_line));
    meta_store.description().set(format!("Watch {}#{}'s live game now on Broken.gg. Get real-time updates and analytics with our ultra-fast, Rust-based League of Legends companion.", summoner().game_name, summoner().tag_line));
    meta_store.url().set(format!("{}?tab=live", summoner().to_route_path()));
    view! {
        <div class="w-[768px]">
            <div class="flex justify-start mb-2">
                <button
                    class="my-button"
                    on:click=move |e| { set_refresh_signal(refresh_signal() + 1) }
                >
                    Refresh
                </button>
            </div>
            <Suspense fallback=move || {
                view! { <p>"Not in Live Game"</p> }
            }>
                {move || Suspend::new(async move {
                    match live_game_resource.await {
                        Ok(result) => {
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
                                            <div>{result.queue_name}</div>
                                            <div>{result.game_map}</div>
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
                        Err(e) => {
                            Either::Left(

                                view! { <p>"Not in Live Game"</p> },
                            )
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}



#[component]
pub fn MatchLiveTable(team_id: i32, participants: Vec<LiveGameResultParticipant>) -> impl IntoView {
    let is_blue_team = team_id == 100;
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
                        class=("text-blue-800", is_blue_team)
                        class=("text-red-800", !is_blue_team)
                    >
                        {if is_blue_team { "Blue Team" } else { "Red Team" }}
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
                    .iter()
                    .map(|participant| {
                        view! {
                            <tr>
                                <td
                                    class=("border-red-500", !is_blue_team)
                                    class=("border-blue-500", is_blue_team)
                                    class=" border-l-2 pl-2.5 py-1"
                                >

                                    <div class="relative w-8">
                                        <img
                                            width="32"
                                            height="32"
                                            alt=Champion::try_from(participant.champion_id as i16)
                                                .unwrap()
                                                .to_string()
                                            src=Champion::get_static_url(participant.champion_id)
                                            class="w-8 h-8 rounded-full block"
                                        />
                                    </div>
                                </td>
                                <td class="py-1">
                                    <div class="relative">
                                        <img
                                            width="16"
                                            height="16"
                                            alt=SummonerSpell::try_from(
                                                    participant.summoner_spell1_id as u16,
                                                )
                                                .unwrap()
                                                .to_string()
                                            src=SummonerSpell::get_static_url(
                                                participant.summoner_spell1_id,
                                            )
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                    <div class="relative">
                                        <img
                                            width="16"
                                            height="16"
                                            alt=SummonerSpell::try_from(
                                                    participant.summoner_spell2_id as u16,
                                                )
                                                .unwrap()
                                                .to_string()
                                            src=SummonerSpell::get_static_url(
                                                participant.summoner_spell2_id,
                                            )
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                </td>
                                <td class="py-1">
                                    <div class="relative">
                                        <img
                                            alt=Perk::try_from(
                                                    participant.perk_primary_selection_id as u16,
                                                )
                                                .unwrap()
                                                .to_string()
                                            width="16"
                                            height="16"
                                            src=Perk::get_static_url(
                                                participant.perk_primary_selection_id,
                                            )
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                    <div class="relative">
                                        <img
                                            width="16"
                                            height="16"
                                            alt=Perk::try_from(participant.perk_sub_style_id as u16)
                                                .unwrap()
                                                .to_string()
                                            src=Perk::get_static_url(participant.perk_sub_style_id)
                                            class="w-4 h-4 rounded"
                                        />
                                    </div>
                                </td>
                                <td class="pl-[5px] py-1 text-ellipsis overflow-hidden text-left">
                                    <div>
                                        <a
                                            target="_blank"
                                            href=format!(
                                                "/{}/summoners/{}-{}",
                                                participant.platform,
                                                participant.game_name,
                                                participant.tag_line,
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
                                    {match &participant.ranked_stats {
                                        Some(ranked_stats) => {
                                            Either::Left(
                                                view! {
                                                    <div>
                                                        {(ranked_stats.ranked_win_rate * 100.0).round()}%
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
                                                        {(champion_stats.champion_win_rate * 100.0).round()}%
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
                                                        {(((champion_stats.avg_kills + champion_stats.avg_assists)
                                                            / champion_stats.avg_deaths.max(1.0)) * 100.0)
                                                            .round() / 100.0}:1
                                                    </div>
                                                    <div>
                                                        {champion_stats.avg_kills}/ {champion_stats.avg_deaths}/
                                                        {champion_stats.avg_assists}
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