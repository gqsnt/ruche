use crate::models::entities::lol_match_participant::LolMatchParticipantMatchesDetailPage;
use leptos::prelude::ReadSignal;
use leptos::{component, view, IntoView};

#[component]
pub fn MatchDetailsTeam(summoner_id: i32, match_details: ReadSignal<Vec<LolMatchParticipantMatchesDetailPage>>) -> impl IntoView {
    view! {}
}