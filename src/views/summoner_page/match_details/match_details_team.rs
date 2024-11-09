use leptos::prelude::ReadSignal;
use leptos::{component, view, IntoView};
use crate::views::summoner_page::match_details::LolMatchParticipantDetails;

#[component]
pub fn MatchDetailsTeam(summoner_id: i32, match_details: ReadSignal<Vec<LolMatchParticipantDetails>>) -> impl IntoView {
    view! {}
}