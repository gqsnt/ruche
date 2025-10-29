use std::sync::Arc;
use crate::views::summoner_page::match_details::LolMatchParticipantDetails;
use leptos::{component, IntoView};


#[component]
pub fn MatchDetailsTeam(
    _match_details: Arc<Vec<LolMatchParticipantDetails>>,
) -> impl IntoView {
}
