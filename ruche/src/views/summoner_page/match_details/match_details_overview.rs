use crate::utils::{format_with_spaces, summoner_encounter_url, summoner_url};
use crate::views::summoner_page::match_details::LolMatchParticipantDetails;
use crate::views::summoner_page::Summoner;
use crate::views::{ImgChampion, ImgItem, ImgPerk, ImgSummonerSpell};
use common::consts::champion::Champion;
use common::consts::item::Item;
use common::consts::perk::Perk;
use common::consts::summoner_spell::SummonerSpell;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::components::A;

#[component]
pub fn MatchDetailsOverview(
    summoner_id: i32,
    match_details: ReadSignal<Vec<LolMatchParticipantDetails>>,
) -> impl IntoView {
    let details = match_details();
    let (summoner_team, summoner_team_won) = {
        let detail = details
            .iter()
            .find(|participant| participant.summoner_id == summoner_id)
            .expect("Summoner id not found");
        (detail.team_id, detail.won)
    };
    let other_team = if summoner_team == 100 { 200 } else { 100 };
    let first_team = details
        .iter()
        .filter(|participant| participant.team_id == summoner_team)
        .cloned()
        .collect::<Vec<_>>();
    let second_team = details
        .iter()
        .filter(|participant| participant.team_id != summoner_team)
        .cloned()
        .collect::<Vec<_>>();
    view! {
        <div>
            <MatchDetailsOverviewTable
                won=summoner_team_won
                team_id=summoner_team
                participants=first_team
            />
            <MatchDetailsOverviewTable
                won=!summoner_team_won
                team_id=other_team
                participants=second_team
            />

        </div>
    }
}

#[component]
pub fn MatchDetailsOverviewTable(
    won: bool,
    team_id: u16,
    participants: Vec<LolMatchParticipantDetails>,
) -> impl IntoView {
    let summoner = expect_context::<Summoner>();
    view! {
        <table class="table-fixed text-xs w-full border-collapse">
            <colgroup>
                <col width="44" />
                <col width="18" />
                <col width="18" />
                <col />
                <col width="98" />
                <col width="120" />
                <col width="48" />
                <col width="56" />
                <col width="175" />
            </colgroup>
            <thead>
                <tr>
                    <th colspan="4">
                        <span>{if won { "Victory" } else { "Defeat" }}</span>
                        {if team_id == 100 { "(Blue Team)" } else { "(Red Team)" }}
                    </th>
                    <th>KDA</th>
                    <th>Damage</th>
                    <th>Wards</th>
                    <th>CS</th>
                    <th>Item</th>
                </tr>
            </thead>
            <tbody>
                <For
                each=move ||participants.clone()
                 key=|participant| participant.id
                let:participant
        >
                {
                        let champion = Champion::try_from(participant.champion_id).unwrap_or_default();
                        let summoner_spell1 = SummonerSpell::try_from(participant.summoner_spell1_id).unwrap_or_default();
                        let summoner_spell2 = SummonerSpell::try_from(participant.summoner_spell2_id).unwrap_or_default();
                        let primary_perk_selection = Perk::try_from(
                            participant.perk_primary_selection_id,
                        ).unwrap_or_default();
                        let sub_perk_style = Perk::try_from(participant.perk_sub_style_id).unwrap_or_default();
                        let items = [
                            participant.item0_id,
                            participant.item1_id,
                            participant.item2_id,
                            participant.item3_id,
                            participant.item4_id,
                            participant.item5_id,
                            participant.item6_id,
                        ]
                            .iter()
                            .filter_map(|i| Item::try_from(*i).ok())
                            .collect::<Vec<_>>();

                        view! {
                            <tr
                                class=("bg-red-900", !won && participant.summoner_id != summoner.id)
                                class=("bg-blue-900", won && participant.summoner_id != summoner.id)
                                class=("bg-red-800", !won && participant.summoner_id == summoner.id)
                                class=("bg-blue-800", won && participant.summoner_id == summoner.id)
                            >
                                <td class="pl-2.5 py-1">
                                    <ImgChampion
                                        champion
                                        parent_class="w-8 h-8 sprite-wrapper relative".to_string()
                                        class="rounded-full self-scale-66 block sprite-inner".to_string()
                                    >
                                        <span class="absolute left-[-3px] bottom-[-3px] w-[15px] h-[15px] bg-gray-600 rounded-full text-[10px] text-center">
                                            {participant.champ_level}
                                        </span>
                                    </ImgChampion>

                                </td>
                                <td class="py-1">
                                    <ImgSummonerSpell
                                        summoner_spell=summoner_spell1
                                        class="self-scale-72 rounded sprite-wrapper".to_string()
                                        parent_class="w-4 h-4 sprite-inner".to_string()
                                    />
                                    <ImgSummonerSpell
                                        summoner_spell=summoner_spell2
                                        class="self-scale-72 rounded sprite-wrapper".to_string()
                                        parent_class="w-4 h-4 sprite-inner".to_string()
                                    />

                                </td>
                                <td class="py-1">
                                    <ImgPerk
                                        perk=primary_perk_selection
                                        class="self-scale-57 rounded".to_string()
                                        parent_class="w-4 h-4".to_string()
                                    />
                                    <ImgPerk
                                        perk=sub_perk_style
                                        class="self-scale-57 rounded".to_string()
                                        parent_class="w-4 h-4".to_string()
                                    />

                                </td>
                                <td class="pl-[5px] py-1 text-ellipsis overflow-hidden text-left">
                                    <div class="flex items-center gap-1">
                                        {(participant.encounter_count > 1)
                                            .then(|| {
                                                view! {
                                                    <A
                                                        href=summoner_encounter_url(
                                                            summoner.platform.code(),
                                                            summoner.game_name.as_str(),
                                                            summoner.tag_line.as_str(),
                                                            participant.platform.code(),
                                                            participant.game_name.as_str(),
                                                            participant.tag_line.as_str(),
                                                        )
                                                        attr:class="text-xs bg-green-800 rounded px-0.5 text-center"
                                                    >
                                                        {participant.encounter_count}
                                                    </A>
                                                }
                                            })}
                                        {summoner
                                            .pro_slug
                                            .map(|pps| {
                                                view! {
                                                    <A
                                                        target="_blank"
                                                        href=format!("https://lolpros.gg/player/{}", pps.as_ref())
                                                        attr:class="text-xs bg-purple-800 rounded px-0.5 text-center"
                                                    >
                                                        pro
                                                    </A>
                                                }
                                            })}
                                        <A
                                            target="_blank"
                                            href=summoner_url(
                                                participant.platform.code(),
                                                participant.game_name.as_str(),
                                                participant.tag_line.as_str(),
                                            )
                                        >
                                            {participant.game_name.clone()}
                                        </A>
                                    </div>
                                    <span class="text-[11px]">
                                        Lvl. {participant.summoner_level}
                                    </span>
                                </td>
                                <td class="py-1 text-center">
                                    <div class="flex  justify-center">
                                        {format!(
                                            "{}/{}/{}",
                                            participant.kills,
                                            participant.deaths,
                                            participant.assists,
                                        )}
                                        <div class="ml-1 relative">
                                            {participant.kill_participation}%
                                        </div>
                                    </div>
                                </td>
                                <td class="py-1">
                                    <div class="flex justify-center space-x-1">
                                        <div>
                                            {format_with_spaces(participant.damage_dealt_to_champions)}
                                        </div>
                                        <span>-</span>

                                        <div>{format_with_spaces(participant.damage_taken)}</div>
                                    </div>
                                </td>
                                <td class="py-1">
                                    <div class="flex justify-center">
                                        <div>{participant.wards_placed}</div>
                                    </div>
                                </td>
                                <td class="py-1">
                                    <div class="flex justify-center">
                                        <div>{participant.cs}</div>
                                    </div>
                                </td>
                                <td class="py-1">
                                    <div class="flex gap-0.5">
                                        {items
                                            .iter()
                                            .map(|item| {
                                                view! { <ImgItem item=*item class="rounded".to_string() /> }
                                            })
                                            .collect::<Vec<_>>()}
                                    </div>
                                </td>
                            </tr>
                        }
                    }

        </For>
            </tbody>
        </table>
    }
}
