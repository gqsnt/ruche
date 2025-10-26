use crate::views::{get_default_navigation_option, BackEndMatchFiltersSearch};
use common::consts::champion::CHAMPION_OPTIONS;
use common::consts::queue::{Queue};
use leptos::context::provide_context;
use leptos::prelude::*;

use leptos::{component, view, IntoView};
use leptos_router::hooks::query_signal_with_options;

#[component]
pub fn MatchFilters(children: Children) -> impl IntoView {
    let (start_date, set_start_date) =
        query_signal_with_options("filters[start_date]", get_default_navigation_option());

    let (end_date, set_end_date) =
        query_signal_with_options("filters[end_date]", get_default_navigation_option());

    let (champion_id, set_champion_id) = query_signal_with_options::<String>(
        "filters[champion_id]",
        get_default_navigation_option(),
    );

    let (queue_id, set_queue_id) =
        query_signal_with_options::<String>("filters[queue_id]", get_default_navigation_option());
    let (_, set_page) = query_signal_with_options::<u16>("page", get_default_navigation_option());
    let filters_signal = RwSignal::new(BackEndMatchFiltersSearch::from_signals(
        queue_id.get_untracked(),
        champion_id.get_untracked(),
        start_date.get_untracked(),
        end_date.get_untracked(),
    ));

    Effect::new(move |_| {
        filters_signal.set(BackEndMatchFiltersSearch::from_signals(
            queue_id(), champion_id(), start_date(), end_date()
        ));
    });

    Effect::new(move |_| {
        let _ = filters_signal.get(); // track
        set_page(None);
    });
    provide_context(filters_signal);
    let to_opt = |v: String| if v.is_empty() { None } else { Some(v) };

    view! {
        <div class="flex justify-center my-2">
            <div class="my-card w-[768px]">
                <div class="flex text-left space-x-2 justify-center">
                    <div class="flex flex-col">
                        <label for="champion_id">Champion</label>
                        <select
                            name="champion_id"
                            class="my-select"
                            id="champion_id"
                            prop:value=move || champion_id().unwrap_or_default()
                            on:change=move |e| set_champion_id(to_opt(event_target_value(&e)))
                        >
                            <option value="">All</option>
                            <For
                                each=|| CHAMPION_OPTIONS.iter().cloned()
                                key=|(id, _)| *id
                                let:opt
                            >
                                { let (id, label) = opt; view!{ <option value=id>{label}</option> } }
                            </For>
                        </select>
                    </div>

                    <div class="flex flex-col">
                        <label for="queue_id">Queue</label>
                        <select
                            class="my-select"
                            name="queue_id"
                            id="queue_id"
                            prop:value=move || queue_id().unwrap_or_default()
                            on:change=move |e| set_queue_id(to_opt(event_target_value(&e)))
                        >
                            <option value="">All</option>
                            <For
                                each=|| Queue::options_basic()
                                key=|(id, _)| *id
                                let:opt
                            >
                                { let (id, label) = opt; view!{ <option value=id>{label}</option> } }
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
                            prop:value=move || start_date().unwrap_or_default()
                            on:input=move |e| set_start_date(to_opt(event_target_value(&e)))
                        />
                    </div>

                    <div class="flex flex-col">
                        <label for="end_date">End Date</label>
                        <input
                            class="my-input"
                            type="date"
                            name="end_date"
                            id="end_date"
                            prop:value=move || end_date().unwrap_or_default()
                            on:input=move |e| set_end_date(to_opt(event_target_value(&e)))
                        />
                    </div>
                </div>
            </div>
        </div>
        {children()}
    }
}
