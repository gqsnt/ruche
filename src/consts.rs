use num_enum_derive::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr, VariantNames};


pub struct Item{}
impl Item{
    pub fn get_static_url(id:i32) -> String {
        format!("/assets/items/{}.avif", id)
    }
}

pub struct ProfileIcon {}
impl ProfileIcon {
    pub fn get_static_url(id:i32) -> String {
        format!("/assets/profile_icons/{}.avif", id)
    }
}



#[repr(u16)]
#[derive(Debug, Clone, Copy)]
#[derive(Eq, PartialEq, Hash)]
#[derive(EnumIter, Display, IntoPrimitive, TryFromPrimitive)]
pub enum SummonerSpell {
    UNKNOWN = 0,
    SummonerBarrier = 21,
    SummonerBoost = 1,
    SummonerCherryFlash = 2202,
    SummonerCherryHold = 2201,
    SummonerDot = 14,
    SummonerExhaust = 3,
    SummonerFlash = 4,
    SummonerHaste = 6,
    SummonerHeal = 7,
    SummonerMana = 13,
    SummonerPoroRecall = 30,
    SummonerPoroThrow = 31,
    SummonerSmite = 11,
    #[allow(non_camel_case_types)]
    SummonerSnowURFSnowball_Mark = 39,
    SummonerSnowball = 32,
    SummonerTeleport = 12,
    #[allow(non_camel_case_types)]
    Summoner_UltBookPlaceholder = 54,
    #[allow(non_camel_case_types)]
    Summoner_UltBookSmitePlaceholder = 55,
}

impl SummonerSpell {
    pub fn get_url(&self, version: String) -> String {
        format!("https://ddragon.leagueoflegends.com/cdn/{}/img/spell/{}.png", version, self)
    }

    pub fn get_static_url(id:i32) -> String {
        format!("/assets/summoner_spells/{}.avif", id)
    }

}


#[repr(u16)]
#[derive(EnumIter, Display, Eq, PartialEq, Hash, Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
pub enum Perk {
    UNKNOWN = 0,
    Domination = 8100,
    Electrocute = 8112,
    DarkHarvest = 8128,
    HailOfBlades = 9923,
    CheapShot = 8126,
    TasteOfBlood = 8139,
    SuddenImpact = 8143,
    ZombieWard = 8136,
    GhostPoro = 8120,
    EyeballCollection = 8138,
    RavenousHunter = 8135,
    IngeniousHunter = 8134,
    RelentlessHunter = 8105,
    UltimateHunter = 8106,
    Inspiration = 8300,
    GlacialAugment = 8351,
    UnsealedSpellbook = 8360,
    FirstStrike = 8369,
    HextechFlashtraption = 8306,
    MagicalFootwear = 8304,
    CashBack = 8321,
    PerfectTiming = 8313,
    TimeWarpTonic = 8352,
    BiscuitDelivery = 8345,
    CosmicInsight = 8347,
    ApproachVelocity = 8410,
    JackOfAllTrades = 8316,
    Precision = 8000,
    PressTheAttack = 8005,
    LethalTempo = 8008,
    FleetFootwork = 8021,
    Conqueror = 8010,
    AbsorbLife = 9101,
    Triumph = 9111,
    PresenceOfMind = 8009,
    LegendAlacrity = 9104,
    LegendHaste = 9105,
    LegendBloodline = 9103,
    CoupDeGrace = 8014,
    CutDown = 8017,
    LastStand = 8299,
    Resolve = 8400,
    GraspOfTheUndying = 8437,
    Aftershock = 8439,
    Guardian = 8465,
    Demolish = 8446,
    FontOfLife = 8463,
    ShieldBash = 8401,
    Conditioning = 8429,
    SecondWind = 8444,
    BonePlating = 8473,
    Overgrowth = 8451,
    Revitalize = 8453,
    Unflinching = 8242,
    Sorcery = 8200,
    SummonAery = 8214,
    ArcaneComet = 8229,
    PhaseRush = 8230,
    NullifyingOrb = 8224,
    ManaflowBand = 8226,
    NimbusCloack = 8275,
    Transcendence = 8210,
    Celerity = 8234,
    AbsoluteFocus = 8233,
    Scorch = 8237,
    Waterwalking = 8232,
    GatheringStorm = 8236,
}


impl Perk {

    pub fn get_static_url(id:i32) -> String {
        format!("/assets/perks/{}.avif", id)
    }

    pub fn get_primary(&self) -> Option<Self> {
        match self {
            | Perk::UNKNOWN
            | Perk::Domination
            | Perk::Inspiration
            | Perk::Precision
            | Perk::Resolve
            | Perk::Sorcery => None,
            | Perk::Electrocute
            | Perk::DarkHarvest
            | Perk::HailOfBlades
            | Perk::CheapShot
            | Perk::TasteOfBlood
            | Perk::SuddenImpact
            | Perk::ZombieWard
            | Perk::GhostPoro
            | Perk::EyeballCollection
            | Perk::RavenousHunter
            | Perk::IngeniousHunter
            | Perk::RelentlessHunter
            | Perk::UltimateHunter => Some(Perk::Domination),
            | Perk::GlacialAugment
            | Perk::UnsealedSpellbook
            | Perk::FirstStrike
            | Perk::HextechFlashtraption
            | Perk::MagicalFootwear
            | Perk::CashBack
            | Perk::PerfectTiming
            | Perk::TimeWarpTonic
            | Perk::BiscuitDelivery
            | Perk::CosmicInsight
            | Perk::ApproachVelocity
            | Perk::JackOfAllTrades => Some(Perk::Inspiration),
            | Perk::PressTheAttack
            | Perk::LethalTempo
            | Perk::FleetFootwork
            | Perk::Conqueror
            | Perk::AbsorbLife
            | Perk::Triumph
            | Perk::PresenceOfMind
            | Perk::LegendAlacrity
            | Perk::LegendHaste
            | Perk::LegendBloodline
            | Perk::CoupDeGrace
            | Perk::CutDown
            | Perk::LastStand => Some(Perk::Precision),
            | Perk::GraspOfTheUndying
            | Perk::Aftershock
            | Perk::Guardian
            | Perk::Demolish
            | Perk::FontOfLife
            | Perk::ShieldBash
            | Perk::Conditioning
            | Perk::SecondWind
            | Perk::BonePlating
            | Perk::Overgrowth
            | Perk::Unflinching
            | Perk::Revitalize => Some(Perk::Resolve),
            | Perk::SummonAery
            | Perk::ArcaneComet
            | Perk::PhaseRush
            | Perk::NullifyingOrb
            | Perk::ManaflowBand
            | Perk::NimbusCloack
            | Perk::Transcendence
            | Perk::Celerity
            | Perk::AbsoluteFocus
            | Perk::Scorch
            | Perk::Waterwalking
            | Perk::GatheringStorm => Some(Perk::Sorcery),
        }
    }
}


#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[derive(Eq, PartialEq, Hash)]
#[derive(EnumIter, Display)]
pub enum Map {
    SummonersRiftOriginalSummerVariant = 1,
    SummonersRiftOriginalAutumnVariant = 2,
    TheProvingGrounds = 3,
    TwistedTreelineOriginalVersion = 4,
    TheCrystalScar = 8,
    TwistedTreeline = 10,
    SummonersRift = 11,
    HowlingAbyss = 12,
    ButchersBridge = 14,
    CosmicsRuin = 16,
    ValoranCityPark = 18,
    Substructure43 = 19,
    CrashSite = 20,
    NexusBlitz = 21,
    Convergence = 22,
    Arena = 30,
    Swarm = 33,
}


#[repr(u16)]
#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Hash,
    EnumIter,
    Display,
    IntoPrimitive,
    TryFromPrimitive
)]
pub enum Queue {
    /// `0`.
    /// Games on Custom games
    Custom = 0,

    /// `2`.
    /// 5v5 Blind Pick games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 430
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 430")]
    SummonersRift5v5BlindPickDeprecated2 = 2,

    /// `4`.
    /// 5v5 Ranked Solo games on Summoner's Rift
    ///
    /// Deprecated in favor of queueId 420
    #[deprecated(note = "Deprecated in favor of queueId 420")]
    SummonersRift5v5RankedSoloDeprecated4 = 4,

    /// `6`.
    /// 5v5 Ranked Premade games on Summoner's Rift
    ///
    /// Game mode deprecated
    #[deprecated(note = "Game mode deprecated")]
    SummonersRift5v5RankedPremade = 6,

    /// `7`.
    /// Co-op vs AI games on Summoner's Rift
    ///
    /// Deprecated in favor of queueId 32 and 33
    #[deprecated(note = "Deprecated in favor of queueId 32 and 33")]
    SummonersRiftCoOpVsAi = 7,

    /// `8`.
    /// 3v3 Normal games on Twisted Treeline
    ///
    /// Deprecated in patch 7.19 in favor of queueId 460
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 460")]
    TwistedTreeline3v3Normal = 8,

    /// `9`.
    /// 3v3 Ranked Flex games on Twisted Treeline
    ///
    /// Deprecated in patch 7.19 in favor of queueId 470
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 470")]
    TwistedTreeline3v3RankedFlexDeprecated9 = 9,

    /// `14`.
    /// 5v5 Draft Pick games on Summoner's Rift
    ///
    /// Deprecated in favor of queueId 400
    #[deprecated(note = "Deprecated in favor of queueId 400")]
    SummonersRift5v5DraftPickDeprecated14 = 14,

    /// `16`.
    /// 5v5 Dominion Blind Pick games on Crystal Scar
    ///
    /// Game mode deprecated
    #[deprecated(note = "Game mode deprecated")]
    CrystalScar5v5DominionBlindPick = 16,

    /// `17`.
    /// 5v5 Dominion Draft Pick games on Crystal Scar
    ///
    /// Game mode deprecated
    #[deprecated(note = "Game mode deprecated")]
    CrystalScar5v5DominionDraftPick = 17,

    /// `25`.
    /// Dominion Co-op vs AI games on Crystal Scar
    ///
    /// Game mode deprecated
    #[deprecated(note = "Game mode deprecated")]
    CrystalScarDominionCoOpVsAi = 25,

    /// `31`.
    /// Co-op vs AI Intro Bot games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 830
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 830")]
    SummonersRiftCoOpVsAiIntroBotDeprecated31 = 31,

    /// `32`.
    /// Co-op vs AI Beginner Bot games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 840
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 840")]
    SummonersRiftCoOpVsAiBeginnerBotDeprecated32 = 32,

    /// `33`.
    /// Co-op vs AI Intermediate Bot games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 850
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 850")]
    SummonersRiftCoOpVsAiIntermediateBotDeprecated33 = 33,

    /// `41`.
    /// 3v3 Ranked Team games on Twisted Treeline
    ///
    /// Game mode deprecated
    #[deprecated(note = "Game mode deprecated")]
    TwistedTreeline3v3RankedTeam = 41,

    /// `42`.
    /// 5v5 Ranked Team games on Summoner's Rift
    ///
    /// Game mode deprecated
    #[deprecated(note = "Game mode deprecated")]
    SummonersRift5v5RankedTeam = 42,

    /// `52`.
    /// Co-op vs AI games on Twisted Treeline
    ///
    /// Deprecated in patch 7.19 in favor of queueId 800
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 800")]
    TwistedTreelineCoOpVsAi = 52,

    /// `61`.
    /// 5v5 Team Builder games on Summoner's Rift
    ///
    /// Game mode deprecated
    #[deprecated(note = "Game mode deprecated")]
    SummonersRift5v5TeamBuilder = 61,

    /// `65`.
    /// 5v5 ARAM games on Howling Abyss
    ///
    /// Deprecated in patch 7.19 in favor of queueId 450
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 450")]
    HowlingAbyss5v5AramDeprecated65 = 65,

    /// `67`.
    /// ARAM Co-op vs AI games on Howling Abyss
    ///
    /// Game mode deprecated
    #[deprecated(note = "Game mode deprecated")]
    HowlingAbyssAramCoOpVsAi = 67,

    /// `70`.
    /// One for All games on Summoner's Rift
    ///
    /// Deprecated in patch 8.6 in favor of queueId 1020
    #[deprecated(note = "Deprecated in patch 8.6 in favor of queueId 1020")]
    SummonersRiftOneForAllDeprecated70 = 70,

    /// `72`.
    /// 1v1 Snowdown Showdown games on Howling Abyss
    HowlingAbyss1v1SnowdownShowdown = 72,

    /// `73`.
    /// 2v2 Snowdown Showdown games on Howling Abyss
    HowlingAbyss2v2SnowdownShowdown = 73,

    /// `75`.
    /// 6v6 Hexakill games on Summoner's Rift
    SummonersRift6v6Hexakill = 75,

    /// `76`.
    /// Ultra Rapid Fire games on Summoner's Rift
    SummonersRiftUltraRapidFire = 76,

    /// `78`.
    /// One For All: Mirror Mode games on Howling Abyss
    HowlingAbyssOneForAllMirrorMode = 78,

    /// `83`.
    /// Co-op vs AI Ultra Rapid Fire games on Summoner's Rift
    SummonersRiftCoOpVsAiUltraRapidFire = 83,

    /// `91`.
    /// Doom Bots Rank 1 games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 950
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 950")]
    SummonersRiftDoomBotsRank1 = 91,

    /// `92`.
    /// Doom Bots Rank 2 games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 950
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 950")]
    SummonersRiftDoomBotsRank2 = 92,

    /// `93`.
    /// Doom Bots Rank 5 games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 950
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 950")]
    SummonersRiftDoomBotsRank5 = 93,

    /// `96`.
    /// Ascension games on Crystal Scar
    ///
    /// Deprecated in patch 7.19 in favor of queueId 910
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 910")]
    CrystalScarAscensionDeprecated96 = 96,

    /// `98`.
    /// 6v6 Hexakill games on Twisted Treeline
    TwistedTreeline6v6Hexakill = 98,

    /// `100`.
    /// 5v5 ARAM games on Butcher's Bridge
    ButchersBridge5v5Aram = 100,

    /// `300`.
    /// Legend of the Poro King games on Howling Abyss
    ///
    /// Deprecated in patch 7.19 in favor of queueId 920
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 920")]
    HowlingAbyssLegendOfThePoroKingDeprecated300 = 300,

    /// `310`.
    /// Nemesis games on Summoner's Rift
    SummonersRiftNemesis = 310,

    /// `313`.
    /// Black Market Brawlers games on Summoner's Rift
    SummonersRiftBlackMarketBrawlers = 313,

    /// `315`.
    /// Nexus Siege games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 940
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 940")]
    SummonersRiftNexusSiegeDeprecated315 = 315,

    /// `317`.
    /// Definitely Not Dominion games on Crystal Scar
    CrystalScarDefinitelyNotDominion = 317,

    /// `318`.
    /// ARURF games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 900
    #[deprecated(note = "Deprecated in patch 7.19 in favor of queueId 900")]
    SummonersRiftArurfDeprecated318 = 318,

    /// `325`.
    /// All Random games on Summoner's Rift
    SummonersRiftAllRandom = 325,

    /// `400`.
    /// 5v5 Draft Pick games on Summoner's Rift
    SummonersRift5v5DraftPick = 400,

    /// `410`.
    /// 5v5 Ranked Dynamic games on Summoner's Rift
    ///
    /// Game mode deprecated in patch 6.22
    #[deprecated(note = "Game mode deprecated in patch 6.22")]
    SummonersRift5v5RankedDynamic = 410,

    /// `420`.
    /// 5v5 Ranked Solo games on Summoner's Rift
    SummonersRift5v5RankedSolo = 420,

    /// `430`.
    /// 5v5 Blind Pick games on Summoner's Rift
    SummonersRift5v5BlindPick = 430,

    /// `440`.
    /// 5v5 Ranked Flex games on Summoner's Rift
    SummonersRift5v5RankedFlex = 440,

    /// `450`.
    /// 5v5 ARAM games on Howling Abyss
    HowlingAbyss5v5Aram = 450,

    /// `460`.
    /// 3v3 Blind Pick games on Twisted Treeline
    ///
    /// Deprecated in patch 9.23
    #[deprecated(note = "Deprecated in patch 9.23")]
    TwistedTreeline3v3BlindPick = 460,

    /// `470`.
    /// 3v3 Ranked Flex games on Twisted Treeline
    ///
    /// Deprecated in patch 9.23
    #[deprecated(note = "Deprecated in patch 9.23")]
    TwistedTreeline3v3RankedFlexDeprecated470 = 470,

    /// `490`.
    /// Normal (Quickplay) games on Summoner's Rift
    SummonersRiftNormalQuickplay = 490,

    /// `600`.
    /// Blood Hunt Assassin games on Summoner's Rift
    SummonersRiftBloodHuntAssassin = 600,

    /// `610`.
    /// Dark Star: Singularity games on Cosmic Ruins
    CosmicRuinsDarkStarSingularity = 610,

    /// `700`.
    /// Summoner's Rift Clash games on Summoner's Rift
    SummonersRiftClash = 700,

    /// `720`.
    /// ARAM Clash games on Howling Abyss
    HowlingAbyssAramClash = 720,

    /// `800`.
    /// Co-op vs. AI Intermediate Bot games on Twisted Treeline
    ///
    /// Deprecated in patch 9.23
    #[deprecated(note = "Deprecated in patch 9.23")]
    TwistedTreelineCoOpVsAiIntermediateBot = 800,

    /// `810`.
    /// Co-op vs. AI Intro Bot games on Twisted Treeline
    ///
    /// Deprecated in patch 9.23
    #[deprecated(note = "Deprecated in patch 9.23")]
    TwistedTreelineCoOpVsAiIntroBot = 810,

    /// `820`.
    /// Co-op vs. AI Beginner Bot games on Twisted Treeline
    TwistedTreelineCoOpVsAiBeginnerBot = 820,

    /// `830`.
    /// Co-op vs. AI Intro Bot games on Summoner's Rift
    SummonersRiftCoOpVsAiIntroBot = 830,

    /// `840`.
    /// Co-op vs. AI Beginner Bot games on Summoner's Rift
    SummonersRiftCoOpVsAiBeginnerBot = 840,

    /// `850`.
    /// Co-op vs. AI Intermediate Bot games on Summoner's Rift
    SummonersRiftCoOpVsAiIntermediateBot = 850,

    /// `900`.
    /// ARURF games on Summoner's Rift
    SummonersRiftArurf = 900,

    /// `910`.
    /// Ascension games on Crystal Scar
    CrystalScarAscension = 910,

    /// `920`.
    /// Legend of the Poro King games on Howling Abyss
    HowlingAbyssLegendOfThePoroKing = 920,

    /// `940`.
    /// Nexus Siege games on Summoner's Rift
    SummonersRiftNexusSiege = 940,

    /// `950`.
    /// Doom Bots Voting games on Summoner's Rift
    SummonersRiftDoomBotsVoting = 950,

    /// `960`.
    /// Doom Bots Standard games on Summoner's Rift
    SummonersRiftDoomBotsStandard = 960,

    /// `980`.
    /// Star Guardian Invasion: Normal games on Valoran City Park
    ValoranCityParkStarGuardianInvasionNormal = 980,

    /// `990`.
    /// Star Guardian Invasion: Onslaught games on Valoran City Park
    ValoranCityParkStarGuardianInvasionOnslaught = 990,

    /// `1000`.
    /// PROJECT: Hunters games on Overcharge
    OverchargeProjectHunters = 1000,

    /// `1010`.
    /// Snow ARURF games on Summoner's Rift
    SummonersRiftSnowArurf = 1010,

    /// `1020`.
    /// One for All games on Summoner's Rift
    SummonersRiftOneForAll = 1020,

    /// `1030`.
    /// Odyssey Extraction: Intro games on Crash Site
    CrashSiteOdysseyExtractionIntro = 1030,

    /// `1040`.
    /// Odyssey Extraction: Cadet games on Crash Site
    CrashSiteOdysseyExtractionCadet = 1040,

    /// `1050`.
    /// Odyssey Extraction: Crewmember games on Crash Site
    CrashSiteOdysseyExtractionCrewmember = 1050,

    /// `1060`.
    /// Odyssey Extraction: Captain games on Crash Site
    CrashSiteOdysseyExtractionCaptain = 1060,

    /// `1070`.
    /// Odyssey Extraction: Onslaught games on Crash Site
    CrashSiteOdysseyExtractionOnslaught = 1070,

    /// `1090`.
    /// Teamfight Tactics games on Convergence
    ConvergenceTeamfightTactics = 1090,

    /// `1091`.
    /// Teamfight Tactics 1v0 games on Convergence
    ConvergenceTeamfightTactics1v0 = 1091,

    /// `1092`.
    /// Teamfight Tactics 2v0 games on Convergence
    ConvergenceTeamfightTactics2v0 = 1092,

    /// `1100`.
    /// Ranked Teamfight Tactics games on Convergence
    ConvergenceRankedTeamfightTactics = 1100,

    /// `1110`.
    /// Teamfight Tactics Tutorial games on Convergence
    ConvergenceTeamfightTacticsTutorial = 1110,

    /// `1111`.
    /// Teamfight Tactics Simulation games on Convergence
    ConvergenceTeamfightTacticsSimulation = 1111,

    /// `1130`.
    /// Ranked Teamfight Tactics (Hyper Roll) games on Convergence
    ConvergenceRankedTeamfightTacticsHyperRoll = 1130,

    /// `1150`.
    /// Ranked Teamfight Tactics (Double Up Workshop) games on Convergence
    ///
    /// Deprecated in patch 12.11 in favor of queueId 1160
    #[deprecated(note = "Deprecated in patch 12.11 in favor of queueId 1160")]
    ConvergenceRankedTeamfightTacticsDoubleUpWorkshopDeprecated1150 = 1150,

    /// `1160`.
    /// Ranked Teamfight Tactics (Double Up Workshop) games on Convergence
    ConvergenceRankedTeamfightTacticsDoubleUpWorkshop = 1160,

    /// `1200`.
    /// Nexus Blitz games on Nexus Blitz
    ///
    /// Deprecated in patch 9.2 in favor of queueId 1300
    #[deprecated(note = "Deprecated in patch 9.2 in favor of queueId 1300")]
    NexusBlitzDeprecated1200 = 1200,

    /// `1210`.
    /// Teamfight Tactics (Choncc's Treasure) games on Convergence
    ConvergenceTeamfightTacticsChonccsTreasure = 1210,

    /// `1300`.
    /// Nexus Blitz games on Nexus Blitz
    NexusBlitz = 1300,

    /// `1400`.
    /// Ultimate Spellbook games on Summoner's Rift
    SummonersRiftUltimateSpellbook = 1400,

    /// `1700`.
    /// 2v2v2v2 `CHERRY` games on Arena
    Arena2v2v2v2Cherry = 1700,

    /// `1710`.
    /// Arena (`CHERRY` games) games on Rings of Wrath
    RingsOfWrathArenaCherryGames = 1710,

    /// `1810`.
    /// Swarm solo (`STRAWBERRY` games) games on Swarm
    SwarmSoloStrawberryGames = 1810,

    /// `1820`.
    /// Swarm duo (`STRAWBERRY` games) games on Swarm
    SwarmDuoStrawberryGames = 1820,

    /// `1830`.
    /// Swarm trio (`STRAWBERRY` games) games on Swarm
    SwarmTrioStrawberryGames = 1830,

    /// `1840`.
    /// Swarm quad (`STRAWBERRY` games) games on Swarm
    SwarmQuadStrawberryGames = 1840,

    /// `1900`.
    /// Pick URF games on Summoner's Rift
    SummonersRiftPickUrf = 1900,

    /// `2000`.
    /// Tutorial 1 games on Summoner's Rift
    SummonersRiftTutorial1 = 2000,

    /// `2010`.
    /// Tutorial 2 games on Summoner's Rift
    SummonersRiftTutorial2 = 2010,

    /// `2020`.
    /// Tutorial 3 games on Summoner's Rift
    SummonersRiftTutorial3 = 2020,

    /// `6000`.
    /// Teamfight Tactics Set 3.5 Revival games on Convergence
    ConvergenceTeamfightTacticsSet35Revival = 6000,
}


pub static QUEUE_OPTIONS: &[(u16, &str)] = &[
    (Queue::SummonersRift5v5DraftPick as u16, "Normal Draft Pick"),
    (Queue::SummonersRift5v5BlindPick as u16, "Normal Blind Pick"),
    (Queue::SummonersRift5v5RankedSolo as u16, "Ranked Solo/Duo"),
    (Queue::SummonersRift5v5RankedFlex as u16, "Ranked Flex"),
    (Queue::HowlingAbyss5v5Aram as u16, "ARAM"),
    (Queue::SummonersRiftArurf as u16, "ARURF"),
    (Queue::SummonersRiftOneForAll as u16, "One for All"),
    (Queue::Arena2v2v2v2Cherry as u16, "2v2v2v2 Cherry"),
    (Queue::SummonersRiftPickUrf as u16, "Pick URF"),
    (Queue::SummonersRiftUltimateSpellbook as u16, "Ultimate Spellbook"),
    (Queue::SummonersRiftNexusSiege as u16, "Nexus Siege"),
    (Queue::SummonersRiftClash as u16, "Clash"),
    (Queue::SummonersRiftNormalQuickplay as u16, "Normal Quickplay"),
    (Queue::NexusBlitz as u16, "Nexus Blitz"),
];

#[derive(Debug, Clone)]
#[derive(Eq, PartialEq, Hash)]
#[derive(EnumString, VariantNames, IntoStaticStr)]
#[repr(u8)]
pub enum GameMode {
    /// Catch-all variant for new, unknown game modes.
    #[strum(default)]
    UNKNOWN(String),

    /// ARAM games
    ARAM,
    /// All Random Summoner's Rift games
    ARSR,
    /// Ascension games
    ASCENSION,
    /// Blood Hunt Assassin games
    ASSASSINATE,
    /// 2v2v2v2 Arena
    CHERRY,
    /// Classic Summoner's Rift and Twisted Treeline games
    CLASSIC,
    /// Dark Star: Singularity games
    DARKSTAR,
    /// Doom Bot games
    DOOMBOTSTEEMO,
    /// Snowdown Showdown games
    FIRSTBLOOD,
    /// Nexus Blitz games
    GAMEMODEX,
    /// Legend of the Poro King games
    KINGPORO,
    /// Nexus Blitz games
    NEXUSBLITZ,
    /// Dominion/Crystal Scar games
    ODIN,
    /// Odyssey: Extraction games
    ODYSSEY,
    /// One for All games
    ONEFORALL,
    /// Practice tool training games.
    PRACTICETOOL,
    /// PROJECT: Hunters games
    PROJECT,
    /// Nexus Siege games
    SIEGE,
    /// Star Guardian Invasion games
    STARGUARDIAN,
    /// Swarm
    STRAWBERRY,
    /// Teamfight Tactics.
    TFT,
    /// Tutorial games
    TUTORIAL,
    /// Tutorial: Welcome to League.
    TutorialModule1,
    /// Tutorial: Power Up.
    TutorialModule2,
    /// Tutorial: Shop for Gear.
    TutorialModule3,
    /// Ultimate Spellbook games
    ULTBOOK,
    /// URF games
    URF,
}

#[repr(i16)]
#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Hash,
    EnumIter,
    Display,
    IntoPrimitive,
    TryFromPrimitive
)]
pub enum Champion {
    UNKNOWN = -1,
    Aatrox = 266,
    Ahri = 103,
    Akali = 84,
    Akshan = 166,
    Alistar = 12,
    Amumu = 32,
    Anivia = 34,
    Annie = 1,
    Aphelios = 523,
    Ashe = 22,
    AurelionSol = 136,
    Aurora = 893,
    Azir = 268,
    Bard = 432,
    BelVeth = 200,
    Blitzcrank = 53,
    Brand = 63,
    Braum = 201,
    Briar = 233,
    Caitlyn = 51,
    Camille = 164,
    Cassiopeia = 69,
    ChoGath = 31,
    Corki = 42,
    Darius = 122,
    Diana = 131,
    DrMundo = 36,
    Draven = 119,
    Ekko = 245,
    Elise = 60,
    Evelynn = 28,
    Ezreal = 81,
    Fiddlesticks = 9,
    Fiora = 114,
    Fizz = 105,
    Galio = 3,
    Gangplank = 41,
    Garen = 86,
    Gnar = 150,
    Gragas = 79,
    Graves = 104,
    Gwen = 887,
    Hecarim = 120,
    Heimerdinger = 74,
    Hwei = 910,
    Illaoi = 420,
    Irelia = 39,
    Ivern = 427,
    Janna = 40,
    JarvanIV = 59,
    Jax = 24,
    Jayce = 126,
    Jhin = 202,
    Jinx = 222,
    KSante = 897,
    KaiSa = 145,
    Kalista = 429,
    Karma = 43,
    Karthus = 30,
    Kassadin = 38,
    Katarina = 55,
    Kayle = 10,
    Kayn = 141,
    Kennen = 85,
    KhaZix = 121,
    Kindred = 203,
    Kled = 240,
    KogMaw = 96,
    LeBlanc = 7,
    LeeSin = 64,
    Leona = 89,
    Lillia = 876,
    Lissandra = 127,
    Lucian = 236,
    Lulu = 117,
    Lux = 99,
    Malphite = 54,
    Malzahar = 90,
    Maokai = 57,
    MasterYi = 11,
    Milio = 902,
    MissFortune = 21,
    Mordekaiser = 82,
    Morgana = 25,
    NAAFIRI = 950,
    Nami = 267,
    Nasus = 75,
    Nautilus = 111,
    Neeko = 518,
    Nidalee = 76,
    Nilah = 895,
    Nocturne = 56,
    NunuWillump = 20,
    Olaf = 2,
    Orianna = 61,
    Ornn = 516,
    Pantheon = 80,
    Poppy = 78,
    Pyke = 555,
    Qiyana = 246,
    Quinn = 133,
    Rakan = 497,
    Rammus = 33,
    RekSai = 421,
    Rell = 526,
    RenataGlasc = 888,
    Renekton = 58,
    Rengar = 107,
    Riven = 92,
    Rumble = 68,
    Ryze = 13,
    Samira = 360,
    Sejuani = 113,
    Senna = 235,
    Seraphine = 147,
    Sett = 875,
    Shaco = 35,
    Shen = 98,
    Shyvana = 102,
    Singed = 27,
    Sion = 14,
    Sivir = 15,
    Skarner = 72,
    Smolder = 901,
    Sona = 37,
    Soraka = 16,
    Swain = 50,
    Sylas = 517,
    Syndra = 134,
    TahmKench = 223,
    Taliyah = 163,
    Talon = 91,
    Taric = 44,
    Teemo = 17,
    Thresh = 412,
    Tristana = 18,
    Trundle = 48,
    Tryndamere = 23,
    TwistedFate = 4,
    Twitch = 29,
    Udyr = 77,
    Urgot = 6,
    Varus = 110,
    Vayne = 67,
    Veigar = 45,
    VelKoz = 161,
    Vex = 711,
    Vi = 254,
    Viego = 234,
    Viktor = 112,
    Vladimir = 8,
    Volibear = 106,
    Warwick = 19,
    Wukong = 62,
    Xayah = 498,
    Xerath = 101,
    XinZhao = 5,
    Yasuo = 157,
    Yone = 777,
    Yorick = 83,
    Yuumi = 350,
    Zac = 154,
    Zed = 238,
    Zeri = 221,
    Ziggs = 115,
    Zilean = 26,
    Zoe = 142,
    Zyra = 143,
}

impl Champion{
    pub fn get_static_url(id:i32) -> String{
        format!("/assets/champions/{}.avif",id)
    }
}


#[derive(Debug)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(TryFromPrimitive, IntoPrimitive)]
#[derive(EnumString, EnumIter, Display, IntoStaticStr)]
#[derive(Clone, Copy)]
#[repr(u8)]
#[non_exhaustive]
pub enum RegionalRoute {
    /// North and South America.
    ///
    /// `1` (riotapi-schema ID/repr)
    AMERICAS = 1,

    /// Asia, used for LoL matches (`match-v5`) and TFT matches (`tft-match-v1`).
    ///
    /// `2` (riotapi-schema ID/repr)
    ASIA = 2,

    /// Europe.
    ///
    /// `3` (riotapi-schema ID/repr)
    EUROPE = 3,

    /// South East Asia, used for LoR, LoL matches (`match-v5`), and TFT matches (`tft-match-v1`).
    ///
    /// `4` (riotapi-schema ID/repr)
    SEA = 4,

    /// Asia-Pacific, deprecated, for some old matches in `lor-match-v1`.
    ///
    /// `10` (riotapi-schema ID/repr)
    #[allow(deprecated)]
    APAC = 10,

    /// Special esports platform for `account-v1`. Do not confuse with the `esports` Valorant platform route.
    ///
    /// `11` (riotapi-schema ID/repr)
    ESPORTS = 11,

}

/// Platform routes for League of Legends (LoL), Teamfight Tactics (TFT), and Legends of Runeterra (LoR).
#[derive(Debug)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(TryFromPrimitive, IntoPrimitive)]
#[derive(EnumString, EnumIter, Display, IntoStaticStr, Serialize, Deserialize)]
#[derive(Clone, Copy)]
#[repr(u8)]
#[non_exhaustive]
// Note: strum(serialize = ...) actually specifies extra DEserialization values.
pub enum PlatformRoute {
    /// Brazil.
    ///
    /// `16` (riotapi-schema ID/repr)
    #[strum(to_string = "BR1", serialize = "BR")]
    BR1 = 16,

    /// Europe, Northeast.
    ///
    /// `17` (riotapi-schema ID/repr)
    #[strum(to_string = "EUN1", serialize = "EUNE")]
    EUN1 = 17,

    /// Europe, West.
    ///
    /// `18` (riotapi-schema ID/repr)
    #[strum(to_string = "EUW1", serialize = "EUW")]
    EUW1 = 18,

    /// Japan.
    ///
    /// `19` (riotapi-schema ID/repr)
    #[strum(to_string = "JP1", serialize = "JP")]
    JP1 = 19,

    /// Korea.
    ///
    /// `20` (riotapi-schema ID/repr)
    KR = 20,

    /// Latin America, North.
    ///
    /// `21` (riotapi-schema ID/repr)
    #[strum(to_string = "LA1", serialize = "LAN")]
    LA1 = 21,

    /// Latin America, South.
    ///
    /// `22` (riotapi-schema ID/repr)
    #[strum(to_string = "LA2", serialize = "LAS")]
    LA2 = 22,

    /// Middle East and North Africa.
    ///
    /// `37` (riotapi-schema ID/repr)
    #[strum(to_string = "ME1", serialize = "MENA")]
    ME1 = 37,

    /// North America.
    ///
    /// `23` (riotapi-schema ID/repr)
    #[strum(to_string = "NA1", serialize = "NA")]
    NA1 = 23,

    /// Oceania.
    ///
    /// `24` (riotapi-schema ID/repr)
    #[strum(to_string = "OC1", serialize = "OCE")]
    OC1 = 24,

    /// Philippines
    ///
    /// `32` (riotapi-schema ID/repr)
    #[strum(to_string = "PH2", serialize = "PH")]
    PH2 = 32,

    /// Russia
    ///
    /// `25` (riotapi-schema ID/repr)
    RU = 25,

    /// Singapore
    ///
    /// `33` (riotapi-schema ID/repr)
    #[strum(to_string = "SG2", serialize = "SG")]
    SG2 = 33,

    /// Thailand
    ///
    /// `34` (riotapi-schema ID/repr)
    #[strum(to_string = "TH2", serialize = "TH")]
    TH2 = 34,

    /// Turkey
    ///
    /// `26` (riotapi-schema ID/repr)
    #[strum(to_string = "TR1", serialize = "TR")]
    TR1 = 26,

    /// Taiwan
    ///
    /// `35` (riotapi-schema ID/repr)
    #[strum(to_string = "TW2", serialize = "TW")]
    TW2 = 35,

    /// Vietnam
    ///
    /// `36` (riotapi-schema ID/repr)
    #[strum(to_string = "VN2", serialize = "VN")]
    VN2 = 36,

    /// Public Beta Environment, special beta testing platform. Located in North America.
    ///
    /// `31` (riotapi-schema ID/repr)
    #[strum(to_string = "PBE1", serialize = "PBE")]
    PBE1 = 31,

}

impl PlatformRoute {

    #[cfg(feature = "ssr")]
    pub fn to_riven(&self) -> riven::consts::PlatformRoute {
        riven::consts::PlatformRoute::from_str(self.to_string().as_str()).unwrap()
    }


    /// Converts this [`PlatformRoute`] into its corresponding
    /// [`RegionalRoute`] for LoL and TFT match endpoints.
    /// For example, [`match-v5`](crate::endpoints::MatchV5).
    pub fn to_regional(self) -> RegionalRoute {
        match self {
            Self::BR1 => RegionalRoute::AMERICAS,
            Self::EUN1 => RegionalRoute::EUROPE,
            Self::EUW1 => RegionalRoute::EUROPE,
            Self::JP1 => RegionalRoute::ASIA,
            Self::KR => RegionalRoute::ASIA,
            Self::LA1 => RegionalRoute::AMERICAS,
            Self::LA2 => RegionalRoute::AMERICAS,
            Self::ME1 => RegionalRoute::EUROPE,
            Self::NA1 => RegionalRoute::AMERICAS,
            Self::OC1 => RegionalRoute::SEA,
            Self::PH2 => RegionalRoute::SEA,
            Self::RU => RegionalRoute::EUROPE,
            Self::SG2 => RegionalRoute::SEA,
            Self::TH2 => RegionalRoute::SEA,
            Self::TR1 => RegionalRoute::EUROPE,
            Self::TW2 => RegionalRoute::SEA,
            Self::VN2 => RegionalRoute::SEA,
            Self::PBE1 => RegionalRoute::AMERICAS,
        }
    }

    /// Converts this [`PlatformRoute`] into its corresponding
    /// [`RegionalRoute`] for LoR endpoints.
    /// For example, [`lor-match-v1`](crate::endpoints::LorMatchV1).
    pub fn to_regional_lor(self) -> RegionalRoute {
        match self {
            Self::BR1 => RegionalRoute::AMERICAS,
            Self::EUN1 => RegionalRoute::EUROPE,
            Self::EUW1 => RegionalRoute::EUROPE,
            Self::JP1 => RegionalRoute::ASIA,
            Self::KR => RegionalRoute::ASIA,
            Self::LA1 => RegionalRoute::AMERICAS,
            Self::LA2 => RegionalRoute::AMERICAS,
            Self::ME1 => RegionalRoute::EUROPE,
            Self::NA1 => RegionalRoute::AMERICAS,
            Self::OC1 => RegionalRoute::SEA,
            Self::PH2 => RegionalRoute::SEA,
            Self::RU => RegionalRoute::SEA,
            Self::SG2 => RegionalRoute::SEA,
            Self::TH2 => RegionalRoute::SEA,
            Self::TR1 => RegionalRoute::SEA,
            Self::TW2 => RegionalRoute::SEA,
            Self::VN2 => RegionalRoute::SEA,
            Self::PBE1 => RegionalRoute::AMERICAS,
        }
    }


    /// Get the slightly more human-friendly alternate name for this `PlatformRoute`. Specifically
    /// excludes any trailing numbers and appends extra N(orth), S(outh), E(ast), and/or W(est)
    /// suffixes to some names. Some of these are old region names which are often still used as
    /// user-facing names, e.g. on op.gg.
    ///
    /// Note these strings *are* handled by the `FromStr` implementation, if you wish to parse them
    /// back into `PlatformRoute`s.
    pub fn as_region_str(self) -> &'static str {
        match self {
            Self::BR1 => "BR",
            Self::EUN1 => "EUNE",
            Self::EUW1 => "EUW",
            Self::JP1 => "JP",
            Self::LA1 => "LAN",
            Self::LA2 => "LAS",
            Self::ME1 => "MENA",
            Self::NA1 => "NA",
            Self::OC1 => "OCE",
            Self::PH2 => "PH",
            Self::SG2 => "SG",
            Self::TH2 => "TH",
            Self::TR1 => "TR",
            Self::TW2 => "TW",
            Self::VN2 => "VN",
            Self::PBE1 => "PBE",
            other => other.into(),
        }
    }

    pub fn from_region_str(region: &str) -> Option<Self> {
        match region {
            "BR" => Some(Self::BR1),
            "EUNE" => Some(Self::EUN1),
            "EUW" => Some(Self::EUW1),
            "JP" => Some(Self::JP1),
            "LAN" => Some(Self::LA1),
            "LAS" => Some(Self::LA2),
            "MENA" => Some(Self::ME1),
            "NA" => Some(Self::NA1),
            "OCE" => Some(Self::OC1),
            "PH" => Some(Self::PH2),
            "SG" => Some(Self::SG2),
            "TH" => Some(Self::TH2),
            "TR" => Some(Self::TR1),
            "TW" => Some(Self::TW2),
            "VN" => Some(Self::VN2),
            "PBE" => Some(Self::PBE1),
            _ => Self::from_str(region).ok(),
        }
    }
}

