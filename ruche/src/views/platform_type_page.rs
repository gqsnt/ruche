use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::views::summoner_page::summoner_search_page::SummonerSearchPage;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::components::{Outlet, A};
use leptos_router::hooks::{ use_params_map};


#[component]
pub fn PlatformTypePage() -> impl IntoView {
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
    let params = use_params_map();
    meta_store.title().set("Ruche | High-Performance League of Legends Stats and Profiles".to_string());
    meta_store.description().set("Experience lightning-fast League of Legends statistics and summoner profiles on Ruche. Built with Rust for unmatched performance and efficiency. Search now to elevate your gaming experience.".to_string());
    meta_store.image().set("https://ruche.lol/assets/favicon.ico".to_string());

    let location = leptos_router::hooks::use_location();


    Effect::new(move |_| {
        meta_store.url().set(location.pathname.get());
    });

    let is_summoner_page = Memo::new(move |_| params.read().get("summoner_slug").is_some());

    view! {
        <div class="my-0 mx-auto max-w-5xl text-center">
            <A href="/" attr:class="p-6 text-4xl my-4">"Welcome to Ruche"</A>
            { move || (!is_summoner_page()).then(|| view!{
                <img src="/assets/logo.avif" class="w-[420px] h-[420px] mx-auto" />
            }) }
            <SummonerSearchPage is_summoner_page=is_summoner_page />
            <Outlet />
        </div>
    }
}
