use crate::views::get_default_navigation_option;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::hooks::query_signal_with_options;

#[component]
pub fn Pagination(max_page: u16) -> impl IntoView {
    let (page, set_page) =
        query_signal_with_options::<u16>("page", get_default_navigation_option());


    let current_page = Memo::new(move |_| page().unwrap_or(1).clamp(1, max_page));

    let go_to_prev_page = move || {
        if current_page() > 1 {
            set_page(Some(current_page() - 1));
        }
    };
    let go_to_next_page = move || {
        if current_page() < max_page {
            set_page(Some(current_page() + 1));
        }
    };

    view! {
        <nav class="flex items-center justify-center mt-4" aria-label="Page navigation">
            <ul class="flex space-x-1">
                // Previous Button
                <li>
                    <button
                        class=move || {
                            if current_page() <= 1 { "disabled-tab" } else { "active-tab" }
                        }
                        disabled=move || current_page() <= 1
                        on:click=move |_| go_to_prev_page()
                        aria-label="Previous"
                    >
                        {"Previous"}
                    </button>
                </li>
                // Page Numbers
                <For
                    each=move || get_display_pages(current_page(), max_page)
                    key=|k| k.unwrap_or_default()
                    let:opt
                >
                    {
                    match opt {
                        None => {
                            Either::Right(

                                view! {
                                    <li>
                                        <span class="ellipsis">{"..."}</span>
                                    </li>
                                },
                            )
                        }
                        Some(page_num) => {
                            let is_current = page_num == current_page();
                            Either::Left(
                                view! {
                                    <li>
                                        <button
                                            class=move || {
                                                if is_current { "active-tab" } else { "default-tab" }
                                            }
                                            on:click=move |_| set_page(Some(page_num))
                                            aria-current=move || {
                                                if is_current { Some("page") } else { None }
                                            }
                                        >
                                            {page_num}
                                        </button>
                                    </li>
                                },
                            )
                        }
                    }}
                </For>
                // Next Button
                <li>
                    <button
                        class=move || {
                            if current_page() >= max_page { "disabled-tab" } else { "active-tab" }
                        }
                        disabled=move || (current_page() >= max_page)
                        on:click=move |_| go_to_next_page()
                        aria-label="Next"
                    >
                        {"Next"}
                    </button>
                </li>
            </ul>
        </nav>
    }
}

fn get_display_pages(current_page: u16, total_pages: u16) -> Vec<Option<u16>> {
    let mut pages = Vec::new();

    if total_pages <= 7 {
        // Show all pages
        for i in 1..=total_pages {
            pages.push(Some(i));
        }
    } else {
        pages.push(Some(1));

        if current_page > 4 {
            pages.push(None); // Ellipsis
        }

        let start = if current_page > 4 {
            current_page - 2
        } else {
            2
        };

        let end = if current_page < total_pages - 3 {
            current_page + 2
        } else {
            total_pages - 1
        };

        for i in start..=end {
            pages.push(Some(i));
        }

        if current_page < total_pages - 3 {
            pages.push(None); // Ellipsis
        }

        pages.push(Some(total_pages));
    }

    pages
}
