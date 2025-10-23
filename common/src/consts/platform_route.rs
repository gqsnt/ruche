use bitcode::{Decode, Encode};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString, IntoStaticStr};

pub const PLATFORM_ROUTE_OPTIONS: [PlatformRoute; 16] = [
    PlatformRoute::BR,
    PlatformRoute::EUNE,
    PlatformRoute::EUW,
    PlatformRoute::JP,
    PlatformRoute::KR,
    PlatformRoute::LAN,
    PlatformRoute::LAS,
    PlatformRoute::MENA,
    PlatformRoute::NA,
    PlatformRoute::OCE,
    PlatformRoute::PH,
    PlatformRoute::SG,
    PlatformRoute::TH,
    PlatformRoute::TR,
    PlatformRoute::TW,
    PlatformRoute::VN,
];

#[derive(
    PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode, Clone, Copy, Debug, Default,
    Serialize, Deserialize,
    IntoPrimitive, TryFromPrimitive, EnumIter, EnumString, IntoStaticStr
)]
#[repr(u8)]
pub enum PlatformRoute {
    /// Europe, West. (default)
    #[default]
    #[strum(serialize = "EUW", serialize = "EUW1", to_string = "EUW")]
    EUW = 18,

    /// Brazil.
    #[strum(serialize = "BR", serialize = "BR1", to_string = "BR")]
    BR = 16,

    /// Europe, Northeast.
    #[strum(serialize = "EUNE", serialize = "EUN1", to_string = "EUNE")]
    EUNE = 17,

    /// Japan.
    #[strum(serialize = "JP", serialize = "JP1", to_string = "JP")]
    JP = 19,

    /// Korea.
    #[strum(serialize = "KR", to_string = "KR")]
    KR = 20,

    /// Latin America, North.
    #[strum(serialize = "LAN", serialize = "LA1", to_string = "LAN")]
    LAN = 21,

    /// Latin America, South.
    #[strum(serialize = "LAS", serialize = "LA2", to_string = "LAS")]
    LAS = 22,

    /// Middle East and North Africa.
    #[strum(serialize = "MENA", serialize = "ME1", to_string = "MENA")]
    MENA = 37,

    /// North America.
    #[strum(serialize = "NA", serialize = "NA1", to_string = "NA")]
    NA = 23,

    /// Oceania.
    #[strum(serialize = "OCE", serialize = "OC1", to_string = "OCE")]
    OCE = 24,

    /// Philippines.
    #[strum(serialize = "PH", serialize = "PH2", to_string = "PH")]
    PH = 32,

    /// Russia.
    #[strum(serialize = "RU", to_string = "RU")]
    RU = 25,

    /// Singapore.
    #[strum(serialize = "SG", serialize = "SG2", to_string = "SG")]
    SG = 33,

    /// Thailand.
    #[strum(serialize = "TH", serialize = "TH2", to_string = "TH")]
    TH = 34,

    /// Turkey.
    #[strum(serialize = "TR", serialize = "TR1", to_string = "TR")]
    TR = 26,

    /// Taiwan.
    #[strum(serialize = "TW", serialize = "TW2", to_string = "TW")]
    TW = 35,

    /// Vietnam.
    #[strum(serialize = "VN", serialize = "VN2", to_string = "VN")]
    VN = 36,

    /// Public Beta Environment.
    #[strum(serialize = "PBE", serialize = "PBE1", to_string = "PBE")]
    PBE = 31,
}

impl PlatformRoute {
    #[inline]
    pub fn id(self) -> u8 { self.into() }

    /// Short region code ("EUW", "NA", …). Uses IntoStaticStr (returns the FIRST `serialize`).
    #[inline]
    pub fn code(self) -> &'static str { self.into() }

    /// Long API code ("EUW1", "NA1", …) for external services.
    #[inline]
    pub const fn api_code(self) -> &'static str {
        match self {
            PlatformRoute::EUW => "EUW1",
            PlatformRoute::BR  => "BR1",
            PlatformRoute::EUNE=> "EUN1",
            PlatformRoute::JP  => "JP1",
            PlatformRoute::KR  => "KR",
            PlatformRoute::LAN => "LA1",
            PlatformRoute::LAS => "LA2",
            PlatformRoute::MENA=> "ME1",
            PlatformRoute::NA  => "NA1",
            PlatformRoute::OCE => "OC1",
            PlatformRoute::PH  => "PH2",
            PlatformRoute::RU  => "RU",
            PlatformRoute::SG  => "SG2",
            PlatformRoute::TH  => "TH2",
            PlatformRoute::TR  => "TR1",
            PlatformRoute::TW  => "TW2",
            PlatformRoute::VN  => "VN2",
            PlatformRoute::PBE => "PBE1",
        }
    }

    /// Accept both "EUW" and "EUW1" (and friends), via `EnumString`.
    #[inline]
    pub fn from_code(code: &str) -> Option<Self> {
        code.parse().ok()
    }

    #[cfg(feature = "ssr")]
    pub fn to_riven(&self) -> riven::consts::PlatformRoute {
        use std::str::FromStr;
        riven::consts::PlatformRoute::from_str(self.api_code())
            .expect("known platform route")
    }
}

// impl From<&str> for PlatformRoute {
//     fn from(s: &str) -> Self {
//         PlatformRoute::from_code(s).unwrap_or(PlatformRoute::EUW)
//     }
// }

impl std::fmt::Display for PlatformRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}
