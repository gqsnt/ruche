use bitcode::{Decode, Encode};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString, IntoStaticStr};

pub const PLATFORM_ROUTE_OPTIONS: [PlatformRoute; 16] = [
    PlatformRoute::BR1,
    PlatformRoute::EUN1,
    PlatformRoute::EUW1,
    PlatformRoute::JP1,
    PlatformRoute::KR,
    PlatformRoute::LA1,
    PlatformRoute::LA2,
    PlatformRoute::ME1,
    PlatformRoute::NA1,
    PlatformRoute::OC1,
    PlatformRoute::PH2,
    PlatformRoute::SG2,
    PlatformRoute::TH2,
    PlatformRoute::TR1,
    PlatformRoute::TW2,
    PlatformRoute::VN2,
];

#[derive(
    PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode, Clone, Copy, Debug, Default,
    Serialize, Deserialize,
    IntoPrimitive, TryFromPrimitive, EnumIter, EnumString, IntoStaticStr
)]
#[repr(u8)]
#[non_exhaustive]
pub enum PlatformRoute {
    /// Europe, West. (default)
    #[default]
    #[strum(serialize = "EUW", serialize = "EUW1")]
    EUW1 = 18,

    /// Brazil.
    #[strum(serialize = "BR", serialize = "BR1")]
    BR1 = 16,
    /// Europe, Northeast.
    #[strum(serialize = "EUNE", serialize = "EUN1")]
    EUN1 = 17,
    /// Japan.
    #[strum(serialize = "JP", serialize = "JP1")]
    JP1 = 19,
    /// Korea.
    #[strum(serialize = "KR")]
    KR = 20,
    /// Latin America, North.
    #[strum(serialize = "LAN", serialize = "LA1")]
    LA1 = 21,
    /// Latin America, South.
    #[strum(serialize = "LAS", serialize = "LA2")]
    LA2 = 22,
    /// Middle East and North Africa.
    #[strum(serialize = "MENA", serialize = "ME1")]
    ME1 = 37,
    /// North America.
    #[strum(serialize = "NA", serialize = "NA1")]
    NA1 = 23,
    /// Oceania.
    #[strum(serialize = "OCE", serialize = "OC1")]
    OC1 = 24,
    /// Philippines.
    #[strum(serialize = "PH", serialize = "PH2")]
    PH2 = 32,
    /// Russia.
    #[strum(serialize = "RU")]
    RU = 25,
    /// Singapore.
    #[strum(serialize = "SG", serialize = "SG2")]
    SG2 = 33,
    /// Thailand.
    #[strum(serialize = "TH", serialize = "TH2")]
    TH2 = 34,
    /// Turkey.
    #[strum(serialize = "TR", serialize = "TR1")]
    TR1 = 26,
    /// Taiwan.
    #[strum(serialize = "TW", serialize = "TW2")]
    TW2 = 35,
    /// Vietnam.
    #[strum(serialize = "VN", serialize = "VN2")]
    VN2 = 36,
    /// Public Beta Environment.
    #[strum(serialize = "PBE", serialize = "PBE1")]
    PBE1 = 31,
}

impl PlatformRoute {
    #[inline]
    pub fn id(self) -> u8 { self.into() }

    /// Short region code (`"EUW"`, `"NA"`, …). Uses `IntoStaticStr`.
    #[inline]
    pub fn code(self) -> &'static str { self.into() }

    /// Accept both `"EUW"` and `"EUW1"` (and friends).
    #[inline]
    pub fn from_code(code: &str) -> Option<Self> {
        <Self as std::str::FromStr>::from_str(code).ok()
    }

    #[cfg(feature = "ssr")]
    pub fn to_riven(&self) -> riven::consts::PlatformRoute {
        use std::str::FromStr;
        riven::consts::PlatformRoute::from_str((*self).into()).unwrap_or(riven::consts::PlatformRoute::EUW1)
    }
}


impl std::fmt::Display for PlatformRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}
