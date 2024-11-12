use crate::views::summoner_page::match_details::LolMatchParticipantDetails;
use leptos::prelude::*;
use leptos::{component, IntoView};

#[component]
pub fn MatchDetailsTeam(_summoner_id: i32, _match_details: ReadSignal<Vec<LolMatchParticipantDetails>>) -> impl IntoView {
}