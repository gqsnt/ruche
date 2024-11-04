use crate::components::summoner_search_page::SummonerSearchPage;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::{component, view, IntoView};
use leptos_router::components::Outlet;

#[component]
pub fn PlatformTypePage() -> impl IntoView {
    view! {
        <main class="my-0 mx-auto max-w-3xl text-center">
            <a href="/" class="p-6 text-4xl">
                "Welcome to Broken.gg"
            </a>
            <SummonerSearchPage />
            <Outlet />
        </main>
    }
}