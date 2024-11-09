use crate::consts::{Champion, Item, Perk, SummonerSpell};
use crate::models::entities::lol_match_participant::LolMatchParticipantMatchesDetailPage;
use leptos::prelude::{ClassAttribute, ReadSignal};
use leptos::prelude::{ElementChild, Show};
use leptos::{component, view, IntoView};

#[component]
pub fn MatchDetailsOverview(summoner_id: i32, match_details: ReadSignal<Vec<LolMatchParticipantMatchesDetailPage>>) -> impl IntoView {
    let details = match_details();
    let (summoner_team, summoner_team_won) = {
        let detail = details.iter().find(|participant| participant.summoner_id == summoner_id).unwrap();
        (detail.team_id, detail.won)
    };
    let other_team = if summoner_team == 100 {
        200
    } else {
        100
    };
    let first_team = details.iter().filter(|participant| participant.team_id == summoner_team).cloned().collect::<Vec<_>>();
    let second_team = details.iter().filter(|participant| participant.team_id != summoner_team).cloned().collect::<Vec<_>>();
    view! {
        <div>
            <MatchDetailsOverviewTable
                won=summoner_team_won
                team_id=summoner_team
                participants=first_team
                summoner_id
            />
            <MatchDetailsOverviewTable
                won=!summoner_team_won
                team_id=other_team
                participants=second_team
                summoner_id
            />
        </div>
    }
}


#[component]
pub fn MatchDetailsOverviewTable(won: bool, team_id: i32, summoner_id: i32, participants: Vec<LolMatchParticipantMatchesDetailPage>) -> impl IntoView {
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
                {participants
                    .iter()
                    .map(|participant| {
                        let item0_id = participant.item0_id;
                        let item1_id = participant.item1_id;
                        let item2_id = participant.item2_id;
                        let item3_id = participant.item3_id;
                        let item4_id = participant.item4_id;
                        let item5_id = participant.item5_id;
                        let item6_id = participant.item6_id;

                        view! {
                            <tr
                                class=("bg-red-900", !won && participant.summoner_id != summoner_id)
                                class=("bg-blue-900", won && participant.summoner_id != summoner_id)
                                class=("bg-red-800", !won && participant.summoner_id == summoner_id)
                                class=("bg-blue-800", won && participant.summoner_id == summoner_id)
                            >
                                <td class="pl-2.5 py-1">
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
                                        <span class="absolute left-[-3px] bottom-[-3px] w-[15px] h-[15px] bg-gray-600 rounded-full text-[10px] text-center">
                                            {participant.champ_level}
                                        </span>
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
                                                participant.summoner_platform,
                                                participant.summoner_name,
                                                participant.summoner_tag_line,
                                            )
                                        >
                                            {participant.summoner_name.clone()}
                                        </a>
                                    </div>
                                    <span class="text-[11px]">
                                        Lvl. {participant.summoner_level}
                                    </span>
                                </td>
                                <td class="py-1 text-center">
                                    <div class="flex  justify-center">
                                        {participant.kills}/{participant.deaths}/
                                        {participant.assists}
                                        <div class="ml-1 relative">
                                            {format!(
                                                "({}%)",
                                                (participant.kill_participation * 100.0).round(),
                                            )}
                                        </div>
                                    </div>
                                </td>
                                <td class="py-1">
                                    <div class="flex justify-center">
                                        <div>{participant.damage_dealt_to_champions}</div>
                                        <div class="ml-2">{participant.damage_taken}</div>
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
                                        <Show when=move || item0_id != 0 fallback=|| view! {}>
                                            <div class="relative rounded">
                                                <img
                                                    alt=format!("Item {}", item0_id)
                                                    width="22"
                                                    height="22"
                                                    src=Item::get_static_url(item0_id)
                                                    class="w-[22px] w-[22px]"
                                                />
                                            </div>
                                        </Show>
                                        <Show when=move || item1_id != 0 fallback=|| view! {}>
                                            <div class="relative rounded">
                                                <img
                                                    alt=format!("Item {}", item1_id)
                                                    width="22"
                                                    height="22"
                                                    src=Item::get_static_url(item1_id)
                                                    class="w-[22px] w-[22px]"
                                                />
                                            </div>
                                        </Show>
                                        <Show when=move || item2_id != 0 fallback=|| view! {}>
                                            <div class="relative rounded">
                                                <img
                                                    alt=format!("Item {}", item2_id)
                                                    width="22"
                                                    height="22"
                                                    src=Item::get_static_url(item2_id)
                                                    class="w-[22px] w-[22px]"
                                                />
                                            </div>
                                        </Show>
                                        <Show when=move || item3_id != 0 fallback=|| view! {}>
                                            <div class="relative rounded">
                                                <img
                                                    alt=format!("Item {}", item3_id)
                                                    width="22"
                                                    height="22"
                                                    src=Item::get_static_url(item3_id)
                                                    class="w-[22px] w-[22px]"
                                                />
                                            </div>
                                        </Show>
                                        <Show when=move || item4_id != 0 fallback=|| view! {}>
                                            <div class="relative rounded">
                                                <img
                                                    alt=format!("Item {}", item4_id)
                                                    width="22"
                                                    height="22"
                                                    src=Item::get_static_url(item4_id)
                                                    class="w-[22px] w-[22px]"
                                                />
                                            </div>
                                        </Show>
                                        <Show when=move || item5_id != 0 fallback=|| view! {}>
                                            <div class="relative rounded">
                                                <img
                                                    alt=format!("Item {}", item5_id)
                                                    width="22"
                                                    height="22"
                                                    src=Item::get_static_url(item5_id)
                                                    class="w-[22px] w-[22px]"
                                                />
                                            </div>
                                        </Show>
                                        <Show when=move || item6_id != 0 fallback=|| view! {}>
                                            <div class="relative rounded">
                                                <img
                                                    alt=format!("Item {}", item6_id)
                                                    width="22"
                                                    height="22"
                                                    src=Item::get_static_url(item6_id)
                                                    class="w-[22px] w-[22px]"
                                                />
                                            </div>
                                        </Show>
                                    </div>
                                </td>
                            </tr>
                        }
                    })
                    .collect::<Vec<_>>()}
            </tbody>
        </table>
    }
}