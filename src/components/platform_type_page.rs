use crate::components::summoner_search_page::SummonerSearchPage;
use leptos::prelude::{expect_context, signal, ClassAttribute, Set};
use leptos::prelude::ElementChild;
use leptos::{component, view, IntoView};
use leptos::context::provide_context;
use leptos_meta::{provide_meta_context, Link, Meta, Title};
use leptos_router::components::Outlet;
use crate::app::{MetaStore, MetaStoreStoreFields};

#[component]
pub fn PlatformTypePage() -> impl IntoView {
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
    meta_store.title().set("Broken.gg | High-Performance League of Legends Stats and Profiles".to_string());
    meta_store.description().set("Experience lightning-fast League of Legends statistics and summoner profiles on Broken.gg. Built with Rust for unmatched performance and efficiency. Search now to elevate your gaming experience.".to_string());
    meta_store.image().set("/favicon.ico".to_string());
    meta_store.url().set("".to_string());
    view! {
        <div class="my-0 mx-auto max-w-3xl text-center">
            <a href="/" class="p-6 text-4xl my-4">
                "Welcome to Broken.gg"
            </a>
            <SummonerSearchPage />
            <Outlet />
        </div>
    }
}