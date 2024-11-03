use std::collections::HashMap;
use leptos::prelude::{event_target_value, signal, ClassAttribute, OnAttribute, PropAttribute, Read, ReadSignal, Resource, ServerFnError, Show, Suspend, Suspense};
use leptos::{component, view, IntoView};
use leptos::either::Either;
use leptos::prelude::ElementChild;
use leptos::prelude::CustomAttribute;

use crate::consts::Champion;
use crate::models::entities::lol_match_participant::LolMatchParticipantMatchesDetailPage;
use crate::models::entities::lol_match_timeline::ItemEvent;

#[component]
pub fn MatchDetailsBuild(summoner_id:i32, match_details : ReadSignal<Vec<LolMatchParticipantMatchesDetailPage>>) -> impl IntoView {
    let summoner_name_with_champion = |participant:&LolMatchParticipantMatchesDetailPage|{
        format!("{}({})", participant.summoner_name, Champion::try_from(participant.champion_id as i16).unwrap().to_string())
    };
    let participant_ids = match_details.read().iter().map(|x| (x.summoner_id, summoner_name_with_champion(x))).collect::<HashMap<i32, String>>();
    let find_participant = move |summoner_id:i32| match_details.read().iter().find(|x| x.summoner_id == summoner_id).cloned().unwrap();
    let (selected_participant, set_selected_participant) = signal(find_participant(summoner_id));
    view!{
        <div class="text-left mt-1 ">
            <select prop:value=summoner_id on:change=move |e| set_selected_participant(find_participant(event_target_value(&e).parse::<i32>().unwrap()))>
                {participant_ids.into_iter().map(|(id, name)| view!{
                    <option
                        value=id
                        selected=move || id == selected_participant().summoner_id
                    >
                        {name.clone()}
                    </option>
                }).collect::<Vec<_>>()}
            </select>
            <div>
                <div>Item Builds</div>
                <div class="flex mt-2 flex-wrap text-xs">
                    {move || {
                        let total = selected_participant().items_event_timeline.len();
                        selected_participant().items_event_timeline.iter().enumerate().map(|(idx, (minute, item_event))|{
                        view!{
                            <div class="flex flex-col items-center relative mb-6">
                                <div class="flex items-center border-gray-900 border-4 rounded text-xs">
                                        {
                                            item_event.iter().filter(|e|matches!(e,ItemEvent::Sold {..} | ItemEvent::Purchased {..})).map(|item_event|{
                                                let is_sold_item = matches!(item_event,ItemEvent::Sold {..});
                                                view!{
                                                    <div class=("rounded", is_sold_item) class="relative border-gray-900 border-4" >
                                                        <img height="30" width="30" src=format!("/assets/items/{}.webp", item_event.get_id()) class=("opacity-75", is_sold_item) class="h-[30px] w-[30px]" />
                                                        <Show when=move ||is_sold_item>
                                                            <div class="absolute -bottom-0.5 -right-0.5">X</div>
                                                        </Show>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()
                                        }

                                </div>
                                   <div class="text-center mt-1 absolute -bottom-3">{*minute} min</div>
                            </div>
                            <Show when=move || idx != total-1>
                                <div class="flex mb-10 items-center" v-if="idx > first_frame_with_events">
                                    <div class="flex items-center h-10  border-8 border-gray-900 bg-gray-900">
                                        >
                                    </div>
                                </div>
                            </Show>
                        }
                    }).collect::<Vec<_>>()}}
                </div>
            </div>
            <div class="my-2">
                <div>Skill Order</div>
                <div class="flex mt-2 space-x-2 text-xs">
                    {move ||{
                        selected_participant().skills_timeline.iter().map(|skill_id|{
                            view!{
                                <div
                                    class=("text-blue-400", *skill_id == 1)
                                    class=("text-green-400", *skill_id == 2)
                                    class=("text-orange-400", *skill_id == 3)
                                    class=("bg-indigo-500", *skill_id == 4)
                                    class=("bg-zinc-700", *skill_id != 4)
                                    class="font-bold rounded w-4 h-4 text-center"


                                >
                                    {
                                        match skill_id{
                                            1 => "Q",
                                            2 => "W",
                                            3 => "E",
                                            4 => "R",
                                            _ => ""
                                        }
                                    }
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>
            </div>
        </div>
    }
}