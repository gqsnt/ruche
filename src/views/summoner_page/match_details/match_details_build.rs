use leptos::prelude::*;
use leptos::{component, view, IntoView};
use std::collections::HashMap;

use crate::consts::champion::Champion;
use crate::consts::item::Item;
use crate::consts::perk::Perk;
use crate::consts::HasStaticAsset;
use crate::views::summoner_page::match_details::{
    ItemEventType, LolMatchParticipantDetails, Skill,
};

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
                                    <div class="flex flex-col items-center relative mb-6">
                                        <div class="flex items-center border-gray-950 border-4 rounded text-xs">
                                            {item_event
                                                .iter()
                                                .map(|item_event| {
                                                    let is_sold_item = item_event.event_type
                                                        == ItemEventType::Sold;
                                                    let item_enum = Item::try_from(item_event.item_id).unwrap();
                                                    view! {
                                                        <div
                                                            class=("rounded", is_sold_item)
                                                            class="relative border-gray-950 border-4"
                                                        >
                                                            <img
                                                                alt=item_enum.to_string()
                                                                height="30"
                                                                width="30"
                                                                src=item_enum.get_static_asset_url()
                                                                class=("opacity-75", is_sold_item)
                                                                class="h-[30px] w-[30px]"
                                                            />
                                                            <Show when=move || is_sold_item>
                                                                <div class="absolute -bottom-[0.1rem] right-2 text-red-500 font-extrabold text-2xl">
                                                                    X
                                                                </div>
                                                            </Show>
                                                        </div>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}

                                        </div>
                                        <div class="text-center mt-1 absolute -bottom-3">
                                            {*minute}min
                                        </div>
                                    </div>
                                    <Show when=move || idx != total - 1>
                                        <div
                                            class="flex mb-10 items-center"
                                            v-if="idx > first_frame_with_events"
                                        >
                                            <div class="flex items-center h-10  border-8 border-gray-900 bg-gray-900">
                                                >
                                            </div>
                                        </div>
                                    </Show>
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
                                        class:text-blue-400=move || skill == Skill::Q
                                        class:text-green-400=move || skill == Skill::W
                                        class:text-orange-400=move || skill == Skill::E
                                        class:bg-indigo-500=move || skill == Skill::R
                                        class:bg-zinc-700=move || skill != Skill::R
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
                                <img
                                    width="28"
                                    height="28"
                                    alt=perk_primary_style.to_string()
                                    src=perk_primary_style.get_static_asset_url()
                                    class="w-[28px] h-[28px] rounded"
                                />
                                <img
                                    width="28"
                                    height="28"
                                    alt=perk_primary_selection.to_string()
                                    src=perk_primary_selection.get_static_asset_url()
                                    class="w-[28px] h-[28px] rounded"
                                />
                                <img
                                    width="28"
                                    height="28"
                                    alt=perk_primary_selection1.to_string()
                                    src=perk_primary_selection1.get_static_asset_url()
                                    class="w-[28px] h-[28px] rounded"
                                />
                                <img
                                    width="28"
                                    height="28"
                                    alt=perk_primary_selection2.to_string()
                                    src=perk_primary_selection2.get_static_asset_url()
                                    class="w-[28px] h-[28px] rounded"
                                />
                                <img
                                    width="28"
                                    height="28"
                                    alt=perk_primary_selection3.to_string()
                                    src=perk_primary_selection3.get_static_asset_url()
                                    class="w-[28px] h-[28px] rounded"
                                />
                            </div>
                            <div class="border-l-2 flex flex-col space-y-1 border-gray-900 h-fit pl-1.5">

                                <img
                                    width="28"
                                    height="28"
                                    alt=perk_sub_style.to_string()
                                    src=perk_sub_style.get_static_asset_url()
                                    class="w-[28px] h-[28px] rounded"
                                />
                                <img
                                    width="28"
                                    height="28"
                                    alt=perk_sub_selection1.to_string()
                                    src=perk_sub_selection1.get_static_asset_url()
                                    class="w-[28px] h-[28px] rounded"
                                />
                                <img
                                    width="28"
                                    height="28"
                                    alt=perk_sub_selection2.to_string()
                                    src=perk_sub_selection2.get_static_asset_url()
                                    class="w-[28px] h-[28px] rounded"
                                />
                            </div>

                            <div class="border-l-2 flex flex-col space-y-1 border-gray-900 h-fit pl-1.5">
                                <img
                                    width="28"
                                    height="28"
                                    alt=perk_offense.to_string()
                                    src=perk_offense.get_static_asset_url()
                                    class="w-[28px] h-[28px] rounded"
                                />
                                <img
                                    width="28"
                                    height="28"
                                    alt=perk_flex.to_string()
                                    src=perk_flex.get_static_asset_url()
                                    class="w-[28px] h-[28px] rounded"
                                />

                                <img
                                    width="28"
                                    height="28"
                                    alt=perk_defense.to_string()
                                    src=perk_defense.get_static_asset_url()
                                    class="w-[28px] h-[28px] rounded"
                                />
                            </div>
                        </div>
                    </div>
                }
            }}

        </div>
    }
}
