use crate::app::{MetaStore, MetaStoreStoreFields};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::components::A;
use crate::views::components::summoner_search_page::SummonerSearch;

#[component]
pub fn SummonerSearchPage() -> impl IntoView {
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

    batch(|| {
        meta_store
            .title()
            .set("League of Legends Stats & Summoner Search | Ruche".to_string());
        meta_store.description().set(
            "Look up League of Legends summoner profiles, match history, champion stats, encounters, and live games. Full-stack Rust, real-time updates, compact binary payloads."
                .to_string(),
        );
        meta_store.url().set("/".to_string());
        meta_store
            .image()
            .set("https://ruche.lol/assets/logo.avif".to_string());
    });

    view! {
        <div class="my-0 mx-auto max-w-5xl text-center">
            <A href="/" attr:class="p-6 text-4xl my-4">
                "Welcome to Ruche"
            </A>
            <img src="/assets/logo.avif" class="w-[420px] h-[420px] mx-auto" />
            <SummonerSearch is_summoner_page=false />
        </div>
    }
}
