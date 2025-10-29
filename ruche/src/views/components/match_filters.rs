use crate::views::{ parse_date, BackEndMatchFiltersSearch, BackEndMatchFiltersSearchStoreFields};
use common::consts::champion::CHAMPION_OPTIONS;
use common::consts::queue::{Queue};

use leptos::prelude::*;

use leptos::{component, view, IntoView};
use reactive_stores::Store;

#[component]
pub fn MatchFilters(hidden:Signal<bool>,children: Children) -> impl IntoView {
    let filters = expect_context::<Store<BackEndMatchFiltersSearch>>();

    Effect::new(move |_| {
        let _ = filters.champion_id().get();
        let _=  filters.queue_id().get();
        let _ = filters.start_date().get();
        let _ = filters.end_date().get();
        filters.page().set(None);
    });
    let to_opt_date = |v: String| if v.is_empty() { None } else { parse_date(Some(v)) };
    let to_opt_u16 = |v: String| if v.is_empty() { None } else { Some(v.parse::<u16>().unwrap_or_default()) };


    view! {
        <div class="flex justify-center my-2" class:hidden=move || hidden()>
            <div class="my-card w-[768px]">
                <div class="flex text-left space-x-2 justify-center">
                    <div class="flex flex-col">
                        <label for="champion_id">Champion</label>
                        <select
                            name="champion_id"
                            class="my-select"
                            id="champion_id"
                            prop:value=filters.champion_id().get()
                            on:change=move |e| {
                                filters.champion_id().set(to_opt_u16(event_target_value(&e)))
                            }
                        >
                            <option value="">All</option>
                            <For each=|| CHAMPION_OPTIONS.iter().cloned() key=|(id, _)| *id let:opt>
                                {
                                    let (id, label) = opt;
                                    view! { <option value=id>{label}</option> }
                                }
                            </For>
                        </select>
                    </div>

                    <div class="flex flex-col">
                        <label for="queue_id">Queue</label>
                        <select
                            class="my-select"
                            name="queue_id"
                            id="queue_id"
                            prop:value=filters.queue_id().get()
                            on:change=move |e| {
                                filters.queue_id().set(to_opt_u16(event_target_value(&e)))
                            }
                        >
                            <option value="">All</option>
                            <For each=|| Queue::options_basic() key=|(id, _)| *id let:opt>
                                {
                                    let (id, label) = opt;
                                    view! { <option value=id>{label}</option> }
                                }
                            </For>
                        </select>
                    </div>

                    <div class="flex flex-col">
                        <label for="start_date">Start Date</label>
                        <input
                            class="my-input"
                            type="date"
                            name="start_date"
                            id="start_date"
                            prop:value=move || {
                                filters.start_date().get().map(|start_date| start_date.to_string())
                            }
                            on:input=move |e| {
                                filters.start_date().set(to_opt_date(event_target_value(&e)))
                            }
                        />
                    </div>

                    <div class="flex flex-col">
                        <label for="end_date">End Date</label>
                        <input
                            class="my-input"
                            type="date"
                            name="end_date"
                            id="end_date"
                            prop:value=move || {
                                filters.end_date().get().map(|end_date| end_date.to_string())
                            }
                            on:input=move |e| {
                                filters.end_date().set(to_opt_date(event_target_value(&e)))
                            }
                        />
                    </div>
                </div>
            </div>
        </div>
        {children()}
    }
}
