use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::components::summoner_search_page::SummonerSearchPage;
use leptos::prelude::ElementChild;
use leptos::prelude::{expect_context, use_context, ClassAttribute, Get, Set, Show};
use leptos::{component, view, IntoView};
use leptos_router::components::Outlet;

#[component]
pub fn PlatformTypePage() -> impl IntoView {
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

    meta_store.title().set("Broken.gg | High-Performance League of Legends Stats and Profiles".to_string());
    meta_store.description().set("Experience lightning-fast League of Legends statistics and summoner profiles on Broken.gg. Built with Rust for unmatched performance and efficiency. Search now to elevate your gaming experience.".to_string());
    meta_store.image().set("https://next-level.xyz/favicon.ico".to_string());
    meta_store.url().set("".to_string());

    let req_include_summoner = || {
        let context = use_context::<leptos_router::components::RouterContext>();
        if let Some(router_context) = context {
            router_context.current_url.read_only().get().path().contains("summoners")
        } else {
            false
        }
    };

    view! {
        <div class="my-0 mx-auto max-w-5xl text-center">
            <a href="/" class="p-6 text-4xl my-4">
                "Welcome to Broken.gg"
            </a>
            <Show when=move || !req_include_summoner()>
                <img src="/logo.webp" class="w-[420px] h-[420px] mx-auto" />
            </Show>
            <SummonerSearchPage />
            <Outlet />
        </div>
    }
}