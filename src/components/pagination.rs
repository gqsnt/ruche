use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::hooks::query_signal_with_options;
use leptos_router::NavigateOptions;

#[component]
pub fn Pagination(max_page: usize) -> impl IntoView {
    // Use query_signal_with_options to get the current page from query parameters
    let (page, set_page) = query_signal_with_options::<usize>("page", NavigateOptions {
        scroll: false,
        replace: true,
        ..Default::default()
    });

    // Set default page to 1 if not present
    let current_page = move || page().unwrap_or(1).clamp(1, max_page);

    // Handlers for Previous and Next buttons
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

    // Function to generate page numbers to display
    let page_numbers = move || {
        let mut pages = vec![];
        let current = current_page();

        if max_page <= 7 {
            // Show all pages if the total number is small
            pages.extend(1..=max_page);
        } else {
            // Always show the first page
            pages.push(1);

            if current > 4 {
                // Add ellipsis if current page is beyond the fourth page
                pages.push(0); // 0 represents '...'
            }

            // Determine the range of page numbers to display around the current page
            let start = if current > 4 { current - 1 } else { 2 };
            let end = if current < max_page - 3 { current + 1 } else { max_page - 1 };

            pages.extend(start..=end);

            if current < max_page - 3 {
                // Add ellipsis if current page is before the last few pages
                pages.push(0); // 0 represents '...'
            }

            // Always show the last page
            pages.push(max_page);
        }
        pages
    };

    view! {
        <nav class="flex items-center justify-center space-x-2 mt-4">
            // Previous Button
            <button
                class=move || {
                    if current_page() <= 1 {
                        "bg-gray-300 text-gray-500 px-4 py-2 rounded cursor-not-allowed"
                    } else {
                        "bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
                    }
                }
                disabled=move || (current_page() <= 1)
                on:click=move |_| go_to_prev_page()
            >
                {"Previous"}
            </button>

            {
                    page_numbers().into_iter().map(|p| {
                        if p == 0 {
                            // Render ellipsis
                            Either::Left(view!{<li class="ellipsis">{"..."}</li> })
                        } else {
                            // Render page number
                            let is_current = p == current_page();
                            let set_p = set_page.clone();
                            Either::Right(view! {
                                <button
                                    class=move || {
                                           if is_current {
                                               "bg-blue-500 text-white px-3 py-1 rounded"
                                           } else {
                                               "bg-white text-blue-500 px-3 py-1 rounded hover:bg-blue-100"
                                           }
                                       }
                                    on:click=move |_| set_p(Some(p))
                                    disabled=move || is_current
                                >
                                    {p}
                                </button>
                            })
                        }
                    }).collect::<Vec<_>>()
                }

            // Next Button
            <button
                class=move || {
                    if current_page() >= max_page  {
                        "bg-gray-300 text-gray-500 px-4 py-2 rounded cursor-not-allowed"
                    } else {
                        "bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
                    }
                }
                on:click=move |_| go_to_next_page()
                disabled=move || (current_page() >= max_page)

            >
                {"Next"}
            </button>
        </nav>
    }
}