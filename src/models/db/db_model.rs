use crate::consts::PlatformRoute;
use crate::models::db::DATE_FORMAT;
use crate::models::entities::summoner::Summoner;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use sqlx::types::JsonValue;
use sqlx::FromRow;
use std::str::FromStr;


#[derive(FromRow)]
pub struct LolSummonerChampionResult{
    pub champion_id: i32,
    pub total_matches: i64,
    pub total_wins: i64,
    pub avg_kda: BigDecimal,
    pub avg_kill_participation: BigDecimal,
    pub avg_kills: BigDecimal,
    pub avg_deaths: BigDecimal,
    pub avg_assists: BigDecimal,
    pub avg_gold_earned: BigDecimal,
    pub avg_cs: BigDecimal,
    pub avg_damage_dealt_to_champions: BigDecimal,
    pub avg_damage_taken: BigDecimal,
    pub total_double_kills: i64,
    pub total_triple_kills: i64,
    pub total_quadra_kills: i64,
    pub total_penta_kills: i64,
}


#[derive(FromRow)]
pub struct LolMatchNotUpdated {
    pub id: i32,
    pub match_id: String,
    pub platform: String,
    pub updated: bool,
}

#[derive(FromRow)]
pub struct LolSummonerEncounter{
    pub id: i32,
    pub tag_line: String,
    pub game_name: String,
    pub platform: String,
    pub profile_icon_id: i32,
    pub encounter_count: i64,
    pub total_count : i64,
}


#[derive(FromRow)]
pub struct LolMatchParticipantDetailsQueryResult {
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
    pub kda: Option<BigDecimal>,
    pub kill_participation: Option<BigDecimal>,
    pub damage_dealt_to_champions: i32,
    pub damage_taken: i32,
    pub gold_earned: i32,
    pub wards_placed: i32,
    pub cs: i32,
    pub summoner_spell1_id: Option<i32>,
    pub summoner_spell2_id: Option<i32>,
    pub perk_defense_id: Option<i32>,
    pub perk_flex_id: Option<i32>,
    pub perk_offense_id: Option<i32>,
    pub perk_primary_style_id: Option<i32>,
    pub perk_sub_style_id: Option<i32>,
    pub perk_primary_selection_id: Option<i32>,
    pub perk_primary_selection1_id: Option<i32>,
    pub perk_primary_selection2_id: Option<i32>,
    pub perk_primary_selection3_id: Option<i32>,
    pub perk_sub_selection1_id: Option<i32>,
    pub perk_sub_selection2_id: Option<i32>,
    pub item0_id: Option<i32>,
    pub item1_id: Option<i32>,
    pub item2_id: Option<i32>,
    pub item3_id: Option<i32>,
    pub item4_id: Option<i32>,
    pub item5_id: Option<i32>,
    pub item6_id: Option<i32>,
}


#[derive(FromRow)]
pub struct LolMatchParticipantMatchesQueryAggregateResult {
    #[allow(dead_code)]
    pub total_count: Option<i64>,
    pub total_wins: Option<i64>,
    pub avg_kills: Option<BigDecimal>,
    pub avg_deaths: Option<BigDecimal>,
    pub avg_assists: Option<BigDecimal>,
    pub avg_kda: Option<BigDecimal>,
    pub avg_kill_participation: Option<BigDecimal>,
}


#[derive(FromRow)]
pub struct LolMatchParticipantMatchesQueryResult {
    #[allow(dead_code)]
    pub id: i32,
    pub lol_match_id: i32,
    pub riot_match_id: String,
    pub platform: Option<String>,
    pub champion_id: i32,
    pub summoner_id: i32,
    pub summoner_spell1_id: Option<i32>,
    pub summoner_spell2_id: Option<i32>,
    #[allow(dead_code)]
    pub team_id: i32,
    pub won: bool,
    pub champ_level: i32,
    pub kill_participation: Option<BigDecimal>,
    pub kda: Option<BigDecimal>,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub perk_primary_selection_id: Option<i32>,
    pub perk_sub_style_id: Option<i32>,
    pub item0_id: Option<i32>,
    pub item1_id: Option<i32>,
    pub item2_id: Option<i32>,
    pub item3_id: Option<i32>,
    pub item4_id: Option<i32>,
    pub item5_id: Option<i32>,
    pub item6_id: Option<i32>,
    pub lol_match_queue_id: Option<i32>,
    pub lol_match_match_end: Option<NaiveDateTime>,
    pub lol_match_match_duration: Option<i32>,
}

#[derive(FromRow)]
pub struct LolMatchParticipantMatchesChildQueryResult {
    pub team_id: i32,
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub summoner_name: String,
    pub champion_id: i32,
    pub summoner_tag_line: String,
    pub summoner_platform: String,
}


#[derive(FromRow)]
pub struct LolMatchTimelineQueryResult {
    pub id: i32,
    pub lol_match_id: i32,
    pub summoner_id: i32,
    pub items_event_timeline: JsonValue,
    pub skills_timeline: Vec<i32>,
}


#[derive(sqlx::FromRow, Debug)]
pub struct IdPuuidUpdatedAt {
    pub id: i32,
    pub puuid: String,
    #[sqlx(default)]
    pub updated_at: Option<NaiveDateTime>,
}


#[derive(sqlx::FromRow)]
pub struct SummonerDb {
    pub id: i32,
    pub game_name: String,
    pub tag_line: String,
    pub puuid: String,
    pub platform: String,
    pub updated_at: NaiveDateTime,
    pub summoner_level: i64,
    pub profile_icon_id: i32,
}

impl SummonerDb {
    pub fn map_to_summoner(&self) -> Summoner {
        Summoner {
            id: self.id,
            game_name: self.game_name.clone(),
            tag_line: self.tag_line.clone(),
            puuid: self.puuid.clone(),
            platform: PlatformRoute::from_str(self.platform.as_str()).unwrap(),
            updated_at: self.updated_at.format(DATE_FORMAT).to_string(),
            summoner_level: self.summoner_level,
            profile_icon_id: self.profile_icon_id,
        }
    }
}
