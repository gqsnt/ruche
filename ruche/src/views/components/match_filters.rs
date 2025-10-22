use crate::views::{get_default_navigation_option, BackEndMatchFiltersSearch};
use common::consts::champion::CHAMPION_OPTIONS;
use common::consts::queue::{Queue};
use itertools::Itertools;
use leptos::context::provide_context;
use leptos::prelude::*;
use leptos::reactive::wrappers::write::SignalSetter;
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

    let filters_signal = RwSignal::new(BackEndMatchFiltersSearch::from_signals(
        queue_id(),
        champion_id(),
        start_date(),
        end_date(),
    ));
    provide_context(filters_signal);

    enum FilterField {
        StartDate,
        EndDate,
        ChampionId,
        QueueId,
    }

    let set_optional_value = move |setter: SignalSetter<Option<String>>,
                                   value: String,
                                   field: FilterField| {
        let value = if value.is_empty() { None } else { Some(value) };
        setter.set(value.clone());
        let filters = match field {
            FilterField::StartDate => BackEndMatchFiltersSearch::from_signals(
                queue_id(),
                champion_id(),
                value,
                end_date(),
            ),
            FilterField::EndDate => BackEndMatchFiltersSearch::from_signals(
                queue_id(),
                champion_id(),
                start_date(),
                value,
            ),
            FilterField::ChampionId => {
                BackEndMatchFiltersSearch::from_signals(queue_id(), value, start_date(), end_date())
            }
            FilterField::QueueId => BackEndMatchFiltersSearch::from_signals(
                value,
                champion_id(),
                start_date(),
                end_date(),
            ),
        };
        filters_signal.set(filters);
    };

    view! {
        <div class="flex justify-center">
            <div class="my-card w-[768px]">
                <div class="flex text-left space-x-2 justify-center">
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
                                FilterField::ChampionId,
                            )
                        >
                            <option value="" selected=move || champion_id().is_none()>
                                All
                            </option>
                            {CHAMPION_OPTIONS
                                .iter()
                                .map(|(id, champion)| {
                                    view! {
                                        <option
                                            value=*id
                                            selected=move || {
                                                *id.to_string() == champion_id().unwrap_or_default()
                                            }
                                        >
                                            {champion.to_string()}
                                        </option>
                                    }
                                })
                                .collect_vec()}
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
                                FilterField::QueueId,
                            )
                        >
                            <option value="" selected=move || queue_id().is_none()>
                                All
                            </option>
                            {Queue::options_basic()
                                .into_iter()
                                .map(|(inner_queue_id, queue_name)| {
                                    view! {
                                        <option
                                            value=inner_queue_id
                                            selected=move || {
                                                inner_queue_id.to_string() == queue_id().unwrap_or_default()
                                            }
                                        >
                                            {queue_name.to_string()}
                                        </option>
                                    }
                                })
                                .collect_vec()}
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
                                FilterField::StartDate,
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
                                FilterField::EndDate,
                            )
                        />
                    </div>
                </div>
            </div>

        </div>
        {children()}
    }
}
