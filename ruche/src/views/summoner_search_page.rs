use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::views::summoner_page::summoner_search_page::SummonerSearch;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::components::{ A};



#[component]
pub fn SummonerSearchPage() -> impl IntoView {
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

    batch(||{
        meta_store.title().set("Ruche | High-Performance League of Legends Stats and Profiles".to_string());
        meta_store.description().set("Experience lightning-fast League of Legends statistics and summoner profiles on Ruche. Built with Rust for unmatched performance and efficiency. Search now to elevate your gaming experience.".to_string());
        meta_store.image().set("https://ruche.lol/assets/favicon.ico".to_string());
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
