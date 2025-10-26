use bitcode::{Decode, Encode};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::{AsRefStr, Display, EnumIter, IntoEnumIterator, IntoStaticStr};

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Hash, Encode, Decode,
    IntoPrimitive, TryFromPrimitive, EnumIter, AsRefStr, Display, IntoStaticStr
)]
#[repr(u8)]
pub enum Map {
    UNKNOWN=0,
    #[strum(serialize = "Summoner's Rift Original Summer Variant")]
    SummonersRiftOriginalSummerVariant = 1,
    #[strum(serialize = "Summoner's Rift Original Autumn Variant")]
    SummonersRiftOriginalAutumnVariant = 2,
    #[strum(serialize = "The Proving Grounds")]
    TheProvingGrounds = 3,
    #[strum(serialize = "Twisted Treeline Original Version")]
    TwistedTreelineOriginalVersion = 4,
    #[strum(serialize = "The Crystal Scar")]
    TheCrystalScar = 8,
    #[strum(serialize = "Twisted Treeline")]
    TwistedTreeline = 10,
    #[strum(serialize = "Summoner's Rift")]
    SummonersRift = 11,
    #[strum(serialize = "Howling Abyss")]
    HowlingAbyss = 12,
    #[strum(serialize = "Butcher's Bridge")]
    ButchersBridge = 14,
    #[strum(serialize = "Cosmic Ruins")]
    CosmicsRuin = 16,
    #[strum(serialize = "Valoran City Park")]
    ValoranCityPark = 18,
    #[strum(serialize = "Substructure 43")]
    Substructure43 = 19,
    #[strum(serialize = "Crash Site")]
    CrashSite = 20,
    #[strum(serialize = "Nexus Blitz")]
    NexusBlitz = 21,
    #[strum(serialize = "Convergence")]
    Convergence = 22,
    #[strum(serialize = "Arena")]
    Arena = 30,
    #[strum(serialize = "Swarm")]
    Swarm = 33,
    #[strum(serialize = "The Bandle wood")]
    TheBandlewood = 35,
}

impl Map {
    #[inline]
    pub fn id(self) -> u8 { self.into() }

    #[inline]
    pub fn label(self) -> &'static str { self.into() }

    #[inline]
    pub fn from_id(id: u8) -> Option<Self> { Self::try_from(id).ok() }

    #[inline]
    pub fn options_all() -> Vec<(u8, &'static str)> {
        Map::iter().map(|m| (m.id(), m.label())).collect()
    }
}
