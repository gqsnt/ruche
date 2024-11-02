use itertools::Itertools;
use leptos::{component, view, IntoView, Params};
use leptos::context::provide_context;
use leptos::either::Either;
use leptos::prelude::{event_target_value, Children, ClassAttribute, Get, OnAttribute, PropAttribute, Read, RwSignal, Set, With};
use leptos_router::hooks::{query_signal_with_options, use_navigate, use_query, use_query_map};
use serde::{Deserialize, Serialize};
use leptos_router::params::Params;
use leptos::prelude::ElementChild;
use leptos::prelude::BindAttribute;
use leptos::reactive::wrappers::write::SignalSetter;
use leptos::server_fn::codec::IntoReq;
use leptos_router::NavigateOptions;
use reactive_stores::StoreFieldIterator;
use strum::IntoEnumIterator;
use crate::apis::MatchFiltersSearch;
use crate::consts::{Champion, Queue, QUEUE_OPTIONS};



#[component]
pub fn MatchFilters(children:Children) -> impl IntoView {
    let query_map = use_query_map();
    let tab = move || {
        query_map.read().get("tab").unwrap_or(String::from("matches"))
    };



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

    let (queue_id, set_queue_id)= query_signal_with_options::<String>(
        "filters[queue_id]",
        NavigateOptions {
            scroll: false,
            replace: true,
            ..Default::default()
        },
    );

    let filters_signal = RwSignal::new(MatchFiltersSearch::from_signals(
        queue_id(),
        champion_id(),
        start_date(),
        end_date(),
    ));
    provide_context(filters_signal);

    let set_optional_value = move |setter:SignalSetter<Option<String>>, value:String, name:&str|{
        let value = if value.is_empty(){
            None
        }else{
            Some(value)
        };
        setter.set(value.clone());
        let filters = if name == "start_date"{
            MatchFiltersSearch::from_signals(queue_id(),champion_id(),value,end_date())
        }else if name == "end_date" {
            MatchFiltersSearch::from_signals(queue_id(),champion_id(),start_date(),value)
        }
        else if name == "champion_id"{
            MatchFiltersSearch::from_signals(queue_id(),value,start_date(),end_date())
        }else{
            MatchFiltersSearch::from_signals(value,champion_id(),start_date(),end_date())
        };
        filters_signal.set(filters);
    };

    let champion_options = Champion::iter().filter(|c|c != &Champion::UNKNOWN).map(|champion|view! {
        <option
            value=champion as i32
            selected=move || (champion as i32).to_string() == champion_id().unwrap_or_default()
        >
            {format!("{}", champion)}
        </option>
    }).collect::<Vec<_>>();
    let queue_options = QUEUE_OPTIONS.iter().map(|(inner_queue_id, queue_name)|view! {
        <option
            value=*inner_queue_id
            selected=move || inner_queue_id.to_string() == queue_id().unwrap_or_default()
        >
            {queue_name.to_string()}
        </option>
    }).collect::<Vec<_>>();

    view! {
        <div>
            <div>
                <div class="flex mb-4 text-left">
                    <div>
                        <label>Champion</label>
                        <select
                            name="champion_id"
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
                    <div>
                        <label>Queue</label>
                        <select
                            name="queue_id"
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
                    <div>
                        <label>Start Date</label>
                        <input
                            placeholder="dd-mm-yyyy"
                            type="date"
                            name="start_date"
                            value=start_date()
                            prop:value=move || start_date().unwrap_or_default()
                            on:input=move |e| set_optional_value(
                                set_start_date,
                                event_target_value(&e),
                                "start_date",
                            )
                        />
                    </div>
                    <div>
                        <label>End Date</label>
                        <input
                            placeholder="dd-mm-yyyy"
                            type="date"
                            name="end_date"
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
            {children()}
        </div>
    }
}