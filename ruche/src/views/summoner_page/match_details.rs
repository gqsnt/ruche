use crate::app::SummonerIdentifier;
use crate::backend::server_fns::get_match_details::get_match_details;
use crate::utils::{ProPlayerSlug, RiotMatchId, SSEVersions, SSEVersionsStoreFields};
use crate::views::summoner_page::match_details::match_details_build::MatchDetailsBuild;
use crate::views::summoner_page::match_details::match_details_overview::MatchDetailsOverview;
use crate::views::summoner_page::match_details::match_details_team::MatchDetailsTeam;
use bitcode::{Decode, Encode};
use common::consts::platform_route::PlatformRoute;
use leptos::either::{Either, EitherOf3};
use leptos::prelude::*;
use leptos::{component, view, IntoView};

use reactive_stores::Store;
use std::fmt::Formatter;
use std::sync::Arc;

pub mod match_details_build;
pub mod match_details_overview;
pub mod match_details_team;

#[component]
pub fn MatchDetails(
    match_id: i32,
    riot_match_id: RiotMatchId,
    platform: PlatformRoute,
    in_encounter: bool,
) -> impl IntoView {
    let summoner_identifier = expect_context::<Memo<SummonerIdentifier>>();
    let sse_version = expect_context::<Store<SSEVersions>>();

    let match_details = Resource::new_bitcode(
        move || {
            (
                sse_version.match_ver().get(),
                match_id,
                riot_match_id,
                platform,
                summoner_identifier.get(),
            )
        },
        |(_, match_id, riot_match_id, platform, summoner_identifier)| async move {
            get_match_details(match_id, Some(summoner_identifier), platform, riot_match_id).await
        },
    );
    let (match_detail_tab, set_match_detail_tab) = signal(MatchDetailTabs::Overview);

    view! {
        <div class="mt-2 w-full">
            <div class="flex space-x-2 mb-2">
                <button
                    on:click=move |_| set_match_detail_tab(MatchDetailTabs::Overview)
                    class=move || {
                        if matches!(match_detail_tab(), MatchDetailTabs::Overview) {
                            "active-tab"
                        } else {
                            "default-tab"
                        }
                    }
                >
                    Overview
                </button>
                <button
                    on:click=move |_| set_match_detail_tab(MatchDetailTabs::Team)
                    class=move || {
                        if matches!(match_detail_tab(), MatchDetailTabs::Team) {
                            "active-tab"
                        } else {
                            "default-tab"
                        }
                    }
                >
                    Team
                </button>
                <button
                    on:click=move |_| set_match_detail_tab(MatchDetailTabs::Build)
                    class=move || {
                        if matches!(match_detail_tab(), MatchDetailTabs::Build) {
                            "active-tab"
                        } else {
                            "default-tab"
                        }
                    }
                >
                    Build
                </button>
            </div>
            <div>
                <Transition fallback=move || {
                    view! { <div class="text-center">Loading Match Details</div> }
                }>
                    {move || Suspend::new(async move {
                        match match_details.await {
                            Ok(match_details) => {
                                Either::Left({
                                    let match_details = Arc::new(match_details);
                                    match &*match_detail_tab.read() {
                                        MatchDetailTabs::Overview => {
                                            EitherOf3::A(

                                                view! {
                                                    <MatchDetailsOverview
                                                        match_details=match_details
                                                        in_encounter=in_encounter
                                                    />
                                                },
                                            )
                                        }
                                        MatchDetailTabs::Team => {
                                            EitherOf3::B(
                                                view! { <MatchDetailsTeam match_details=match_details /> },
                                            )
                                        }
                                        MatchDetailTabs::Build => {
                                            EitherOf3::C(
                                                view! { <MatchDetailsBuild match_details=match_details /> },
                                            )
                                        }
                                    }
                                })
                            }
                            Err(_) => Either::Right(()),
                        }
                    })}

                </Transition>
            </div>
        </div>
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MatchDetailTabs {
    Overview,
    Team,
    Build,
}

#[derive(Clone, Encode, Decode)]
pub struct LolMatchParticipantDetails {
    pub id: i32,
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub is_self_summoner: bool,
    pub item0_id: u32,
    pub item1_id: u32,
    pub item2_id: u32,
    pub item3_id: u32,
    pub item4_id: u32,
    pub item5_id: u32,
    pub item6_id: u32,
    pub damage_dealt_to_champions: u32,
    pub damage_taken: u32,
    pub gold_earned: u32,
    pub kill_participation: u16,
    pub summoner_icon_id: u16,
    pub summoner_level: u16,
    pub encounter_count: u16,
    pub champion_id: u16,
    pub team_id: u16,
    pub kills: u16,
    pub deaths: u16,
    pub assists: u16,
    pub champ_level: u16,
    pub wards_placed: u16,
    pub cs: u16,
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
    pub won: bool,
    pub platform: PlatformRoute,
    pub game_name: String,
    pub tag_line: String,
    pub summoner_pro_player_slug: Option<ProPlayerSlug>,
    pub items_event_timeline: Vec<(u16, Vec<ItemEvent>)>,
    pub skills_timeline: Vec<Skill>,
}

impl PartialEq for LolMatchParticipantDetails {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone)]
pub struct LolMatchTimeline {
    pub id: i32,
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub skills_timeline: Vec<Skill>,
    pub items_event_timeline: Vec<(u16, Vec<ItemEvent>)>,
}

#[derive(Clone, Encode, Decode)]
#[cfg_attr(feature = "ssr", derive(serde::Serialize, serde::Deserialize))]
pub struct ItemEvent {
    pub item_id: u32,
    pub event_type: ItemEventType,
}

#[repr(u8)]
#[derive(Clone, Encode, Decode, PartialEq)]
#[cfg_attr(feature = "ssr", derive(serde::Serialize, serde::Deserialize))]
pub enum ItemEventType {
    Purchased,
    Sold,
}

#[repr(u8)]
#[derive(Clone, Encode, Decode, PartialEq, Copy)]
#[cfg_attr(feature = "ssr", derive(serde::Serialize, serde::Deserialize))]
pub enum Skill {
    Q = 1,
    W = 2,
    E = 3,
    R = 4,
}

impl std::fmt::Display for Skill {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Skill::Q => write!(f, "Q"),
            Skill::W => write!(f, "W"),
            Skill::E => write!(f, "E"),
            Skill::R => write!(f, "R"),
        }
    }
}

impl From<u8> for Skill {
    fn from(value: u8) -> Self {
        match value {
            1 => Skill::Q,
            2 => Skill::W,
            3 => Skill::E,
            4 => Skill::R,
            _ => panic!("Invalid Skill value: {}", value),
        }
    }
}
