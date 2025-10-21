use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy , Encode,Decode, Eq, PartialEq, Hash)]
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
    TheBandlewood=35
}

impl Map {
    pub const fn get_static_name(&self) -> &'static str {
        match self {
            Map::SummonersRiftOriginalSummerVariant => "Summoner's Rift Original Summer Variant",
            Map::SummonersRiftOriginalAutumnVariant => "Summoner's Rift Original Autumn Variant",
            Map::TheProvingGrounds => "The Proving Grounds",
            Map::TwistedTreelineOriginalVersion => "Twisted Treeline Original Version",
            Map::TheCrystalScar => "The Crystal Scar",
            Map::TwistedTreeline => "Twisted Treeline",
            Map::SummonersRift => "Summoner's Rift",
            Map::HowlingAbyss => "Howling Abyss",
            Map::ButchersBridge => "Butcher's Bridge",
            Map::CosmicsRuin => "Cosmic Ruins",
            Map::ValoranCityPark => "Valoran City Park",
            Map::Substructure43 => "Substructure 43",
            Map::CrashSite => "Crash Site",
            Map::NexusBlitz => "Nexus Blitz",
            Map::Convergence => "Convergence",
            Map::Arena => "Arena",
            Map::Swarm => "Swarm",
            Map::TheBandlewood => "The Bandle wood"
        }
    }
}

impl From<u8> for Map {
    fn from(value: u8) -> Self {
        match value {
            1 => Map::SummonersRiftOriginalSummerVariant,
            2 => Map::SummonersRiftOriginalAutumnVariant,
            3 => Map::TheProvingGrounds,
            4 => Map::TwistedTreelineOriginalVersion,
            8 => Map::TheCrystalScar,
            10 => Map::TwistedTreeline,
            11 => Map::SummonersRift,
            12 => Map::HowlingAbyss,
            14 => Map::ButchersBridge,
            16 => Map::CosmicsRuin,
            18 => Map::ValoranCityPark,
            19 => Map::Substructure43,
            20 => Map::CrashSite,
            21 => Map::NexusBlitz,
            22 => Map::Convergence,
            30 => Map::Arena,
            33 => Map::Swarm,
            35 => Map::TheBandlewood,
            _ => panic!("Invalid map id"),
        }
    }
}
