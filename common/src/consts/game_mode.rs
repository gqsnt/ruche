use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Encode,Decode)]
#[repr(u8)]
pub enum GameMode {
    /// Catch-all variant for new, unknown game modes.
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
    BRAWL,
}
