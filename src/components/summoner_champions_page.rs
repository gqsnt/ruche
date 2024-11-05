use leptos::prelude::{expect_context, ElementChild, ReadSignal, Set};
use leptos::{component, view, IntoView};
use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::models::entities::summoner::Summoner;

#[component]
pub fn SummonerChampionsPage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

    meta_store.title().set(format!("{}#{} | Champions | Broken.gg", summoner().game_name, summoner().tag_line));
    meta_store.description().set(format!("Discover the top champions played by {}#{} on League Of Legends. Access in-depth statistics, win rates, and performance insights on Broken.gg, powered by Rust for optimal performance.", summoner().game_name, summoner().tag_line));
    meta_store.url().set(format!("{}?tab=champions",summoner().to_route_path()));
    view! { <h2>"Summoner Champions Page"</h2> }
}