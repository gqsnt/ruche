use std::future::Future;
use std::sync::Arc;
use riven::consts::RegionalRoute;
use riven::RiotApi;

pub fn get_riven_match<'a>(api: &'a Arc<RiotApi>, route: RegionalRoute, match_id: &str) -> impl Future<Output=riven::Result<Option<crate::riven_fix::Match>>> + 'a {
    let route_str = route.into();
    let request = api.request(riven::reqwest::Method::GET, route_str, &format!("/lol/match/v5/matches/{}", match_id));
    let future = api.execute_opt::<crate::riven_fix::Match>("match-v5.getMatch", route_str, request);
    #[cfg(feature = "tracing")]
    let future = future.instrument(tracing::info_span!("match-v5.getMatch"));
    future
}


/// Match data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Match {
    /// Match metadata.
    #[serde(rename = "metadata")]
    pub metadata: Metadata,
    /// Match info.
    #[serde(rename = "info")]
    pub info: Info,
}
/// Metadata data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Metadata {
    /// Match data version.
    #[serde(rename = "dataVersion")]
    pub data_version: String,
    /// Match id.
    #[serde(rename = "matchId")]
    pub match_id: String,
    /// A list of participant PUUIDs.
    #[serde(rename = "participants")]
    pub participants: std::vec::Vec<String>,
}
/// Info data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Info {
    /// Refer to indicate if the game ended in termination.
    #[serde(rename = "endOfGameResult")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_of_game_result: Option<String>,
    /// Unix timestamp for when the game is created on the game server (i.e., the loading screen).
    #[serde(rename = "gameCreation")]
    pub game_creation: i64,
    /// Prior to patch 11.20, this field returns the game length in milliseconds calculated from gameEndTimestamp - gameStartTimestamp. Post patch 11.20, this field returns the max timePlayed of any participant in the game in seconds, which makes the behavior of this field consistent with that of match-v4. The best way to handling the change in this field is to treat the value as milliseconds if the gameEndTimestamp field isn't in the response and to treat the value as seconds if gameEndTimestamp is in the response.
    #[serde(rename = "gameDuration")]
    pub game_duration: i64,
    /// Unix timestamp for when match ends on the game server. This timestamp can occasionally be significantly longer than when the match "ends". The most reliable way of determining the timestamp for the end of the match would be to add the max time played of any participant to the gameStartTimestamp. This field was added to match-v5 in patch 11.20 on Oct 5th, 2021.
    #[serde(rename = "gameEndTimestamp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_end_timestamp: Option<i64>,
    #[serde(rename = "gameId")]
    pub game_id: i64,
    /// Refer to the Game Constants documentation.
    #[serde(rename = "gameMode")]
    pub game_mode: riven::consts::GameMode,
    #[serde(rename = "gameName")]
    pub game_name: String,
    /// Unix timestamp for when match starts on the game server.
    #[serde(rename = "gameStartTimestamp")]
    pub game_start_timestamp: i64,
    #[serde(rename = "gameType")]
    ///
    /// Will be `None` if empty string is returned: https://github.com/RiotGames/developer-relations/issues/898
    #[serde(serialize_with = "serialize_empty_string_none")]
    #[serde(deserialize_with = "deserialize_empty_string_none")]
    pub game_type: Option<riven::consts::GameType>,
    /// The first two parts can be used to determine the patch a game was played on.
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    /// Refer to the Game Constants documentation.
    #[serde(rename = "mapId")]
    pub map_id: riven::consts::Map,
    #[serde(rename = "participants")]
    pub participants: std::vec::Vec<Participant>,
    /// Platform where the match was played.
    #[serde(rename = "platformId")]
    pub platform_id: String,
    /// Refer to the Game Constants documentation.
    #[serde(rename = "queueId")]
    pub queue_id: riven::consts::Queue,
    #[serde(rename = "teams")]
    pub teams: std::vec::Vec<Team>,
    /// Tournament code used to generate the match. This field was added to match-v5 in patch 11.13 on June 23rd, 2021.
    #[serde(rename = "tournamentCode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tournament_code: Option<String>,
}
/// Participant data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Participant {
    /// Yellow crossed swords
    #[serde(rename = "allInPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_in_pings: Option<i32>,
    /// Green flag
    #[serde(rename = "assistMePings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assist_me_pings: Option<i32>,
    #[serde(rename = "assists")]
    pub assists: i32,
    #[serde(rename = "baronKills")]
    pub baron_kills: i32,
    #[serde(rename = "bountyLevel")]
    pub bounty_level: i32,
    #[serde(rename = "champExperience")]
    pub champ_experience: i32,
    #[serde(rename = "champLevel")]
    pub champ_level: i32,
    /// Prior to patch 11.4, on Feb 18th, 2021, this field returned invalid championIds. We recommend determining the champion based on the championName field for matches played prior to patch 11.4.
    #[serde(rename = "championId")]
    ///
    /// Instead use [`Self::champion()`] which checks this field then parses [`Self::champion_name`].
    #[deprecated(
        since = "2.5.0",
        note = "Use `Participant.champion()` instead. Riot sometimes returns corrupted data for this field: https://github.com/RiotGames/developer-relations/issues/553"
    )]
    #[serde(serialize_with = "serialize_champion_result")]
    #[serde(deserialize_with = "deserialize_champion_result")]
    pub champion_id: Result<riven::consts::Champion, std::num::TryFromIntError>,
    #[serde(rename = "championName")]
    pub champion_name: String,
    /// Blue generic ping (ALT+click)
    #[serde(rename = "commandPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_pings: Option<i32>,
    /// This field is currently only utilized for Kayn's transformations. (Legal values: 0 - None, 1 - Slayer, 2 - Assassin)
    #[serde(rename = "championTransform")]
    pub champion_transform: i32,
    #[serde(rename = "consumablesPurchased")]
    pub consumables_purchased: i32,
    #[serde(rename = "challenges")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenges: Option<Challenges>,
    #[serde(rename = "damageDealtToBuildings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage_dealt_to_buildings: Option<i32>,
    #[serde(rename = "damageDealtToObjectives")]
    pub damage_dealt_to_objectives: i32,
    #[serde(rename = "damageDealtToTurrets")]
    pub damage_dealt_to_turrets: i32,
    #[serde(rename = "damageSelfMitigated")]
    pub damage_self_mitigated: i32,
    #[serde(rename = "deaths")]
    pub deaths: i32,
    #[serde(rename = "detectorWardsPlaced")]
    pub detector_wards_placed: i32,
    #[serde(rename = "doubleKills")]
    pub double_kills: i32,
    #[serde(rename = "dragonKills")]
    pub dragon_kills: i32,
    #[serde(rename = "eligibleForProgression")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eligible_for_progression: Option<bool>,
    /// Yellow questionmark
    #[serde(rename = "enemyMissingPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enemy_missing_pings: Option<i32>,
    /// Red eyeball
    #[serde(rename = "enemyVisionPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enemy_vision_pings: Option<i32>,
    #[serde(rename = "firstBloodAssist")]
    pub first_blood_assist: bool,
    #[serde(rename = "firstBloodKill")]
    pub first_blood_kill: bool,
    #[serde(rename = "firstTowerAssist")]
    pub first_tower_assist: bool,
    #[serde(rename = "firstTowerKill")]
    pub first_tower_kill: bool,
    /// This is an offshoot of the OneStone challenge. The code checks if a spell with the same instance ID does the final point of damage to at least 2 Champions. It doesn't matter if they're enemies, but you cannot hurt your friends.
    #[serde(rename = "gameEndedInEarlySurrender")]
    pub game_ended_in_early_surrender: bool,
    #[serde(rename = "gameEndedInSurrender")]
    pub game_ended_in_surrender: bool,
    #[serde(rename = "holdPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold_pings: Option<i32>,
    /// Yellow circle with horizontal line
    #[serde(rename = "getBackPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_back_pings: Option<i32>,
    #[serde(rename = "goldEarned")]
    pub gold_earned: i32,
    #[serde(rename = "goldSpent")]
    pub gold_spent: i32,
    /// Both individualPosition and teamPosition are computed by the game server and are different versions of the most likely position played by a player. The individualPosition is the best guess for which position the player actually played in isolation of anything else. The teamPosition is the best guess for which position the player actually played if we add the constraint that each team must have one top player, one jungle, one middle, etc. Generally the recommendation is to use the teamPosition field over the individualPosition field.
    #[serde(rename = "individualPosition")]
    pub individual_position: String,
    #[serde(rename = "inhibitorKills")]
    pub inhibitor_kills: i32,
    #[serde(rename = "inhibitorTakedowns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inhibitor_takedowns: Option<i32>,
    #[serde(rename = "inhibitorsLost")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inhibitors_lost: Option<i32>,
    #[serde(rename = "item0")]
    pub item0: i32,
    #[serde(rename = "item1")]
    pub item1: i32,
    #[serde(rename = "item2")]
    pub item2: i32,
    #[serde(rename = "item3")]
    pub item3: i32,
    #[serde(rename = "item4")]
    pub item4: i32,
    #[serde(rename = "item5")]
    pub item5: i32,
    #[serde(rename = "item6")]
    pub item6: i32,
    #[serde(rename = "itemsPurchased")]
    pub items_purchased: i32,
    #[serde(rename = "killingSprees")]
    pub killing_sprees: i32,
    #[serde(rename = "kills")]
    pub kills: i32,
    #[serde(rename = "lane")]
    pub lane: String,
    #[serde(rename = "largestCriticalStrike")]
    pub largest_critical_strike: i32,
    #[serde(rename = "largestKillingSpree")]
    pub largest_killing_spree: i32,
    #[serde(rename = "largestMultiKill")]
    pub largest_multi_kill: i32,
    #[serde(rename = "longestTimeSpentLiving")]
    pub longest_time_spent_living: i32,
    #[serde(rename = "magicDamageDealt")]
    pub magic_damage_dealt: i32,
    #[serde(rename = "magicDamageDealtToChampions")]
    pub magic_damage_dealt_to_champions: i32,
    #[serde(rename = "magicDamageTaken")]
    pub magic_damage_taken: i32,
    #[serde(rename = "missions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missions: Option<Missions>,
    /// neutralMinionsKilled = mNeutralMinionsKilled, which is incremented on kills of kPet and kJungleMonster
    #[serde(rename = "neutralMinionsKilled")]
    pub neutral_minions_killed: i32,
    /// Green ward
    #[serde(rename = "needVisionPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub need_vision_pings: Option<i32>,
    #[serde(rename = "nexusKills")]
    pub nexus_kills: i32,
    #[serde(rename = "nexusTakedowns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nexus_takedowns: Option<i32>,
    #[serde(rename = "nexusLost")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nexus_lost: Option<i32>,
    #[serde(rename = "objectivesStolen")]
    pub objectives_stolen: i32,
    #[serde(rename = "objectivesStolenAssists")]
    pub objectives_stolen_assists: i32,
    /// Blue arrow pointing at ground
    #[serde(rename = "onMyWayPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_my_way_pings: Option<i32>,
    #[serde(rename = "participantId")]
    pub participant_id: i32,
    #[serde(rename = "playerScore0")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score0: Option<i32>,
    #[serde(rename = "playerScore1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score1: Option<i32>,
    #[serde(rename = "playerScore2")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score2: Option<i32>,
    #[serde(rename = "playerScore3")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score3: Option<i32>,
    #[serde(rename = "playerScore4")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score4: Option<i32>,
    #[serde(rename = "playerScore5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score5: Option<i32>,
    #[serde(rename = "playerScore6")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score6: Option<i32>,
    #[serde(rename = "playerScore7")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score7: Option<i32>,
    #[serde(rename = "playerScore8")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score8: Option<i32>,
    #[serde(rename = "playerScore9")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score9: Option<i32>,
    #[serde(rename = "playerScore10")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score10: Option<i32>,
    #[serde(rename = "playerScore11")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score11: Option<i32>,
    #[serde(rename = "pentaKills")]
    pub penta_kills: i32,
    #[serde(rename = "perks")]
    pub perks: Perks,
    #[serde(rename = "physicalDamageDealt")]
    pub physical_damage_dealt: i32,
    #[serde(rename = "physicalDamageDealtToChampions")]
    pub physical_damage_dealt_to_champions: i32,
    #[serde(rename = "physicalDamageTaken")]
    pub physical_damage_taken: i32,
    #[serde(rename = "placement")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement: Option<i32>,
    #[serde(rename = "playerAugment1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_augment1: Option<i32>,
    #[serde(rename = "playerAugment2")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_augment2: Option<i32>,
    #[serde(rename = "playerAugment3")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_augment3: Option<i32>,
    #[serde(rename = "playerAugment4")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_augment4: Option<i32>,
    #[serde(rename = "playerSubteamId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_subteam_id: Option<i32>,
    /// Green minion
    #[serde(rename = "pushPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_pings: Option<i32>,
    #[serde(rename = "profileIcon")]
    pub profile_icon: i32,
    #[serde(rename = "puuid")]
    pub puuid: String,
    #[serde(rename = "quadraKills")]
    pub quadra_kills: i32,
    #[serde(rename = "riotIdGameName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub riot_id_game_name: Option<String>,
    #[serde(rename = "riotIdName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub riot_id_name: Option<String>,
    #[serde(rename = "riotIdTagline")]
    pub riot_id_tagline: String,
    #[serde(rename = "role")]
    pub role: String,
    #[serde(rename = "sightWardsBoughtInGame")]
    pub sight_wards_bought_in_game: i32,
    #[serde(rename = "spell1Casts")]
    pub spell1_casts: i32,
    #[serde(rename = "spell2Casts")]
    pub spell2_casts: i32,
    #[serde(rename = "spell3Casts")]
    pub spell3_casts: i32,
    #[serde(rename = "spell4Casts")]
    pub spell4_casts: i32,
    #[serde(rename = "subteamPlacement")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subteam_placement: Option<i32>,
    #[serde(rename = "summoner1Casts")]
    pub summoner1_casts: i32,
    #[serde(rename = "summoner1Id")]
    pub summoner1_id: i32,
    #[serde(rename = "summoner2Casts")]
    pub summoner2_casts: i32,
    #[serde(rename = "summoner2Id")]
    pub summoner2_id: i32,
    #[serde(rename = "summonerId")]
    pub summoner_id: String,
    #[serde(rename = "summonerLevel")]
    pub summoner_level: i32,
    #[serde(rename = "summonerName")]
    pub summoner_name: String,
    #[serde(rename = "teamEarlySurrendered")]
    pub team_early_surrendered: bool,
    #[serde(rename = "teamId")]
    pub team_id: riven::consts::Team,
    /// Both individualPosition and teamPosition are computed by the game server and are different versions of the most likely position played by a player. The individualPosition is the best guess for which position the player actually played in isolation of anything else. The teamPosition is the best guess for which position the player actually played if we add the constraint that each team must have one top player, one jungle, one middle, etc. Generally the recommendation is to use the teamPosition field over the individualPosition field.
    #[serde(rename = "teamPosition")]
    pub team_position: String,
    #[serde(rename = "timeCCingOthers")]
    pub time_c_cing_others: i32,
    #[serde(rename = "timePlayed")]
    pub time_played: i32,
    #[serde(rename = "totalAllyJungleMinionsKilled")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_ally_jungle_minions_killed: Option<i32>,
    #[serde(rename = "totalDamageDealt")]
    pub total_damage_dealt: i32,
    #[serde(rename = "totalDamageDealtToChampions")]
    pub total_damage_dealt_to_champions: i32,
    #[serde(rename = "totalDamageShieldedOnTeammates")]
    pub total_damage_shielded_on_teammates: i32,
    #[serde(rename = "totalDamageTaken")]
    pub total_damage_taken: i32,
    #[serde(rename = "totalEnemyJungleMinionsKilled")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_enemy_jungle_minions_killed: Option<i32>,
    /// Whenever positive health is applied (which translates to all heals in the game but not things like regeneration), totalHeal is incremented by the amount of health received. This includes healing enemies, jungle monsters, yourself, etc
    #[serde(rename = "totalHeal")]
    pub total_heal: i32,
    /// Whenever positive health is applied (which translates to all heals in the game but not things like regeneration), totalHealsOnTeammates is incremented by the amount of health received.  This is post modified, so if you heal someone missing 5 health for 100 you will get +5 totalHealsOnTeammates
    #[serde(rename = "totalHealsOnTeammates")]
    pub total_heals_on_teammates: i32,
    /// totalMillionsKilled = mMinionsKilled, which is only incremented on kills of kTeamMinion, kMeleeLaneMinion, kSuperLaneMinion, kRangedLaneMinion and kSiegeLaneMinion
    #[serde(rename = "totalMinionsKilled")]
    pub total_minions_killed: i32,
    #[serde(rename = "totalTimeCCDealt")]
    pub total_time_cc_dealt: i32,
    #[serde(rename = "totalTimeSpentDead")]
    pub total_time_spent_dead: i32,
    #[serde(rename = "totalUnitsHealed")]
    pub total_units_healed: i32,
    #[serde(rename = "tripleKills")]
    pub triple_kills: i32,
    #[serde(rename = "trueDamageDealt")]
    pub true_damage_dealt: i32,
    #[serde(rename = "trueDamageDealtToChampions")]
    pub true_damage_dealt_to_champions: i32,
    #[serde(rename = "trueDamageTaken")]
    pub true_damage_taken: i32,
    #[serde(rename = "turretKills")]
    pub turret_kills: i32,
    #[serde(rename = "turretTakedowns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turret_takedowns: Option<i32>,
    #[serde(rename = "turretsLost")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turrets_lost: Option<i32>,
    #[serde(rename = "unrealKills")]
    pub unreal_kills: i32,
    #[serde(rename = "visionScore")]
    pub vision_score: i32,
    #[serde(rename = "visionClearedPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision_cleared_pings: Option<i32>,
    #[serde(rename = "visionWardsBoughtInGame")]
    pub vision_wards_bought_in_game: i32,
    #[serde(rename = "wardsKilled")]
    pub wards_killed: i32,
    #[serde(rename = "wardsPlaced")]
    pub wards_placed: i32,
    #[serde(rename = "win")]
    pub win: bool,
    #[serde(rename = "baitPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bait_pings: Option<i32>,
    /// https://github.com/RiotGames/developer-relations/issues/870
    #[serde(rename = "dangerPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danger_pings: Option<i32>,
    /// https://github.com/RiotGames/developer-relations/issues/814
    #[serde(rename = "basicPings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic_pings: Option<i32>,
    #[serde(rename = "playerAugment5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_augment5: Option<i32>,
    #[serde(rename = "playerAugment6")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_augment6: Option<i32>,
}
/// Challenges data object.
/// # Description
/// Challenges DTO
///
/// Note: This struct is automatically generated
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Challenges {
    #[serde(rename = "12AssistStreakCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x12_assist_streak_count: Option<i32>,
    #[serde(rename = "baronBuffGoldAdvantageOverThreshold")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baron_buff_gold_advantage_over_threshold: Option<f64>,
    #[serde(rename = "controlWardTimeCoverageInRiverOrEnemyHalf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_ward_time_coverage_in_river_or_enemy_half: Option<f64>,
    #[serde(rename = "earliestBaron")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earliest_baron: Option<f64>,
    #[serde(rename = "earliestDragonTakedown")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earliest_dragon_takedown: Option<f64>,
    #[serde(rename = "earliestElderDragon")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earliest_elder_dragon: Option<f64>,
    #[serde(rename = "earlyLaningPhaseGoldExpAdvantage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub early_laning_phase_gold_exp_advantage: Option<f64>,
    #[serde(rename = "fasterSupportQuestCompletion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faster_support_quest_completion: Option<f64>,
    #[serde(rename = "fastestLegendary")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fastest_legendary: Option<f64>,
    #[serde(rename = "hadAfkTeammate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub had_afk_teammate: Option<f64>,
    #[serde(rename = "highestChampionDamage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_champion_damage: Option<f64>,
    #[serde(rename = "highestCrowdControlScore")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_crowd_control_score: Option<f64>,
    #[serde(rename = "highestWardKills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_ward_kills: Option<f64>,
    #[serde(rename = "junglerKillsEarlyJungle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jungler_kills_early_jungle: Option<f64>,
    #[serde(rename = "killsOnLanersEarlyJungleAsJungler")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kills_on_laners_early_jungle_as_jungler: Option<f64>,
    #[serde(rename = "laningPhaseGoldExpAdvantage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub laning_phase_gold_exp_advantage: Option<f64>,
    #[serde(rename = "legendaryCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legendary_count: Option<f64>,
    #[serde(rename = "maxCsAdvantageOnLaneOpponent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cs_advantage_on_lane_opponent: Option<f64>,
    #[serde(rename = "maxLevelLeadLaneOpponent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_level_lead_lane_opponent: Option<f64>,
    #[serde(rename = "mostWardsDestroyedOneSweeper")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub most_wards_destroyed_one_sweeper: Option<f64>,
    #[serde(rename = "mythicItemUsed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mythic_item_used: Option<f64>,
    #[serde(rename = "playedChampSelectPosition")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub played_champ_select_position: Option<f64>,
    #[serde(rename = "soloTurretsLategame")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solo_turrets_lategame: Option<f64>,
    #[serde(rename = "takedownsFirst25Minutes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takedowns_first25_minutes: Option<f64>,
    #[serde(rename = "teleportTakedowns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teleport_takedowns: Option<f64>,
    #[serde(rename = "thirdInhibitorDestroyedTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub third_inhibitor_destroyed_time: Option<f64>,
    #[serde(rename = "threeWardsOneSweeperCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub three_wards_one_sweeper_count: Option<f64>,
    #[serde(rename = "visionScoreAdvantageLaneOpponent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision_score_advantage_lane_opponent: Option<f64>,
    #[serde(rename = "InfernalScalePickup")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infernal_scale_pickup: Option<f64>,
    #[serde(rename = "fistBumpParticipation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fist_bump_participation: Option<i32>,
    #[serde(rename = "voidMonsterKill")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub void_monster_kill: Option<i32>,
    #[serde(rename = "abilityUses")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability_uses: Option<i32>,
    #[serde(rename = "acesBefore15Minutes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aces_before15_minutes: Option<i32>,
    #[serde(rename = "alliedJungleMonsterKills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allied_jungle_monster_kills: Option<f64>,
    #[serde(rename = "baronTakedowns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baron_takedowns: Option<i32>,
    #[serde(rename = "blastConeOppositeOpponentCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blast_cone_opposite_opponent_count: Option<i32>,
    #[serde(rename = "bountyGold")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounty_gold: Option<f32>,
    #[serde(rename = "buffsStolen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffs_stolen: Option<i32>,
    #[serde(rename = "completeSupportQuestInTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete_support_quest_in_time: Option<i32>,
    #[serde(rename = "controlWardsPlaced")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_wards_placed: Option<i32>,
    #[serde(rename = "damagePerMinute")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage_per_minute: Option<f64>,
    #[serde(rename = "damageTakenOnTeamPercentage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage_taken_on_team_percentage: Option<f64>,
    #[serde(rename = "dancedWithRiftHerald")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danced_with_rift_herald: Option<i32>,
    #[serde(rename = "deathsByEnemyChamps")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deaths_by_enemy_champs: Option<i32>,
    #[serde(rename = "dodgeSkillShotsSmallWindow")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dodge_skill_shots_small_window: Option<i32>,
    #[serde(rename = "doubleAces")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_aces: Option<i32>,
    #[serde(rename = "dragonTakedowns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dragon_takedowns: Option<i32>,
    #[serde(rename = "legendaryItemUsed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legendary_item_used: Option<std::vec::Vec<i32>>,
    #[serde(rename = "effectiveHealAndShielding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_heal_and_shielding: Option<f32>,
    #[serde(rename = "elderDragonKillsWithOpposingSoul")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elder_dragon_kills_with_opposing_soul: Option<i32>,
    #[serde(rename = "elderDragonMultikills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elder_dragon_multikills: Option<i32>,
    #[serde(rename = "enemyChampionImmobilizations")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enemy_champion_immobilizations: Option<i32>,
    #[serde(rename = "enemyJungleMonsterKills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enemy_jungle_monster_kills: Option<f64>,
    #[serde(rename = "epicMonsterKillsNearEnemyJungler")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epic_monster_kills_near_enemy_jungler: Option<i32>,
    #[serde(rename = "epicMonsterKillsWithin30SecondsOfSpawn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epic_monster_kills_within30_seconds_of_spawn: Option<i32>,
    #[serde(rename = "epicMonsterSteals")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epic_monster_steals: Option<i32>,
    #[serde(rename = "epicMonsterStolenWithoutSmite")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epic_monster_stolen_without_smite: Option<i32>,
    #[serde(rename = "firstTurretKilled")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_turret_killed: Option<f64>,
    #[serde(rename = "firstTurretKilledTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_turret_killed_time: Option<f32>,
    #[serde(rename = "flawlessAces")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flawless_aces: Option<i32>,
    #[serde(rename = "fullTeamTakedown")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_team_takedown: Option<i32>,
    #[serde(rename = "gameLength")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_length: Option<f64>,
    #[serde(rename = "getTakedownsInAllLanesEarlyJungleAsLaner")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_takedowns_in_all_lanes_early_jungle_as_laner: Option<i32>,
    #[serde(rename = "goldPerMinute")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gold_per_minute: Option<f64>,
    #[serde(rename = "hadOpenNexus")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub had_open_nexus: Option<i32>,
    #[serde(rename = "immobilizeAndKillWithAlly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immobilize_and_kill_with_ally: Option<i32>,
    #[serde(rename = "initialBuffCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_buff_count: Option<i32>,
    #[serde(rename = "initialCrabCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_crab_count: Option<i32>,
    #[serde(rename = "jungleCsBefore10Minutes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jungle_cs_before10_minutes: Option<f64>,
    #[serde(rename = "junglerTakedownsNearDamagedEpicMonster")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jungler_takedowns_near_damaged_epic_monster: Option<i32>,
    #[serde(rename = "kda")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kda: Option<f64>,
    #[serde(rename = "killAfterHiddenWithAlly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kill_after_hidden_with_ally: Option<i32>,
    #[serde(rename = "killedChampTookFullTeamDamageSurvived")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub killed_champ_took_full_team_damage_survived: Option<i32>,
    #[serde(rename = "killingSprees")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub killing_sprees: Option<i32>,
    #[serde(rename = "killParticipation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kill_participation: Option<f64>,
    #[serde(rename = "killsNearEnemyTurret")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kills_near_enemy_turret: Option<i32>,
    #[serde(rename = "killsOnOtherLanesEarlyJungleAsLaner")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kills_on_other_lanes_early_jungle_as_laner: Option<i32>,
    #[serde(rename = "killsOnRecentlyHealedByAramPack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kills_on_recently_healed_by_aram_pack: Option<i32>,
    #[serde(rename = "killsUnderOwnTurret")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kills_under_own_turret: Option<i32>,
    #[serde(rename = "killsWithHelpFromEpicMonster")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kills_with_help_from_epic_monster: Option<i32>,
    #[serde(rename = "knockEnemyIntoTeamAndKill")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub knock_enemy_into_team_and_kill: Option<i32>,
    #[serde(rename = "kTurretsDestroyedBeforePlatesFall")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub k_turrets_destroyed_before_plates_fall: Option<i32>,
    #[serde(rename = "landSkillShotsEarlyGame")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub land_skill_shots_early_game: Option<i32>,
    #[serde(rename = "laneMinionsFirst10Minutes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lane_minions_first10_minutes: Option<i32>,
    #[serde(rename = "lostAnInhibitor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lost_an_inhibitor: Option<i32>,
    #[serde(rename = "maxKillDeficit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_kill_deficit: Option<i32>,
    #[serde(rename = "mejaisFullStackInTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mejais_full_stack_in_time: Option<i32>,
    #[serde(rename = "moreEnemyJungleThanOpponent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more_enemy_jungle_than_opponent: Option<f64>,
    /// This is an offshoot of the OneStone challenge. The code checks if a spell with the same instance ID does the final point of damage to at least 2 Champions. It doesn't matter if they're enemies, but you cannot hurt your friends.
    #[serde(rename = "multiKillOneSpell")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_kill_one_spell: Option<i32>,
    #[serde(rename = "multikills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multikills: Option<i32>,
    #[serde(rename = "multikillsAfterAggressiveFlash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multikills_after_aggressive_flash: Option<i32>,
    #[serde(rename = "multiTurretRiftHeraldCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_turret_rift_herald_count: Option<i32>,
    #[serde(rename = "outerTurretExecutesBefore10Minutes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outer_turret_executes_before10_minutes: Option<i32>,
    #[serde(rename = "outnumberedKills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outnumbered_kills: Option<i32>,
    #[serde(rename = "outnumberedNexusKill")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outnumbered_nexus_kill: Option<i32>,
    #[serde(rename = "perfectDragonSoulsTaken")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perfect_dragon_souls_taken: Option<i32>,
    #[serde(rename = "perfectGame")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perfect_game: Option<i32>,
    #[serde(rename = "pickKillWithAlly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pick_kill_with_ally: Option<i32>,
    #[serde(rename = "poroExplosions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poro_explosions: Option<i32>,
    #[serde(rename = "quickCleanse")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quick_cleanse: Option<i32>,
    #[serde(rename = "quickFirstTurret")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quick_first_turret: Option<i32>,
    #[serde(rename = "quickSoloKills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quick_solo_kills: Option<i32>,
    #[serde(rename = "riftHeraldTakedowns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rift_herald_takedowns: Option<i32>,
    #[serde(rename = "saveAllyFromDeath")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_ally_from_death: Option<i32>,
    #[serde(rename = "scuttleCrabKills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scuttle_crab_kills: Option<i32>,
    #[serde(rename = "shortestTimeToAceFromFirstTakedown")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortest_time_to_ace_from_first_takedown: Option<f32>,
    #[serde(rename = "skillshotsDodged")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skillshots_dodged: Option<i32>,
    #[serde(rename = "skillshotsHit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skillshots_hit: Option<i32>,
    #[serde(rename = "snowballsHit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snowballs_hit: Option<i32>,
    #[serde(rename = "soloBaronKills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solo_baron_kills: Option<i32>,
    #[serde(rename = "soloKills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solo_kills: Option<i32>,
    #[serde(rename = "stealthWardsPlaced")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stealth_wards_placed: Option<i32>,
    #[serde(rename = "survivedSingleDigitHpCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub survived_single_digit_hp_count: Option<i32>,
    #[serde(rename = "survivedThreeImmobilizesInFight")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub survived_three_immobilizes_in_fight: Option<i32>,
    #[serde(rename = "takedownOnFirstTurret")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takedown_on_first_turret: Option<i32>,
    #[serde(rename = "takedowns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takedowns: Option<i32>,
    #[serde(rename = "takedownsAfterGainingLevelAdvantage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takedowns_after_gaining_level_advantage: Option<i32>,
    #[serde(rename = "takedownsBeforeJungleMinionSpawn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takedowns_before_jungle_minion_spawn: Option<i32>,
    #[serde(rename = "takedownsFirstXMinutes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takedowns_first_x_minutes: Option<i32>,
    #[serde(rename = "takedownsInAlcove")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takedowns_in_alcove: Option<i32>,
    #[serde(rename = "takedownsInEnemyFountain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub takedowns_in_enemy_fountain: Option<i32>,
    #[serde(rename = "teamBaronKills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_baron_kills: Option<i32>,
    #[serde(rename = "teamDamagePercentage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_damage_percentage: Option<f64>,
    #[serde(rename = "teamElderDragonKills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_elder_dragon_kills: Option<i32>,
    #[serde(rename = "teamRiftHeraldKills")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_rift_herald_kills: Option<i32>,
    #[serde(rename = "tookLargeDamageSurvived")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub took_large_damage_survived: Option<i32>,
    #[serde(rename = "turretPlatesTaken")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turret_plates_taken: Option<i32>,
    /// Any player who damages a tower that is destroyed within 30 seconds of a Rift Herald charge will receive credit. A player who does not damage the tower will not receive credit.
    #[serde(rename = "turretsTakenWithRiftHerald")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turrets_taken_with_rift_herald: Option<i32>,
    #[serde(rename = "turretTakedowns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turret_takedowns: Option<i32>,
    #[serde(rename = "twentyMinionsIn3SecondsCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twenty_minions_in3_seconds_count: Option<i32>,
    #[serde(rename = "twoWardsOneSweeperCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two_wards_one_sweeper_count: Option<i32>,
    #[serde(rename = "unseenRecalls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unseen_recalls: Option<i32>,
    #[serde(rename = "visionScorePerMinute")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision_score_per_minute: Option<f64>,
    #[serde(rename = "wardsGuarded")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wards_guarded: Option<i32>,
    #[serde(rename = "wardTakedowns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ward_takedowns: Option<i32>,
    #[serde(rename = "wardTakedownsBefore20M")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ward_takedowns_before20_m: Option<i32>,
    #[serde(rename = "SWARM_DefeatAatrox")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_defeat_aatrox: Option<i32>,
    #[serde(rename = "SWARM_DefeatBriar")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_defeat_briar: Option<i32>,
    #[serde(rename = "SWARM_DefeatMiniBosses")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_defeat_mini_bosses: Option<i32>,
    #[serde(rename = "SWARM_EvolveWeapon")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_evolve_weapon: Option<i32>,
    #[serde(rename = "SWARM_Have3Passives")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_have3_passives: Option<i32>,
    #[serde(rename = "SWARM_KillEnemy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_kill_enemy: Option<i32>,
    #[serde(rename = "SWARM_PickupGold")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_pickup_gold: Option<i32>,
    #[serde(rename = "SWARM_ReachLevel50")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_reach_level50: Option<i32>,
    #[serde(rename = "SWARM_Survive15Min")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_survive15_min: Option<i32>,
    #[serde(rename = "SWARM_WinWith5EvolvedWeapons")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swarm_win_with5_evolved_weapons: Option<i32>,
}
/// Missions data object.
/// # Description
/// Missions DTO
///
/// Note: This struct is automatically generated
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Missions {
    #[serde(rename = "playerScore0")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score0: Option<i32>,
    #[serde(rename = "playerScore1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score1: Option<i32>,
    #[serde(rename = "playerScore2")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score2: Option<i32>,
    #[serde(rename = "playerScore3")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score3: Option<i32>,
    #[serde(rename = "playerScore4")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score4: Option<i32>,
    #[serde(rename = "playerScore5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score5: Option<i32>,
    #[serde(rename = "playerScore6")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score6: Option<i32>,
    #[serde(rename = "playerScore7")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score7: Option<i32>,
    #[serde(rename = "playerScore8")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score8: Option<i32>,
    #[serde(rename = "playerScore9")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score9: Option<i32>,
    #[serde(rename = "playerScore10")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score10: Option<i32>,
    #[serde(rename = "playerScore11")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_score11: Option<i32>,
}
/// Perks data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Perks {
    #[serde(rename = "statPerks")]
    pub stat_perks: PerkStats,
    #[serde(rename = "styles")]
    pub styles: std::vec::Vec<PerkStyle>,
}
/// PerkStats data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct PerkStats {
    #[serde(rename = "defense")]
    pub defense: i32,
    #[serde(rename = "flex")]
    pub flex: i32,
    #[serde(rename = "offense")]
    pub offense: i32,
}
/// PerkStyle data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct PerkStyle {
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "selections")]
    pub selections: std::vec::Vec<PerkStyleSelection>,
    #[serde(rename = "style")]
    pub style: i32,
}
/// PerkStyleSelection data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct PerkStyleSelection {
    #[serde(rename = "perk")]
    pub perk: i32,
    #[serde(rename = "var1")]
    pub var1: i32,
    #[serde(rename = "var2")]
    pub var2: i32,
    #[serde(rename = "var3")]
    pub var3: i32,
}
/// Team data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Team {
    #[serde(rename = "bans")]
    pub bans: std::vec::Vec<Ban>,
    #[serde(rename = "objectives")]
    pub objectives: Objectives,
    #[serde(rename = "teamId")]
    pub team_id: riven::consts::Team,
    #[serde(rename = "win")]
    pub win: bool,
}
/// Ban data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Ban {
    #[serde(rename = "championId")]
    pub champion_id: riven::consts::Champion,
    #[serde(rename = "pickTurn")]
    pub pick_turn: i32,
}
/// Objectives data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Objectives {
    #[serde(rename = "baron")]
    pub baron: Objective,
    #[serde(rename = "champion")]
    pub champion: Objective,
    #[serde(rename = "dragon")]
    pub dragon: Objective,
    #[serde(rename = "horde")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horde: Option<Objective>,
    #[serde(rename = "inhibitor")]
    pub inhibitor: Objective,
    #[serde(rename = "riftHerald")]
    pub rift_herald: Objective,
    #[serde(rename = "tower")]
    pub tower: Objective,
}
/// Objective data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Objective {
    #[serde(rename = "first")]
    pub first: bool,
    #[serde(rename = "kills")]
    pub kills: i32,
}
/// Timeline data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Timeline {
    /// Match metadata.
    #[serde(rename = "metadata")]
    pub metadata: MetadataTimeLine,
    /// Match info.
    #[serde(rename = "info")]
    pub info: InfoTimeLine,
}
/// MetadataTimeLine data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct MetadataTimeLine {
    /// Match data version.
    #[serde(rename = "dataVersion")]
    pub data_version: String,
    /// Match id.
    #[serde(rename = "matchId")]
    pub match_id: String,
    /// A list of participant PUUIDs.
    #[serde(rename = "participants")]
    pub participants: std::vec::Vec<String>,
}
/// InfoTimeLine data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct InfoTimeLine {
    /// Refer to indicate if the game ended in termination.
    #[serde(rename = "endOfGameResult")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_of_game_result: Option<String>,
    #[serde(rename = "frameInterval")]
    pub frame_interval: i64,
    #[serde(rename = "gameId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<i64>,
    #[serde(rename = "participants")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participants: Option<std::vec::Vec<ParticipantTimeLine>>,
    #[serde(rename = "frames")]
    pub frames: std::vec::Vec<FramesTimeLine>,
}
/// ParticipantTimeLine data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct ParticipantTimeLine {
    #[serde(rename = "participantId")]
    pub participant_id: i32,
    #[serde(rename = "puuid")]
    pub puuid: String,
}
/// FramesTimeLine data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct FramesTimeLine {
    #[serde(rename = "events")]
    pub events: std::vec::Vec<EventsTimeLine>,
    #[serde(rename = "participantFrames")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_frames: Option<std::collections::HashMap<i32, ParticipantFrame>>,
    #[serde(rename = "timestamp")]
    pub timestamp: i32,
}
/// EventsTimeLine data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct EventsTimeLine {
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "realTimestamp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub real_timestamp: Option<i64>,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "itemId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<i32>,
    #[serde(rename = "participantId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<i32>,
    #[serde(rename = "levelUpType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level_up_type: Option<String>,
    #[serde(rename = "skillSlot")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_slot: Option<i32>,
    #[serde(rename = "creatorId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<i32>,
    #[serde(rename = "wardType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ward_type: Option<String>,
    #[serde(rename = "level")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i32>,
    #[serde(rename = "assistingParticipantIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assisting_participant_ids: Option<std::vec::Vec<i32>>,
    #[serde(rename = "bounty")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounty: Option<f32>,
    #[serde(rename = "killStreakLength")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kill_streak_length: Option<i32>,
    #[serde(rename = "killerId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub killer_id: Option<i32>,
    #[serde(rename = "position")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(rename = "victimDamageDealt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub victim_damage_dealt: Option<std::vec::Vec<MatchTimelineVictimDamage>>,
    #[serde(rename = "victimDamageReceived")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub victim_damage_received: Option<std::vec::Vec<MatchTimelineVictimDamage>>,
    #[serde(rename = "victimId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub victim_id: Option<i32>,
    #[serde(rename = "killType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kill_type: Option<String>,
    #[serde(rename = "laneType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lane_type: Option<String>,
    #[serde(rename = "teamId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<riven::consts::Team>,
    #[serde(rename = "multiKillLength")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_kill_length: Option<i32>,
    #[serde(rename = "killerTeamId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub killer_team_id: Option<riven::consts::Team>,
    #[serde(rename = "monsterType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_type: Option<String>,
    #[serde(rename = "monsterSubType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_sub_type: Option<String>,
    #[serde(rename = "buildingType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_type: Option<String>,
    #[serde(rename = "towerType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tower_type: Option<String>,
    #[serde(rename = "afterId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<i32>,
    #[serde(rename = "beforeId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<i32>,
    #[serde(rename = "goldGain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gold_gain: Option<i32>,
    #[serde(rename = "gameId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<i64>,
    #[serde(rename = "winningTeam")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winning_team: Option<i32>,
    #[serde(rename = "transformType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform_type: Option<String>,
    #[serde(rename = "name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "shutdownBounty")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shutdown_bounty: Option<i32>,
    #[serde(rename = "actualStartTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_start_time: Option<i64>,
}
/// ParticipantFrames data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct ParticipantFrames {
    /// Key value mapping for each participant
    #[serde(rename = "1-9")]
    pub x1_9: ParticipantFrame,
}
/// ParticipantFrame data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct ParticipantFrame {
    #[serde(rename = "championStats")]
    pub champion_stats: ChampionStats,
    #[serde(rename = "currentGold")]
    pub current_gold: i32,
    #[serde(rename = "damageStats")]
    pub damage_stats: DamageStats,
    #[serde(rename = "goldPerSecond")]
    pub gold_per_second: i32,
    #[serde(rename = "jungleMinionsKilled")]
    pub jungle_minions_killed: i32,
    #[serde(rename = "level")]
    pub level: i32,
    #[serde(rename = "minionsKilled")]
    pub minions_killed: i32,
    #[serde(rename = "participantId")]
    pub participant_id: i32,
    #[serde(rename = "position")]
    pub position: Position,
    #[serde(rename = "timeEnemySpentControlled")]
    pub time_enemy_spent_controlled: i32,
    #[serde(rename = "totalGold")]
    pub total_gold: i32,
    #[serde(rename = "xp")]
    pub xp: i32,
}
/// ChampionStats data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct ChampionStats {
    #[serde(rename = "abilityHaste")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability_haste: Option<i32>,
    #[serde(rename = "abilityPower")]
    pub ability_power: i32,
    #[serde(rename = "armor")]
    pub armor: i32,
    #[serde(rename = "armorPen")]
    pub armor_pen: i32,
    #[serde(rename = "armorPenPercent")]
    pub armor_pen_percent: i32,
    #[serde(rename = "attackDamage")]
    pub attack_damage: i32,
    #[serde(rename = "attackSpeed")]
    pub attack_speed: i32,
    #[serde(rename = "bonusArmorPenPercent")]
    pub bonus_armor_pen_percent: i32,
    #[serde(rename = "bonusMagicPenPercent")]
    pub bonus_magic_pen_percent: i32,
    #[serde(rename = "ccReduction")]
    pub cc_reduction: i32,
    #[serde(rename = "cooldownReduction")]
    pub cooldown_reduction: i32,
    #[serde(rename = "health")]
    pub health: i32,
    #[serde(rename = "healthMax")]
    pub health_max: i32,
    #[serde(rename = "healthRegen")]
    pub health_regen: i32,
    #[serde(rename = "lifesteal")]
    pub lifesteal: i32,
    #[serde(rename = "magicPen")]
    pub magic_pen: i32,
    #[serde(rename = "magicPenPercent")]
    pub magic_pen_percent: i32,
    #[serde(rename = "magicResist")]
    pub magic_resist: i32,
    #[serde(rename = "movementSpeed")]
    pub movement_speed: i32,
    #[serde(rename = "omnivamp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omnivamp: Option<i32>,
    #[serde(rename = "physicalVamp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_vamp: Option<i32>,
    #[serde(rename = "power")]
    pub power: i32,
    #[serde(rename = "powerMax")]
    pub power_max: i32,
    #[serde(rename = "powerRegen")]
    pub power_regen: i32,
    #[serde(rename = "spellVamp")]
    pub spell_vamp: i32,
}
/// DamageStats data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct DamageStats {
    #[serde(rename = "magicDamageDone")]
    pub magic_damage_done: i32,
    #[serde(rename = "magicDamageDoneToChampions")]
    pub magic_damage_done_to_champions: i32,
    #[serde(rename = "magicDamageTaken")]
    pub magic_damage_taken: i32,
    #[serde(rename = "physicalDamageDone")]
    pub physical_damage_done: i32,
    #[serde(rename = "physicalDamageDoneToChampions")]
    pub physical_damage_done_to_champions: i32,
    #[serde(rename = "physicalDamageTaken")]
    pub physical_damage_taken: i32,
    #[serde(rename = "totalDamageDone")]
    pub total_damage_done: i32,
    #[serde(rename = "totalDamageDoneToChampions")]
    pub total_damage_done_to_champions: i32,
    #[serde(rename = "totalDamageTaken")]
    pub total_damage_taken: i32,
    #[serde(rename = "trueDamageDone")]
    pub true_damage_done: i32,
    #[serde(rename = "trueDamageDoneToChampions")]
    pub true_damage_done_to_champions: i32,
    #[serde(rename = "trueDamageTaken")]
    pub true_damage_taken: i32,
}
/// Position data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Position {
    #[serde(rename = "x")]
    pub x: i32,
    #[serde(rename = "y")]
    pub y: i32,
}
/// MatchTimelineVictimDamage data object.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct MatchTimelineVictimDamage {
    #[serde(rename = "basic")]
    pub basic: bool,
    #[serde(rename = "magicDamage")]
    pub magic_damage: i32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "participantId")]
    pub participant_id: i32,
    #[serde(rename = "physicalDamage")]
    pub physical_damage: i32,
    #[serde(rename = "spellName")]
    pub spell_name: String,
    #[serde(rename = "spellSlot")]
    pub spell_slot: i32,
    #[serde(rename = "trueDamage")]
    pub true_damage: i32,
    #[serde(rename = "type")]
    pub r#type: String,
}


/// https://github.com/RiotGames/developer-relations/issues/898
pub(crate) fn deserialize_empty_string_none<'de, D, T>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: serde::de::Deserializer<'de>,
    T: serde::de::Deserialize<'de>,
{
    use serde::de::{Deserialize, IntoDeserializer};
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer()).map(Some),
    }
}

pub(crate) fn serialize_empty_string_none<S, T>(
    val: &Option<T>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
    T: serde::ser::Serialize,
{
    use serde::ser::Serialize;
    if let Some(val) = val {
        val.serialize(serializer)
    } else {
        "".serialize(serializer)
    }
}


pub(crate) fn serialize_champion_result<S>(
    val: &Result<riven::consts::Champion, std::num::TryFromIntError>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    use serde::ser::Serialize;
    val.unwrap_or(riven::consts::Champion(-1)).serialize(serializer)
}

/// https://github.com/MingweiSamuel/Riven/issues/36
pub(crate) fn deserialize_champion_result<'de, D>(
    deserializer: D,
) -> Result<Result<riven::consts::Champion, std::num::TryFromIntError>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use std::convert::TryInto;
    <i64 as serde::de::Deserialize>::deserialize(deserializer).map(|id| id.try_into().map(riven::consts::Champion))
}
