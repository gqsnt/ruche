use serde::{Deserialize, Serialize};
use crate::models::entities::lol_match::LolMatch;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LolMatchDefaultParticipantMatchesPage{
    pub summoner_id: i32,
    pub match_id: i32,
    pub match_ended_since: String,
    pub match_duration: String,
    pub queue: String,
    pub champion_id: i32,
    pub won: bool,
    pub champ_level:i32,
    pub kda: f64,
    pub kill_participation: f64,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub summoner_spell1_id : i32,
    pub summoner_spell2_id : i32,
    pub perk_primary_selection_id: i32,
    pub perk_sub_style_id: i32,
    pub item0_id: i32,
    pub item1_id: i32,
    pub item2_id: i32,
    pub item3_id: i32,
    pub item4_id: i32,
    pub item5_id: i32,
    pub item6_id: i32,

    pub participants: Vec<LolMatchParticipantMatchesPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LolMatchParticipantMatchesPage{
    pub lol_match_id: i32,
    pub summoner_id:i32,
    pub summoner_name: String,
    pub summoner_tag_line: String,
    pub summoner_platform: String,
    pub champion_id: i32,
    pub team_id:i32,
}


/// Represents a participant in a League of Legends match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LolMatchParticipant {
    pub id: i32,
    pub champion_id: i32,
    pub summoner_id: i32,
    pub lol_match_id: i32,
    pub summoner_spell1_id: i32,
    pub summoner_spell2_id: i32,
    pub team_id: i32,
    pub won: bool,
    pub kda: f64,
    pub kill_participation: f64,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub champ_level: i32,
    pub stats: LolMatchParticipantStats,
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
}

/// Statistics for a match participant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LolMatchParticipantStats {
    pub minions_killed: i32,
    pub largest_killing_spree: i32,
    pub double_kills: i32,
    pub triple_kills: i32,
    pub quadra_kills: i32,
    pub penta_kills: i32,
    pub total_damage_dealt_to_champions: i32,
    pub total_damage_taken: i32,
    pub gold_earned: i32,
    pub wards_placed: i32,
}
