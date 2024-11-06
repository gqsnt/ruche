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

impl Summoner {
    pub fn to_route_path(&self) -> String {
        format!(
            "/{}/summoners/{}-{}",
            self.platform.as_region_str(),
            self.game_name,
            self.tag_line,
        )
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
