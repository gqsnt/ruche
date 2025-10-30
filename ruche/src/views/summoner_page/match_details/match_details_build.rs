use crate::views::summoner_page::match_details::{
    ItemEventType, LolMatchParticipantDetails, Skill,
};
use crate::views::{ImgItem, ImgPerk};
use common::consts::champion::Champion;
use common::consts::item::Item;
use common::consts::perk::Perk;
use itertools::Itertools;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use std::collections::HashMap;
use std::sync::Arc;

#[component]
pub fn MatchDetailsBuild(match_details: Arc<Vec<LolMatchParticipantDetails>>) -> impl IntoView {
    let summoner_name_with_champion = |p: &LolMatchParticipantDetails| {
        format!(
            "{} | {}",
            p.game_name.as_str(),
            Champion::try_from(p.champion_id)
                .unwrap_or_default()
                .label()
        )
    };
    let participant_ids =match_details
        .iter()
        .map(|x| (x.summoner_id, summoner_name_with_champion(x)))
        .collect::<HashMap<i32, String>>();

    let default_participant = match_details
        .iter()
        .find(|x| x.is_self_summoner)
        .map(|p| p.summoner_id)
        .expect("Participant not found");

    let find_participant = move |id: i32| {
        match_details
            .iter()
            .find(|x| x.summoner_id == id)
            .cloned()
            .expect("Participant not found")
    };

    let (selected_participant, set_selected_participant) =
        signal(find_participant(default_participant));
    let total = Memo::new(move |_| selected_participant().items_event_timeline.len());
    view! {
        <div class="text-left">
            <select
                class="my-select"
                aria-label="Select a participant"
                prop:value=move || selected_participant().summoner_id.to_string()
                on:change=move |e| {
                    let id = event_target_value(&e).parse::<i32>().expect("Invalid summoner id");
                    set_selected_participant(find_participant(id));
                }
            >
                <For each=move || participant_ids.clone() key=|(id, _)| *id let:entry>
                    {
                        let (id, name) = entry;
                        view! { <option value=id>{name}</option> }
                    }
                </For>
            </select>
            <div class="my-card w-fit my-2">
                <div>Items Build</div>
                <div class="flex mt-2 flex-wrap text-xs">
                    <For
                        each=move || {
                            selected_participant()
                                .items_event_timeline
                                .iter()
                                .cloned()
                                .enumerate()
                                .collect_vec()
                        }
                        key=move |(idx, _)| (*idx,selected_participant().summoner_id)
                        let:entry
                    >
                        {
                            let (idx, (minute, item_event)) = entry;
                            view! {
                                <div class="flex flex-col items-center relative mb-8">
                                    <div class="flex items-center border-gray-950 border-4 rounded text-xs">
                                        {item_event
                                            .iter()
                                            .map(|item_event| {
                                                let is_sold_item = item_event.event_type
                                                    == ItemEventType::Sold;
                                                let item_enum = Item::try_from(item_event.item_id)
                                                    .unwrap_or_default();
                                                view! {
                                                    <ImgItem
                                                        item=item_enum
                                                        class=format!(
                                                            "relative sprite-wrapper {}",
                                                            if is_sold_item { "rounded opacity-75" } else { "" },
                                                        )
                                                        parent_class="border-4 border-gray-950 w-[29.5px] h-[29.5px] sprite-inner"
                                                            .to_string()
                                                    >
                                                        {is_sold_item
                                                            .then(|| {
                                                                view! {
                                                                    <div class="z-10 absolute bottom-1 right-[0.725rem] text-red-500 font-extrabold text-xl">
                                                                        X
                                                                    </div>
                                                                }
                                                            })}

                                                    </ImgItem>
                                                }
                                            })
                                            .collect::<Vec<_>>()}

                                    </div>
                                    <div class="text-center mt-1 absolute -bottom-5">
                                        {minute}min
                                    </div>
                                </div>
                                {(idx != total.get() - 1)
                                    .then(|| {
                                        view! {
                                            <div class="flex mb-8 items-center ">
                                                <div class="flex items-center h-7 border-4 border-gray-900 bg-gray-900">
                                                    >
                                                </div>
                                            </div>
                                        }
                                    })}
                            }
                        }
                    </For>
                </div>
            </div>
            <div class="my-2 my-card w-fit">
                <div class="">Skill Order</div>
                <div class="flex mt-2 space-x-2 text-xs">
                    {move || {
                        selected_participant()
                            .skills_timeline
                            .clone()
                            .into_iter()
                            .map(|skill| {
                                view! {
                                    <div
                                        class:text-blue-400=skill == Skill::Q
                                        class:text-green-400=skill == Skill::W
                                        class:text-orange-400=skill == Skill::E
                                        class:bg-indigo-500=skill == Skill::R
                                        class:bg-zinc-700=skill != Skill::R
                                        class=" font-bold rounded w-4 h-4 text-center"
                                    >

                                        {skill.to_string()}
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>
            </div>
            {move || {
                let perk_primary_style = Perk::try_from(
                        selected_participant().perk_primary_style_id,
                    )
                    .unwrap_or_default();
                let perk_primary_selection = Perk::try_from(
                        selected_participant().perk_primary_selection_id,
                    )
                    .unwrap_or_default();
                let perk_primary_selection1 = Perk::try_from(
                        selected_participant().perk_primary_selection1_id,
                    )
                    .unwrap_or_default();
                let perk_primary_selection2 = Perk::try_from(
                        selected_participant().perk_primary_selection2_id,
                    )
                    .unwrap_or_default();
                let perk_primary_selection3 = Perk::try_from(
                        selected_participant().perk_primary_selection3_id,
                    )
                    .unwrap_or_default();
                let perk_sub_style = Perk::try_from(selected_participant().perk_sub_style_id)
                    .unwrap_or_default();
                let perk_sub_selection1 = Perk::try_from(
                        selected_participant().perk_sub_selection1_id,
                    )
                    .unwrap_or_default();
                let perk_sub_selection2 = Perk::try_from(
                        selected_participant().perk_sub_selection2_id,
                    )
                    .unwrap_or_default();
                let perk_offense = Perk::try_from(selected_participant().perk_offense_id)
                    .unwrap_or_default();
                let perk_flex = Perk::try_from(selected_participant().perk_flex_id)
                    .unwrap_or_default();
                let perk_defense = Perk::try_from(selected_participant().perk_defense_id)
                    .unwrap_or_default();

                view! {
                    <div class="my-2 my-card w-fit">
                        <div class="">Runes</div>
                        <div class="flex justify-center mt-2 space-x-2 text-xs">
                            <div class="flex flex-col space-y-1.5 ">
                                <ImgPerk
                                    perk=perk_primary_style
                                    class="w-[28px] h-[28px] rounded".to_string()
                                />
                                <ImgPerk
                                    perk=perk_primary_selection
                                    class="w-[28px] h-[28px] rounded".to_string()
                                />
                                <ImgPerk
                                    perk=perk_primary_selection1
                                    class="w-[28px] h-[28px] rounded".to_string()
                                />
                                <ImgPerk
                                    perk=perk_primary_selection2
                                    class="w-[28px] h-[28px] rounded".to_string()
                                />
                                <ImgPerk
                                    perk=perk_primary_selection3
                                    class="w-[28px] h-[28px] rounded".to_string()
                                />
                            </div>
                            <div class="border-l-2 flex flex-col space-y-1 border-gray-900 h-fit pl-1.5">

                                <ImgPerk
                                    perk=perk_sub_style
                                    class="w-[28px] h-[28px] rounded".to_string()
                                />
                                <ImgPerk
                                    perk=perk_sub_selection1
                                    class="w-[28px] h-[28px] rounded".to_string()
                                />
                                <ImgPerk
                                    perk=perk_sub_selection2
                                    class="w-[28px] h-[28px] rounded".to_string()
                                />

                            </div>

                            <div class="border-l-2 flex flex-col space-y-1 border-gray-900 h-fit pl-1.5">

                                <ImgPerk
                                    perk=perk_offense
                                    class="w-[28px] h-[28px] rounded".to_string()
                                />
                                <ImgPerk
                                    perk=perk_flex
                                    class="w-[28px] h-[28px] rounded".to_string()
                                />
                                <ImgPerk
                                    perk=perk_defense
                                    class="w-[28px] h-[28px] rounded".to_string()
                                />
                            </div>
                        </div>
                    </div>
                }
            }}

        </div>
    }
}
