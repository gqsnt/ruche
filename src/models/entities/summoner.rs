use crate::consts::PlatformRoute;
use serde::{Deserialize, Serialize};

/// Represents a League of Legends summoner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Summoner {
    pub id: i32,
    pub game_name: String,
    pub tag_line: String,
    pub puuid: String,
    pub platform: PlatformRoute,
    pub updated_at: String,
    pub summoner_level: i64,
    pub profile_icon_id: i32,
}


pub fn summoner_route_path(platform: &str, game_name: &str, tag_line: &str) -> String {
    format!(
        "/{}/summoners/{}-{}",
        platform,
        game_name,
        tag_line,
    )
}

impl Summoner {
    pub fn to_route_path(&self) -> String {
        summoner_route_path(self.platform.as_region_str(), &self.game_name, &self.tag_line)
    }

    /// Generates a URL-friendly slug.
    pub fn slug(&self) -> String {
        Self::generate_slug(&self.game_name, &self.tag_line)
    }

    /// Generates a slug from the game name and tag line.
    pub fn generate_slug(game_name: &str, tag_line: &str) -> String {
        format!(
            "{}-{}",
            urlencoding::encode(game_name),
            urlencoding::encode(tag_line)
        )
    }

    /// Parses a slug into a game name and tag line.
    pub fn parse_slug(slug: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = slug.split('-').collect();
        if parts.len() != 2 {
            return None;
        }
        let game_name = urlencoding::decode(parts[0]).ok()?.into_owned();
        let tag_line = urlencoding::decode(parts[1]).ok()?.into_owned();
        Some((game_name, tag_line))
    }

    /// Returns the URL of the summoner's profile icon.
    pub fn profile_icon_url(&self) -> String {
        format!(
            "https://raw.communitydragon.org/latest/game/assets/ux/summonericons/profileicon{}.png",
            self.profile_icon_id
        )
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LolSummonerEncounterPage{
    pub id: i32,
    pub count: i64,
    pub game_name: String,
    pub tag_line: String,
    pub platform: String,
    pub profile_icon_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LolSummonerEncounterPageResult{
    pub encounters: Vec<LolSummonerEncounterPage>,
    pub total_pages: i64,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiveGameResult{
    pub game_id:String,
    pub game_length: i64,
    pub game_map: String,
    pub queue_name: String,
    pub participants:Vec<LiveGameResultParticipant>
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiveGameResultParticipant{
    pub puuid: String,
    pub champion_id: i32,
    pub summoner_spell1_id: i32,
    pub summoner_spell2_id: i32,
    pub perk_primary_selection_id : i32,
    pub perk_sub_style_id : i32,
    pub game_name: String,
    pub tag_line: String,
    pub platform: String,
    pub summoner_level : i64,
    pub team_id : i32,
    pub ranked_stats: Option<LiveGameResultParticipantRankedStats>,
    pub champion_stats: Option<LiveGameResultParticipantChampionStats>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiveGameResultParticipantRankedStats{
    pub total_ranked: i32,
    pub total_ranked_wins: i32,
    pub total_ranked_losses: i32,
    pub ranked_win_rate: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiveGameResultParticipantChampionStats{
    pub total_champion_played :i32,
    pub total_champion_wins :i32,
    pub total_champion_losses :i32,
    pub champion_win_rate :f64,
    pub avg_kills: f64,
    pub avg_deaths: f64,
    pub avg_assists: f64,
}

