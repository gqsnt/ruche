use crate::apis::{get_summoner_encounters, MatchFiltersSearch};
use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::views::components::pagination::Pagination;
use crate::models::entities::summoner::{summoner_route_path, Summoner};
use leptos::either::Either;
use leptos::prelude::{event_target_value, expect_context, OnAttribute, ReadSignal, Set, Suspend, Suspense};
use leptos::prelude::{signal, ClassAttribute, Effect, ElementChild, For, Get, Resource, RwSignal, Show};
use leptos::{component, view, IntoView};
use leptos_router::hooks::query_signal_with_options;
use leptos_router::NavigateOptions;
use crate::consts::ProfileIcon;

#[component]
pub fn SummonerEncountersPage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
    let match_filters_updated = expect_context::<RwSignal<MatchFiltersSearch>>();

    let (page_number, set_page_number) = query_signal_with_options::<i32>(
        "page",
        NavigateOptions {
            scroll: false,
            replace: true,
            ..Default::default()
        },
    );

    let (search_summoner, set_search_summoner) = query_signal_with_options::<String>(
        "q",
        NavigateOptions {
            scroll: false,
            replace: true,
            ..Default::default()
        },
    );
    let (search_summoner_signal, set_search_summoner_signal) = signal(search_summoner.get().unwrap_or_default());


    let (reset_page_number, set_reset_page_number) = signal::<bool>(false);
    Effect::new(move |_| {
        if reset_page_number() {
            set_page_number(None);
            set_reset_page_number(false);
        }
    });

    let encounters_resource = Resource::new(
        move || (search_summoner.get(), match_filters_updated.get(), summoner(), page_number()),
        |(search_summoner,filters, summoner, page_number)| async move {
            //println!("{:?} {:?} {:?}", filters, summoner, page_number);
            get_summoner_encounters(summoner.id, page_number.unwrap_or(1), Some(filters), search_summoner).await
        },
    );

    meta_store.title().set(format!("{}#{} | Encounters | Broken.gg", summoner().game_name, summoner().tag_line));
    meta_store.description().set(format!("Discover the top champions played by {}#{}. Access in-depth statistics, win rates, and performance insights on Broken.gg, powered by Rust for optimal performance.", summoner().game_name, summoner().tag_line));
    meta_store.url().set(format!("{}?tab=encounters", summoner().to_route_path()));
    view! {
        <div>
            <div class="my-card flex space-x-2 my-2 w-fit">
                <input
                    type="text"
                    class="my-input"
                    placeholder="Search for a summoner"
                    value=search_summoner_signal.get()
                    on:input=move |e| { set_search_summoner_signal(event_target_value(&e)) }
                />
                <button
                    class="my-button"
                    on:click=move |e| {
                        set_search_summoner(Some(search_summoner_signal.get()));
                    }
                >
                    Search
                </button>
                <button
                    class="my-button bg-red-700 hover:bg-red-800 text-gray-200"
                    on:click=move |e| {
                        set_search_summoner(None);
                        set_search_summoner_signal(String::new());
                    }
                >
                    Clear
                </button>
            </div>
            <Suspense fallback=move || {
                view! { <p>Loading Encounters ...</p> }
            }>
                {move || Suspend::new(async move {
                    match encounters_resource.await {
                        Ok(encounters_result) => {
                            let total_pages = encounters_result.total_pages;
                            let current_page = page_number().unwrap_or(1);
                            if total_pages == 0 || (total_pages as i32) < current_page {
                                set_reset_page_number(true);
                            }
                            if !encounters_result.encounters.is_empty() {
                                Ok(
                                    Either::Left(
                                        view! {
                                            <div class="flex my-card w-fit">
                                                <table class="text-gray-200 space-y-2 ">
                                                    <thead>
                                                        <tr>
                                                            <th class="text-left">Summoner</th>
                                                            <th class="text-left">Encounters</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        <For
                                                            each=move || encounters_result.encounters.clone()
                                                            key=|encounter| encounter.id
                                                            let:encounter
                                                        >
                                                            <tr>
                                                                <td class="text-left">
                                                                    <div class="flex items-center py-0.5">
                                                                        <div>
                                                                            <img
                                                                                alt="Profile Icon"
                                                                                src=ProfileIcon::get_static_url(encounter.profile_icon_id)
                                                                                class="w-8 h-8 rounded"
                                                                                height="32"
                                                                                width="32"
                                                                            />
                                                                        </div>
                                                                        <div class="ml-2">
                                                                            <a
                                                                                href=summoner_route_path(
                                                                                    encounter.platform.clone().as_str(),
                                                                                    encounter.game_name.clone().as_str(),
                                                                                    encounter.tag_line.clone().as_str(),
                                                                                )
                                                                                class="text-blue-300 hover:underline"
                                                                            >
                                                                                {encounter.game_name.clone()}
                                                                            </a>
                                                                        </div>
                                                                    </div>
                                                                </td>
                                                                <td>{encounter.count}</td>
                                                            </tr>
                                                        </For>
                                                    </tbody>
                                                </table>
                                            </div>

                                            <Show when=move || (total_pages > 1)>
                                                <Pagination max_page=(move || total_pages as usize)() />
                                            </Show>
                                        },
                                    ),
                                )
                            } else {
                                Ok(
                                    Either::Right(
                                        view! { <div class="text-center">No encounters found</div> },
                                    ),
                                )
                            }
                        }
                        Err(e) => Err(e),
                    }
                })}
            </Suspense>

        </div>
    }
}