use bitcode::{Decode, Encode};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::{AsRefStr, Display, EnumIter, IntoEnumIterator, IntoStaticStr};

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Hash, Encode, Decode,
    IntoPrimitive, TryFromPrimitive, EnumIter, AsRefStr, Display, IntoStaticStr
)]
#[repr(u16)]
pub enum Queue {
    #[num_enum(default)]
    #[strum(serialize = "Custom")]
    Custom = 0,
    #[strum(serialize = "SR 5v5 Blind Pick (deprecated)")]
    SummonersRift5v5BlindPickDeprecated2 = 2,
    #[strum(serialize = "SR Ranked Solo (deprecated)")]
    SummonersRift5v5RankedSoloDeprecated4 = 4,
    #[strum(serialize = "SR Ranked Premade")]
    SummonersRift5v5RankedPremade = 6,
    #[strum(serialize = "SR Co-op vs AI")]
    SummonersRiftCoOpVsAi = 7,
    TwistedTreeline3v3Normal = 8,
    TwistedTreeline3v3RankedFlexDeprecated9 = 9,
    SummonersRift5v5DraftPickDeprecated14 = 14,
    CrystalScar5v5DominionBlindPick = 16,
    CrystalScar5v5DominionDraftPick = 17,
    CrystalScarDominionCoOpVsAi = 25,
    SummonersRiftCoOpVsAiIntroBotDeprecated31 = 31,
    SummonersRiftCoOpVsAiBeginnerBotDeprecated32 = 32,
    SummonersRiftCoOpVsAiIntermediateBotDeprecated33 = 33,
    TwistedTreeline3v3RankedTeam = 41,
    SummonersRift5v5RankedTeam = 42,
    TwistedTreelineCoOpVsAi = 52,
    SummonersRift5v5TeamBuilder = 61,
    HowlingAbyss5v5AramDeprecated65 = 65,
    HowlingAbyssAramCoOpVsAi = 67,
    SummonersRiftOneForAllDeprecated70 = 70,
    HowlingAbyss1v1SnowdownShowdown = 72,
    HowlingAbyss2v2SnowdownShowdown = 73,
    SummonersRift6v6Hexakill = 75,
    SummonersRiftUltraRapidFire = 76,
    HowlingAbyssOneForAllMirrorMode = 78,
    SummonersRiftCoOpVsAiUltraRapidFire = 83,
    SummonersRiftDoomBotsRank1 = 91,
    SummonersRiftDoomBotsRank2 = 92,
    SummonersRiftDoomBotsRank5 = 93,
    CrystalScarAscensionDeprecated96 = 96,
    TwistedTreeline6v6Hexakill = 98,
    ButchersBridge5v5Aram = 100,
    HowlingAbyssLegendOfThePoroKingDeprecated300 = 300,
    SummonersRiftNemesis = 310,
    SummonersRiftBlackMarketBrawlers = 313,
    SummonersRiftNexusSiegeDeprecated315 = 315,
    CrystalScarDefinitelyNotDominion = 317,
    SummonersRiftArurfDeprecated318 = 318,
    SummonersRiftAllRandom = 325,

    #[strum(serialize = "Normal Draft Pick")]
    SummonersRift5v5DraftPick = 400,
    #[strum(serialize = "Ranked Dynamic")]
    SummonersRift5v5RankedDynamic = 410,
    #[strum(serialize = "Ranked Solo/Duo")]
    SummonersRift5v5RankedSolo = 420,
    #[strum(serialize = "Normal Blind Pick")]
    SummonersRift5v5BlindPick = 430,
    #[strum(serialize = "Ranked Flex")]
    SummonersRift5v5RankedFlex = 440,
    #[strum(serialize = "ARAM")]
    HowlingAbyss5v5Aram = 450,
    TwistedTreeline3v3BlindPick = 460,
    TwistedTreeline3v3RankedFlexDeprecated470 = 470,
    #[strum(serialize = "Normal Quickplay")]
    SummonersRiftNormalQuickplay = 490,
    SummonersRiftBloodHuntAssassin = 600,
    CosmicRuinsDarkStarSingularity = 610,
    #[strum(serialize = "Clash")]
    SummonersRiftClash = 700,
    HowlingAbyssAramClash = 720,

    TwistedTreelineCoOpVsAiIntermediateBot = 800,
    TwistedTreelineCoOpVsAiIntroBot = 810,
    TwistedTreelineCoOpVsAiBeginnerBot = 820,
    SummonersRiftCoOpVsAiIntroBot = 830,
    SummonersRiftCoOpVsAiBeginnerBot = 840,
    SummonersRiftCoOpVsAiIntermediateBot = 850,

    #[strum(serialize = "ARURF")]
    SummonersRiftArurf = 900,
    CrystalScarAscension = 910,
    HowlingAbyssLegendOfThePoroKing = 920,
    #[strum(serialize = "Nexus Siege")]
    SummonersRiftNexusSiege = 940,
    SummonersRiftDoomBotsVoting = 950,
    SummonersRiftDoomBotsStandard = 960,
    ValoranCityParkStarGuardianInvasionNormal = 980,
    ValoranCityParkStarGuardianInvasionOnslaught = 990,
    OverchargeProjectHunters = 1000,
    #[strum(serialize = "Snow ARURF")]
    SummonersRiftSnowArurf = 1010,
    #[strum(serialize = "One for All")]
    SummonersRiftOneForAll = 1020,

    CrashSiteOdysseyExtractionIntro = 1030,
    CrashSiteOdysseyExtractionCadet = 1040,
    CrashSiteOdysseyExtractionCrewmember = 1050,
    CrashSiteOdysseyExtractionCaptain = 1060,
    CrashSiteOdysseyExtractionOnslaught = 1070,

    ConvergenceTeamfightTactics = 1090,
    ConvergenceTeamfightTactics1v0 = 1091,
    ConvergenceTeamfightTactics2v0 = 1092,
    ConvergenceRankedTeamfightTactics = 1100,
    ConvergenceTeamfightTacticsTutorial = 1110,
    ConvergenceTeamfightTacticsSimulation = 1111,
    ConvergenceRankedTeamfightTacticsHyperRoll = 1130,
    ConvergenceRankedTeamfightTacticsDoubleUpWorkshopDeprecated1150 = 1150,
    ConvergenceRankedTeamfightTacticsDoubleUpWorkshop = 1160,

    NexusBlitzDeprecated1200 = 1200,
    ConvergenceTeamfightTacticsChonccsTreasure = 1210,

    #[strum(serialize = "Nexus Blitz")]
    NexusBlitz = 1300,
    #[strum(serialize = "Ultimate Spellbook")]
    SummonersRiftUltimateSpellbook = 1400,

    #[strum(serialize = "Arena")]
    Arena2v2v2v2Cherry = 1700,
    RingsOfWrathArenaCherryGames = 1710,
    SwarmSoloStrawberryGames = 1810,
    SwarmDuoStrawberryGames = 1820,
    SwarmTrioStrawberryGames = 1830,
    SwarmQuadStrawberryGames = 1840,

    #[strum(serialize = "Pick URF")]
    SummonersRiftPickUrf = 1900,

    SummonersRiftTutorial1 = 2000,
    SummonersRiftTutorial2 = 2010,
    SummonersRiftTutorial3 = 2020,

    #[strum(serialize = "The Bandlewood")]
    TheBandlewood = 2300,

    ConvergenceTeamfightTacticsSet35Revival = 6000,
}

impl Queue {
    #[inline]
    pub fn id(self) -> u16 { self.into() }

    #[inline]
    pub fn label(self) -> &'static str { self.into() }

    #[inline]
    pub fn from_id_or_custom(id: u16) -> Self {
        Self::try_from(id).unwrap_or(Queue::Custom)
    }

    #[inline]
    pub fn options_all() -> Vec<(u16, &'static str)> {
        Queue::iter().map(|q| (q.id(), q.label())).collect()
    }

    #[inline]
    pub fn options_basic() -> Vec<(u16, &'static str)> {
        use Queue::*;
        [
            SummonersRift5v5DraftPick,
            SummonersRift5v5BlindPick,
            SummonersRift5v5RankedSolo,
            SummonersRift5v5RankedFlex,
            HowlingAbyss5v5Aram,
            SummonersRiftArurf,
            SummonersRiftOneForAll,
            Arena2v2v2v2Cherry,
            SummonersRiftPickUrf,
            SummonersRiftUltimateSpellbook,
            SummonersRiftNexusSiege,
            SummonersRiftClash,
            SummonersRiftNormalQuickplay,
            NexusBlitz,
            TheBandlewood,
        ]
            .into_iter()
            .map(|q| (q.id(), q.label()))
            .collect()
    }
}
