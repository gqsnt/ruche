use crate::consts::champion::CHAMPION_OPTIONS;
use crate::consts::queue::QUEUE_OPTIONS;
use crate::views::{BackEndMatchFiltersSearch};
use leptos::context::provide_context;
use leptos::prelude::*;
use leptos::reactive::wrappers::write::SignalSetter;
use leptos::{component, view, IntoView};
use leptos_router::hooks::query_signal_with_options;
use leptos_router::NavigateOptions;

#[component]
pub fn MatchFilters(children: Children) -> impl IntoView {
    let (start_date, set_start_date) = query_signal_with_options(
        "filters[start_date]",
        NavigateOptions {
            scroll: false,
            replace: true,
            ..Default::default()
        },
    );

    let (end_date, set_end_date) = query_signal_with_options(
        "filters[end_date]",
        NavigateOptions {
            scroll: false,
            replace: true,
            ..Default::default()
        },
    );

    let (champion_id, set_champion_id) = query_signal_with_options::<String>(
        "filters[champion_id]",
        NavigateOptions {
            scroll: false,
            replace: true,
            ..Default::default()
        },
    );

    let (queue_id, set_queue_id) = query_signal_with_options::<String>(
        "filters[queue_id]",
        NavigateOptions {
            scroll: false,
            replace: true,
            ..Default::default()
        },
    );

    let filters_signal = RwSignal::new(BackEndMatchFiltersSearch::from_signals(
        queue_id(),
        champion_id(),
        start_date(),
        end_date(),
    ));
    provide_context(filters_signal);

    let set_optional_value = move |setter: SignalSetter<Option<String>>, value: String, name: &str| {
        let value = if value.is_empty() {
            None
        } else {
            Some(value)
        };
        setter.set(value.clone());
        let filters = if name == "start_date" {
            BackEndMatchFiltersSearch::from_signals(queue_id(), champion_id(), value, end_date())
        } else if name == "end_date" {
            BackEndMatchFiltersSearch::from_signals(queue_id(), champion_id(), start_date(), value)
        } else if name == "champion_id" {
            BackEndMatchFiltersSearch::from_signals(queue_id(), value, start_date(), end_date())
        } else {
            BackEndMatchFiltersSearch::from_signals(value, champion_id(), start_date(), end_date())
        };
        filters_signal.set(filters);
    };

    let champion_options = CHAMPION_OPTIONS.iter().map(|(id, champion)| view! {
        <option value=*id selected=move || *id.to_string() == champion_id().unwrap_or_default()>
            {champion.to_string()}
        </option>
    }).collect::<Vec<_>>();
    let queue_options = QUEUE_OPTIONS.iter().map(|(inner_queue_id, queue_name)| view! {
        <option
            value=*inner_queue_id
            selected=move || inner_queue_id.to_string() == queue_id().unwrap_or_default()
        >
            {queue_name.to_string()}
        </option>
    }).collect::<Vec<_>>();

    view! {
        <div class="flex justify-center">
            <div class="my-card w-[768px]">
                <div class="flex text-left space-x-2">
                    <div class="flex flex-col">
                        <label for="champion_id">Champion</label>
                        <select
                            name="champion_id"
                            class="my-select"
                            id="champion_id"
                            prop:value=move || champion_id().unwrap_or_default()
                            on:change=move |e| set_optional_value(
                                set_champion_id,
                                event_target_value(&e),
                                "champion_id",
                            )
                        >
                            <option value="" selected=move || champion_id().is_none()>
                                All
                            </option>
                            {champion_options}
                        </select>
                    </div>
                    <div class="flex flex-col">
                        <label for="queue_id">Queue</label>
                        <select
                            class="my-select"
                            name="queue_id"
                            id="queue_id"
                            prop:value=move || queue_id().unwrap_or_default()
                            on:change=move |e| set_optional_value(
                                set_queue_id,
                                event_target_value(&e),
                                "queue_id",
                            )
                        >
                            <option value="" selected=move || queue_id().is_none()>
                                All
                            </option>
                            {queue_options}
                        </select>
                    </div>
                    <div class="flex flex-col">
                        <label for="start_date">Start Date</label>
                        <input
                            class="my-input"
                            placeholder="dd-mm-yyyy"
                            type="date"
                            name="start_date"
                            id="start_date"
                            value=start_date()
                            prop:value=move || start_date().unwrap_or_default()
                            on:input=move |e| set_optional_value(
                                set_start_date,
                                event_target_value(&e),
                                "start_date",
                            )
                        />
                    </div>
                    <div class="flex flex-col">
                        <label for="end_date">End Date</label>
                        <input
                            class="my-input"
                            placeholder="dd-mm-yyyy"
                            type="date"
                            name="end_date"
                            id="end_date"
                            value=end_date()
                            prop:value=move || end_date().unwrap_or_default()
                            on:input=move |e| set_optional_value(
                                set_end_date,
                                event_target_value(&e),
                                "end_date",
                            )
                        />
                    </div>
                </div>
            </div>

        </div>
        {children()}
    }
}