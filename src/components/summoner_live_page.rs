use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::models::entities::summoner::Summoner;
use leptos::prelude::ElementChild;
use leptos::prelude::{expect_context, ReadSignal, Set};
use leptos::{component, view, IntoView};

#[component]
pub fn SummonerLivePage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

    meta_store.title().set(format!("{}#{} | Live Game | Broken.gg", summoner().game_name, summoner().tag_line));
    meta_store.description().set(format!("Watch {}#{}'s live game now on Broken.gg. Get real-time updates and analytics with our ultra-fast, Rust-based League of Legends companion.", summoner().game_name, summoner().tag_line));
    meta_store.url().set(format!("{}?tab=live", summoner().to_route_path()));
    view! { <h1>"Summoner Live Page"</h1> }
}