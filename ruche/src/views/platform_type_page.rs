use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::views::summoner_page::summoner_search_page::SummonerSearchPage;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::components::Outlet;
use leptos_router::hooks::use_location;

#[component]
pub fn PlatformTypePage() -> impl IntoView {
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
    let location = use_location();
    meta_store
        .title()
        .set("Ruche | High-Performance League of Legends Stats and Profiles".to_string());
    meta_store.description().set("Experience lightning-fast League of Legends statistics and summoner profiles on Ruche. Built with Rust for unmatched performance and efficiency. Search now to elevate your gaming experience.".to_string());
    meta_store
        .image()
        .set("https://ruche.lol/assets/favicon.ico".to_string());
    meta_store.url().set(location.pathname.get());
    let req_include_summoner = move || location.pathname.get().contains("summoners");
    view! {
        <div class="my-0 mx-auto max-w-5xl text-center">
            <a href="/" class="p-6 text-4xl my-4">
                "Welcome to Ruche"
            </a>
        {move || (
            !req_include_summoner()).then(||view!{<img src="/assets/logo.avif" class="w-[420px] h-[420px] mx-auto" />}
        )}
            <SummonerSearchPage is_summoner_page=Signal::derive(req_include_summoner)/>
            <Outlet />
        </div>
    }
}
