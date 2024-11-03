use crate::apis::get_match_details;
use leptos::prelude::{signal, ClassAttribute, OnAttribute, Resource, ServerFnError, Show, Suspend, Suspense};
use leptos::{component, view, IntoView};
use leptos::either::Either;
use leptos::prelude::ElementChild;
use crate::components::match_details::match_details_build::MatchDetailsBuild;
use crate::components::match_details::match_details_overview::MatchDetailsOverview;
use crate::components::match_details::match_details_team::MatchDetailsTeam;

pub mod match_details_overview;
pub mod match_details_team;
pub mod match_details_build;

#[component]
pub fn MatchDetails(match_id: i32, riot_match_id:String, platform:String, summoner_id:i32) -> impl IntoView {
    let match_details = Resource::new(
        move || (match_id, riot_match_id.clone(), platform.clone()),
        |(match_id, riot_match_id, platform)| async move {
            get_match_details(match_id, riot_match_id, platform).await
        },
    );
    let (match_detail_tab, set_match_detail_tab) = signal("overview".to_string());
    let match_detail_view = Suspend::new(async move {
        match match_details.await {
            Ok(match_details) => Either::Left({
                let (match_details_signal, _) = signal(match_details);
                view! {
                    <Show when=move || match_detail_tab() == "overview">
                        <MatchDetailsOverview
                            match_details=match_details_signal
                            summoner_id=summoner_id
                        />
                    </Show>
                    <Show when=move || match_detail_tab() == "team">
                        <MatchDetailsTeam
                            match_details=match_details_signal
                            summoner_id=summoner_id
                        />
                    </Show>
                    <Show when=move || match_detail_tab() == "build">
                        <MatchDetailsBuild
                            match_details=match_details_signal
                            summoner_id=summoner_id
                        />
                    </Show>
                }
            }),
            Err(err) => Either::Right(view!{})
        }
    });

    view! {
        <div class="mt-2 w-full">
            <div class="flex">
                <button
                    on:click=move |e| set_match_detail_tab("overview".to_string())
                    class=move || {
                        if match_detail_tab() == "overview" { "active-tab" } else { "default-tab" }
                    }
                >
                    Overview
                </button>
                <button
                    on:click=move |e| set_match_detail_tab("team".to_string())
                    class=move || {
                        if match_detail_tab() == "team" { "active-tab" } else { "default-tab" }
                    }
                >
                    Team
                </button>
                <button
                    on:click=move |e| set_match_detail_tab("build".to_string())
                    class=move || {
                        if match_detail_tab() == "build" { "active-tab" } else { "default-tab" }
                    }
                >
                    Build
                </button>
            </div>
            <div>
                <Suspense fallback=move || {
                    view! { <p>"Loading..."</p> }
                }>{match_detail_view}</Suspense>
            </div>
        </div>
    }
}