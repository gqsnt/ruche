use crate::views::summoner_page::match_details::LolMatchParticipantDetails;
use leptos::{component, IntoView};
use std::sync::Arc;

#[component]
pub fn MatchDetailsTeam(_match_details: Arc<Vec<LolMatchParticipantDetails>>) -> impl IntoView {}
