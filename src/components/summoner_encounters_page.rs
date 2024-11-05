use leptos::{component, view, IntoView};
use leptos::prelude::{expect_context, ReadSignal, Set};
use leptos_meta::{provide_meta_context, Meta, Title};
use crate::models::entities::summoner::Summoner;
use leptos::prelude::ElementChild;
use leptos_router::components::A;
use crate::app::{MetaStore, MetaStoreStoreFields};

#[component]
pub fn SummonerEncountersPage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

    meta_store.title().set(format!("{}#{} | Encounters | Broken.gg", summoner().game_name, summoner().tag_line));
    meta_store.description().set(format!("Discover the top champions played by {}#{}. Access in-depth statistics, win rates, and performance insights on Broken.gg, powered by Rust for optimal performance.", summoner().game_name, summoner().tag_line));
    meta_store.url().set(format!("{}?tab=encounters",summoner().to_route_path()));
    view! {<h2>"Summoner Encounters Page"</h2> }
}