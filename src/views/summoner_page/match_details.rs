use crate::views::summoner_page::match_details::match_details_build::MatchDetailsBuild;
use crate::views::summoner_page::match_details::match_details_overview::MatchDetailsOverview;
use crate::views::summoner_page::match_details::match_details_team::MatchDetailsTeam;
use leptos::either::Either;
use leptos::prelude::{signal, ClassAttribute, OnAttribute, Resource, Show, Suspend};
use leptos::prelude::{ElementChild, Transition};
use leptos::{component, view, IntoView};
use serde::{Deserialize, Serialize};
use crate::backend::server_fns::get_match_details::get_match_details;

pub mod match_details_overview;
pub mod match_details_team;
pub mod match_details_build;

#[component]
pub fn MatchDetails(match_id: i32, riot_match_id: String, platform: String, summoner_id: i32) -> impl IntoView {
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
                    view! { <p>"Loading match details ..."</p> }
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
    pub summoner_icon_id: i32,
    pub summoner_level: i64,
    pub champion_id: i32,
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
    pub summoner_spell1_id: i32,
    pub summoner_spell2_id: i32,
    pub perk_defense_id: i32,
    pub perk_flex_id: i32,
    pub perk_offense_id: i32,
    pub perk_primary_style_id: i32,
    pub perk_sub_style_id: i32,
    pub perk_primary_selection_id: i32,
    pub perk_primary_selection1_id: i32,
    pub perk_primary_selection2_id: i32,
    pub perk_primary_selection3_id: i32,
    pub perk_sub_selection1_id: i32,
    pub perk_sub_selection2_id: i32,
    pub item0_id: i32,
    pub item1_id: i32,
    pub item2_id: i32,
    pub item3_id: i32,
    pub item4_id: i32,
    pub item5_id: i32,
    pub item6_id: i32,
    pub items_event_timeline: Vec<(i32, Vec<ItemEvent>)>,
    pub skills_timeline: Vec<i32>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LolMatchTimeline {
    pub id: i32,
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub items_event_timeline: Vec<(i32, Vec<ItemEvent>)>,
    pub skills_timeline: Vec<i32>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemEvent {
    Purchased {
        item_id: i32,
    },
    Sold {
        item_id: i32,
    },
}

impl ItemEvent {
    pub fn get_id(&self) -> i32 {
        match self {
            ItemEvent::Purchased { item_id } => *item_id,
            ItemEvent::Sold { item_id } => *item_id,
        }
    }
}
