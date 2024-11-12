#[repr(u16)]
#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Hash,
)]
pub enum Queue {
    /// `0`.
    /// Games on Custom games
    Custom = 0,

    /// `2`.
    /// 5v5 Blind Pick games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 430
    SummonersRift5v5BlindPickDeprecated2 = 2,

    /// `4`.
    /// 5v5 Ranked Solo games on Summoner's Rift
    ///
    /// Deprecated in favor of queueId 420
    SummonersRift5v5RankedSoloDeprecated4 = 4,

    /// `6`.
    /// 5v5 Ranked Premade games on Summoner's Rift
    ///
    /// Game mode deprecated
    SummonersRift5v5RankedPremade = 6,

    /// `7`.
    /// Co-op vs AI games on Summoner's Rift
    ///
    /// Deprecated in favor of queueId 32 and 33
    SummonersRiftCoOpVsAi = 7,

    /// `8`.
    /// 3v3 Normal games on Twisted Treeline
    ///
    /// Deprecated in patch 7.19 in favor of queueId 460
    TwistedTreeline3v3Normal = 8,

    /// `9`.
    /// 3v3 Ranked Flex games on Twisted Treeline
    ///
    /// Deprecated in patch 7.19 in favor of queueId 470
    TwistedTreeline3v3RankedFlexDeprecated9 = 9,

    /// `14`.
    /// 5v5 Draft Pick games on Summoner's Rift
    ///
    /// Deprecated in favor of queueId 400
    SummonersRift5v5DraftPickDeprecated14 = 14,

    /// `16`.
    /// 5v5 Dominion Blind Pick games on Crystal Scar
    ///
    /// Game mode deprecated
    CrystalScar5v5DominionBlindPick = 16,

    /// `17`.
    /// 5v5 Dominion Draft Pick games on Crystal Scar
    ///
    /// Game mode deprecated
    CrystalScar5v5DominionDraftPick = 17,

    /// `25`.
    /// Dominion Co-op vs AI games on Crystal Scar
    ///
    /// Game mode deprecated
    CrystalScarDominionCoOpVsAi = 25,

    /// `31`.
    /// Co-op vs AI Intro Bot games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 830
    SummonersRiftCoOpVsAiIntroBotDeprecated31 = 31,

    /// `32`.
    /// Co-op vs AI Beginner Bot games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 840
    SummonersRiftCoOpVsAiBeginnerBotDeprecated32 = 32,

    /// `33`.
    /// Co-op vs AI Intermediate Bot games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 850
    SummonersRiftCoOpVsAiIntermediateBotDeprecated33 = 33,

    /// `41`.
    /// 3v3 Ranked Team games on Twisted Treeline
    ///
    /// Game mode deprecated
    TwistedTreeline3v3RankedTeam = 41,

    /// `42`.
    /// 5v5 Ranked Team games on Summoner's Rift
    ///
    /// Game mode deprecated
    SummonersRift5v5RankedTeam = 42,

    /// `52`.
    /// Co-op vs AI games on Twisted Treeline
    ///
    /// Deprecated in patch 7.19 in favor of queueId 800
    TwistedTreelineCoOpVsAi = 52,

    /// `61`.
    /// 5v5 Team Builder games on Summoner's Rift
    ///
    /// Game mode deprecated
    SummonersRift5v5TeamBuilder = 61,

    /// `65`.
    /// 5v5 ARAM games on Howling Abyss
    ///
    /// Deprecated in patch 7.19 in favor of queueId 450
    HowlingAbyss5v5AramDeprecated65 = 65,

    /// `67`.
    /// ARAM Co-op vs AI games on Howling Abyss
    ///
    /// Game mode deprecated
    HowlingAbyssAramCoOpVsAi = 67,

    /// `70`.
    /// One for All games on Summoner's Rift
    ///
    /// Deprecated in patch 8.6 in favor of queueId 1020
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
    SummonersRiftDoomBotsRank1 = 91,

    /// `92`.
    /// Doom Bots Rank 2 games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 950
    SummonersRiftDoomBotsRank2 = 92,

    /// `93`.
    /// Doom Bots Rank 5 games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 950
    SummonersRiftDoomBotsRank5 = 93,

    /// `96`.
    /// Ascension games on Crystal Scar
    ///
    /// Deprecated in patch 7.19 in favor of queueId 910
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
    SummonersRiftNexusSiegeDeprecated315 = 315,

    /// `317`.
    /// Definitely Not Dominion games on Crystal Scar
    CrystalScarDefinitelyNotDominion = 317,

    /// `318`.
    /// ARURF games on Summoner's Rift
    ///
    /// Deprecated in patch 7.19 in favor of queueId 900
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
    TwistedTreeline3v3BlindPick = 460,

    /// `470`.
    /// 3v3 Ranked Flex games on Twisted Treeline
    ///
    /// Deprecated in patch 9.23
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
    TwistedTreelineCoOpVsAiIntermediateBot = 800,

    /// `810`.
    /// Co-op vs. AI Intro Bot games on Twisted Treeline
    ///
    /// Deprecated in patch 9.23
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
    ConvergenceRankedTeamfightTacticsDoubleUpWorkshopDeprecated1150 = 1150,

    /// `1160`.
    /// Ranked Teamfight Tactics (Double Up Workshop) games on Convergence
    ConvergenceRankedTeamfightTacticsDoubleUpWorkshop = 1160,

    /// `1200`.
    /// Nexus Blitz games on Nexus Blitz
    ///
    /// Deprecated in patch 9.2 in favor of queueId 1300
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


impl Queue {
    pub const fn to_str(&self) -> &'static str {
        match self {
            Queue::SummonersRift5v5DraftPick => "Normal Draft Pick",
            Queue::SummonersRift5v5BlindPick => "Normal Blind Pick",
            Queue::SummonersRift5v5RankedSolo => "Ranked Solo/Duo",
            Queue::SummonersRift5v5RankedFlex => "Ranked Flex",
            Queue::HowlingAbyss5v5Aram => "ARAM",
            Queue::SummonersRiftArurf => "ARURF",
            Queue::SummonersRiftOneForAll => "One for All",
            Queue::Arena2v2v2v2Cherry => "2v2v2v2 Cherry",
            Queue::SummonersRiftPickUrf => "Pick URF",
            Queue::SummonersRiftUltimateSpellbook => "Ultimate Spellbook",
            Queue::SummonersRiftNexusSiege => "Nexus Siege",
            Queue::SummonersRiftClash => "Clash",
            Queue::SummonersRiftNormalQuickplay => "Normal Quickplay",
            Queue::NexusBlitz => "Nexus Blitz",
            _ => "Unknown"
        }
    }
}


pub static QUEUE_OPTIONS: &[(u16, &str)] = &[
    (Queue::SummonersRift5v5DraftPick as u16, Queue::SummonersRift5v5DraftPick.to_str()),
    (Queue::SummonersRift5v5BlindPick as u16, Queue::SummonersRift5v5BlindPick.to_str()),
    (Queue::SummonersRift5v5RankedSolo as u16, Queue::SummonersRift5v5RankedSolo.to_str()),
    (Queue::SummonersRift5v5RankedFlex as u16, Queue::SummonersRift5v5RankedFlex.to_str()),
    (Queue::HowlingAbyss5v5Aram as u16, Queue::HowlingAbyss5v5Aram.to_str()),
    (Queue::SummonersRiftArurf as u16, Queue::SummonersRiftArurf.to_str()),
    (Queue::SummonersRiftOneForAll as u16, Queue::SummonersRiftOneForAll.to_str()),
    (Queue::Arena2v2v2v2Cherry as u16, Queue::Arena2v2v2v2Cherry.to_str()),
    (Queue::SummonersRiftPickUrf as u16, Queue::SummonersRiftPickUrf.to_str()),
    (Queue::SummonersRiftUltimateSpellbook as u16, Queue::SummonersRiftUltimateSpellbook.to_str()),
    (Queue::SummonersRiftNexusSiege as u16, Queue::SummonersRiftNexusSiege.to_str()),
    (Queue::SummonersRiftClash as u16, Queue::SummonersRiftClash.to_str()),
    (Queue::SummonersRiftNormalQuickplay as u16, Queue::SummonersRiftNormalQuickplay.to_str()),
    (Queue::NexusBlitz as u16, Queue::NexusBlitz.to_str()),
];

impl From<u16> for Queue {
    fn from(value: u16) -> Self {
        match value {
            0 => Queue::Custom,
            2 => Queue::SummonersRift5v5BlindPickDeprecated2,
            4 => Queue::SummonersRift5v5RankedSoloDeprecated4,
            6 => Queue::SummonersRift5v5RankedPremade,
            7 => Queue::SummonersRiftCoOpVsAi,
            8 => Queue::TwistedTreeline3v3Normal,
            9 => Queue::TwistedTreeline3v3RankedFlexDeprecated9,
            14 => Queue::SummonersRift5v5DraftPickDeprecated14,
            16 => Queue::CrystalScar5v5DominionBlindPick,
            17 => Queue::CrystalScar5v5DominionDraftPick,
            25 => Queue::CrystalScarDominionCoOpVsAi,
            31 => Queue::SummonersRiftCoOpVsAiIntroBotDeprecated31,
            32 => Queue::SummonersRiftCoOpVsAiBeginnerBotDeprecated32,
            33 => Queue::SummonersRiftCoOpVsAiIntermediateBotDeprecated33,
            41 => Queue::TwistedTreeline3v3RankedTeam,
            42 => Queue::SummonersRift5v5RankedTeam,
            52 => Queue::TwistedTreelineCoOpVsAi,
            61 => Queue::SummonersRift5v5TeamBuilder,
            65 => Queue::HowlingAbyss5v5AramDeprecated65,
            67 => Queue::HowlingAbyssAramCoOpVsAi,
            70 => Queue::SummonersRiftOneForAllDeprecated70,
            72 => Queue::HowlingAbyss1v1SnowdownShowdown,
            73 => Queue::HowlingAbyss2v2SnowdownShowdown,
            75 => Queue::SummonersRift6v6Hexakill,
            76 => Queue::SummonersRiftUltraRapidFire,
            78 => Queue::HowlingAbyssOneForAllMirrorMode,
            83 => Queue::SummonersRiftCoOpVsAiUltraRapidFire,
            91 => Queue::SummonersRiftDoomBotsRank1,
            92 => Queue::SummonersRiftDoomBotsRank2,
            93 => Queue::SummonersRiftDoomBotsRank5,
            96 => Queue::CrystalScarAscensionDeprecated96,
            98 => Queue::TwistedTreeline6v6Hexakill,
            100 => Queue::ButchersBridge5v5Aram,
            300 => Queue::HowlingAbyssLegendOfThePoroKingDeprecated300,
            310 => Queue::SummonersRiftNemesis,
            313 => Queue::SummonersRiftBlackMarketBrawlers,
            315 => Queue::SummonersRiftNexusSiegeDeprecated315,
            317 => Queue::CrystalScarDefinitelyNotDominion,
            318 => Queue::SummonersRiftArurfDeprecated318,
            325 => Queue::SummonersRiftAllRandom,
            400 => Queue::SummonersRift5v5DraftPick,
            410 => Queue::SummonersRift5v5RankedDynamic,
            420 => Queue::SummonersRift5v5RankedSolo,
            430 => Queue::SummonersRift5v5BlindPick,
            440 => Queue::SummonersRift5v5RankedFlex,
            450 => Queue::HowlingAbyss5v5Aram,
            460 => Queue::TwistedTreeline3v3BlindPick,
            470 => Queue::TwistedTreeline3v3RankedFlexDeprecated470,
            490 => Queue::SummonersRiftNormalQuickplay,
            600 => Queue::SummonersRiftBloodHuntAssassin,
            610 => Queue::CosmicRuinsDarkStarSingularity,
            700 => Queue::SummonersRiftClash,
            720 => Queue::HowlingAbyssAramClash,
            800 => Queue::TwistedTreelineCoOpVsAiIntermediateBot,
            810 => Queue::TwistedTreelineCoOpVsAiIntroBot,
            820 => Queue::TwistedTreelineCoOpVsAiBeginnerBot,
            830 => Queue::SummonersRiftCoOpVsAiIntroBot,
            840 => Queue::SummonersRiftCoOpVsAiBeginnerBot,
            850 => Queue::SummonersRiftCoOpVsAiIntermediateBot,
            900 => Queue::SummonersRiftArurf,
            910 => Queue::CrystalScarAscension,
            920 => Queue::HowlingAbyssLegendOfThePoroKing,
            940 => Queue::SummonersRiftNexusSiege,
            950 => Queue::SummonersRiftDoomBotsVoting,
            960 => Queue::SummonersRiftDoomBotsStandard,
            980 => Queue::ValoranCityParkStarGuardianInvasionNormal,
            990 => Queue::ValoranCityParkStarGuardianInvasionOnslaught,
            1000 => Queue::OverchargeProjectHunters,
            1010 => Queue::SummonersRiftSnowArurf,
            1020 => Queue::SummonersRiftOneForAll,
            1030 => Queue::CrashSiteOdysseyExtractionIntro,
            1040 => Queue::CrashSiteOdysseyExtractionCadet,
            1050 => Queue::CrashSiteOdysseyExtractionCrewmember,
            1060 => Queue::CrashSiteOdysseyExtractionCaptain,
            1070 => Queue::CrashSiteOdysseyExtractionOnslaught,
            1090 => Queue::ConvergenceTeamfightTactics,
            1091 => Queue::ConvergenceTeamfightTactics1v0,
            1092 => Queue::ConvergenceTeamfightTactics2v0,
            1100 => Queue::ConvergenceRankedTeamfightTactics,
            1110 => Queue::ConvergenceTeamfightTacticsTutorial,
            1111 => Queue::ConvergenceTeamfightTacticsSimulation,
            1130 => Queue::ConvergenceRankedTeamfightTacticsHyperRoll,
            1150 => Queue::ConvergenceRankedTeamfightTacticsDoubleUpWorkshopDeprecated1150,
            1160 => Queue::ConvergenceRankedTeamfightTacticsDoubleUpWorkshop,
            1200 => Queue::NexusBlitzDeprecated1200,
            1210 => Queue::ConvergenceTeamfightTacticsChonccsTreasure,
            1300 => Queue::NexusBlitz,
            1400 => Queue::SummonersRiftUltimateSpellbook,
            1700 => Queue::Arena2v2v2v2Cherry,
            1710 => Queue::RingsOfWrathArenaCherryGames,
            1810 => Queue::SwarmSoloStrawberryGames,
            1820 => Queue::SwarmDuoStrawberryGames,
            1830 => Queue::SwarmTrioStrawberryGames,
            1840 => Queue::SwarmQuadStrawberryGames,
            1900 => Queue::SummonersRiftPickUrf,
            2000 => Queue::SummonersRiftTutorial1,
            2010 => Queue::SummonersRiftTutorial2,
            2020 => Queue::SummonersRiftTutorial3,
            6000 => Queue::ConvergenceTeamfightTacticsSet35Revival,
            _ => Queue::Custom
        }
    }
}