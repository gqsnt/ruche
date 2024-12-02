use leptos::prelude::*;
use leptos::{component, view, IntoView};
use std::collections::HashMap;

use crate::views::summoner_page::match_details::{
    ItemEventType, LolMatchParticipantDetails, Skill,
};
use crate::views::{ImgItem, ImgPerk};
use common::consts::champion::Champion;
use common::consts::item::Item;
use common::consts::perk::Perk;

#[component]
pub fn MatchDetailsBuild(
    summoner_id: i32,
    match_details: ReadSignal<Vec<LolMatchParticipantDetails>>,
) -> impl IntoView {
    let summoner_name_with_champion = |participant: &LolMatchParticipantDetails| {
        format!(
            "{}({})",
            participant.game_name.as_str(),
            Champion::from(participant.champion_id).to_str()
        )
    };
    let participant_ids = match_details
        .read()
        .iter()
        .map(|x| (x.summoner_id, summoner_name_with_champion(x)))
        .collect::<HashMap<i32, String>>();
    let find_participant = move |summoner_id: i32| {
        match_details
            .read()
            .iter()
            .find(|x| x.summoner_id == summoner_id)
            .cloned()
            .expect("Participant not found")
    };
    let (selected_participant, set_selected_participant) = signal(find_participant(summoner_id));
    view! {
        <div class="text-left">
            <select
                class="my-select"
                aria-label="Select a participant"
                prop:value=summoner_id
                on:change=move |e| set_selected_participant(
                    find_participant(
                        event_target_value(&e).parse::<i32>().expect("Invalid summoner id"),
                    ),
                )
            >
                {participant_ids
                    .into_iter()
                    .map(|(id, name)| {
                        view! {
                            <option
                                value=id
                                selected=move || id == selected_participant().summoner_id
                            >
                                {name.clone()}
                            </option>
                        }
                    })
                    .collect::<Vec<_>>()}
            </select>
            <div class="my-card w-fit my-2">
                <div>Items Build</div>
                <div class="flex mt-2 flex-wrap text-xs">
                    {move || {
                        let total = selected_participant().items_event_timeline.len();
                        selected_participant()
                            .items_event_timeline
                            .iter()
                            .enumerate()
                            .map(|(idx, (minute, item_event))| {
                                view! {
                                    <div class="flex flex-col items-center relative mb-8">
                                        <div class="flex items-center border-gray-950 border-4 rounded text-xs">
                                            {item_event
                                                .iter()
                                                .map(|item_event| {
                                                    let is_sold_item = item_event.event_type
                                                        == ItemEventType::Sold;
                                                    let item_enum = Item::try_from(item_event.item_id).unwrap();
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
                                            {*minute}min
                                        </div>
                                    </div>
                                    {(idx != total - 1)
                                        .then(|| {
                                            view! {
                                                <div
                                                    class="flex mb-8 items-center "
                                                    v-if="idx > first_frame_with_events"
                                                >
                                                    <div class="flex items-center h-7 border-4 border-gray-900 bg-gray-900">
                                                        >
                                                    </div>
                                                </div>
                                            }
                                        })}
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
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
                let perk_primary_style = Perk::from(selected_participant().perk_primary_style_id);
                let perk_primary_selection = Perk::from(
                    selected_participant().perk_primary_selection_id,
                );
                let perk_primary_selection1 = Perk::from(
                    selected_participant().perk_primary_selection1_id,
                );
                let perk_primary_selection2 = Perk::from(
                    selected_participant().perk_primary_selection2_id,
                );
                let perk_primary_selection3 = Perk::from(
                    selected_participant().perk_primary_selection3_id,
                );
                let perk_sub_style = Perk::from(selected_participant().perk_sub_style_id);
                let perk_sub_selection1 = Perk::from(selected_participant().perk_sub_selection1_id);
                let perk_sub_selection2 = Perk::from(selected_participant().perk_sub_selection2_id);
                let perk_offense = Perk::from(selected_participant().perk_offense_id);
                let perk_flex = Perk::from(selected_participant().perk_flex_id);
                let perk_defense = Perk::from(selected_participant().perk_defense_id);

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
