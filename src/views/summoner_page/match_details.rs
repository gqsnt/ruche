use crate::backend::server_fns::get_match_details::get_match_details;
use crate::views::summoner_page::match_details::match_details_build::MatchDetailsBuild;
use crate::views::summoner_page::match_details::match_details_overview::MatchDetailsOverview;
use crate::views::summoner_page::match_details::match_details_team::MatchDetailsTeam;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::server_fn::serde::{Deserialize, Serialize};
use leptos::{component, view, IntoView};
use crate::views::summoner_page::Summoner;

pub mod match_details_overview;
pub mod match_details_team;
pub mod match_details_build;

#[component]
pub fn MatchDetails(match_id: i32, riot_match_id: String, platform: String, summoner: ReadSignal<Summoner>) -> impl IntoView {
    let match_details = Resource::new(
        move || (match_id, riot_match_id.clone(), platform.clone(), summoner().id),
        |(match_id, riot_match_id, platform,summoner_id)| async move {
            get_match_details(match_id, riot_match_id, platform, Some(summoner_id)).await
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
                            summoner
                        />
                    </Show>
                    <Show when=move || match_detail_tab() == "team">
                        <MatchDetailsTeam
                            _match_details=match_details_signal
                            _summoner_id=summoner().id
                        />
                    </Show>
                    <Show when=move || match_detail_tab() == "build">
                        <MatchDetailsBuild
                            match_details=match_details_signal
                            summoner_id=summoner().id
                        />
                    </Show>
                }
            }),
            Err(_) => Either::Right(())
        }
    });

    view! {
        <div class="mt-2 w-full">
            <div class="flex space-x-2 mb-2">
                <button
                    on:click=move |_| set_match_detail_tab("overview".to_string())
                    class=move || {
                        if match_detail_tab() == "overview" { "active-tab" } else { "default-tab" }
                    }
                >
                    Overview
                </button>
                <button
                    on:click=move |_| set_match_detail_tab("team".to_string())
                    class=move || {
                        if match_detail_tab() == "team" { "active-tab" } else { "default-tab" }
                    }
                >
                    Team
                </button>
                <button
                    on:click=move |_| set_match_detail_tab("build".to_string())
                    class=move || {
                        if match_detail_tab() == "build" { "active-tab" } else { "default-tab" }
                    }
                >
                    Build
                </button>
            </div>
            <div>
                <Transition fallback=move || {
                    view! { <div class="text-center">Loading Match Details</div>}
                }>{match_detail_view}</Transition>
            </div>
        </div>
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LolMatchParticipantDetails {
    pub id: i32,
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub summoner_name: String,
    pub summoner_tag_line: String,
    pub summoner_platform: String,
    pub summoner_pro_player_slug: Option<String>,
    pub summoner_icon_id: u16,
    pub summoner_level: i64,
    pub encounter_count: i32,
    pub champion_id: u16,
    pub team_id: i32,
    pub won: bool,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub champ_level: i32,
    pub kda: f64,
    pub kill_participation: f64,
    pub damage_dealt_to_champions: i32,
    pub damage_taken: i32,
    pub gold_earned: i32,
    pub wards_placed: i32,
    pub cs: i32,
    pub summoner_spell1_id: u16,
    pub summoner_spell2_id: u16,
    pub perk_defense_id: u16,
    pub perk_flex_id: u16,
    pub perk_offense_id: u16,
    pub perk_primary_style_id: u16,
    pub perk_sub_style_id: u16,
    pub perk_primary_selection_id: u16,
    pub perk_primary_selection1_id: u16,
    pub perk_primary_selection2_id: u16,
    pub perk_primary_selection3_id: u16,
    pub perk_sub_selection1_id: u16,
    pub perk_sub_selection2_id: u16,
    pub item0_id: u32,
    pub item1_id: u32,
    pub item2_id: u32,
    pub item3_id: u32,
    pub item4_id: u32,
    pub item5_id: u32,
    pub item6_id: u32,
    pub items_event_timeline: Vec<(u32, Vec<ItemEvent>)>,
    pub skills_timeline: Vec<i32>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LolMatchTimeline {
    pub id: i32,
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub items_event_timeline: Vec<(u32, Vec<ItemEvent>)>,
    pub skills_timeline: Vec<i32>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemEvent {
    pub item_id: u32,
    pub event_type: ItemEventType,
}


#[repr(u8)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ItemEventType {
    Purchased,
    Sold,
}


