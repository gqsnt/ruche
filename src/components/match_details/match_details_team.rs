use leptos::prelude::{signal, ClassAttribute, OnAttribute, ReadSignal, Resource, ServerFnError, Suspend, Suspense};
use leptos::{component, view, IntoView};
use leptos::either::Either;
use leptos::prelude::ElementChild;
use crate::models::entities::lol_match_participant::LolMatchParticipantMatchesDetailPage;

#[component]
pub fn MatchDetailsTeam(summoner_id:i32, match_details : ReadSignal<Vec<LolMatchParticipantMatchesDetailPage>>) -> impl IntoView {
    view!{}
}