use crate::views::summoner_page::match_details::LolMatchParticipantDetails;
use leptos::prelude::ReadSignal;
use leptos::{component, view, IntoView};

#[component]
pub fn MatchDetailsTeam(summoner_id: i32, match_details: ReadSignal<Vec<LolMatchParticipantDetails>>) -> impl IntoView {
    view! {}
}