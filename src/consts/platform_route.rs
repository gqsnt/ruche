use serde::{Deserialize, Serialize};

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


/// Platform routes for League of Legends (LoL), Teamfight Tactics (TFT), and Legends of Runeterra (LoR).
#[derive(Debug)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[derive(Clone, Copy)]
#[repr(u8)]
#[non_exhaustive]
pub enum PlatformRoute {
    /// Brazil.
    ///
    /// `16` (riotapi-schema ID/repr)
    BR1 = 16,

    /// Europe, Northeast.
    ///
    /// `17` (riotapi-schema ID/repr)
    EUN1 = 17,

    /// Europe, West.
    ///
    /// `18` (riotapi-schema ID/repr)
    EUW1 = 18,

    /// Japan.
    ///
    /// `19` (riotapi-schema ID/repr)
    JP1 = 19,

    /// Korea.
    ///
    /// `20` (riotapi-schema ID/repr)
    KR = 20,

    /// Latin America, North.
    ///
    /// `21` (riotapi-schema ID/repr)
    LA1 = 21,

    /// Latin America, South.
    ///
    /// `22` (riotapi-schema ID/repr)
    LA2 = 22,

    /// Middle East and North Africa.
    ///
    /// `37` (riotapi-schema ID/repr)
    ME1 = 37,

    /// North America.
    ///
    /// `23` (riotapi-schema ID/repr)
    NA1 = 23,

    /// Oceania.
    ///
    /// `24` (riotapi-schema ID/repr)
    OC1 = 24,

    /// Philippines
    ///
    /// `32` (riotapi-schema ID/repr)
    PH2 = 32,

    /// Russia
    ///
    /// `25` (riotapi-schema ID/repr)
    RU = 25,

    /// Singapore
    ///
    /// `33` (riotapi-schema ID/repr)
    SG2 = 33,

    /// Thailand
    ///
    /// `34` (riotapi-schema ID/repr)
    TH2 = 34,

    /// Turkey
    ///
    /// `26` (riotapi-schema ID/repr)
    TR1 = 26,

    /// Taiwan
    ///
    /// `35` (riotapi-schema ID/repr)
    TW2 = 35,

    /// Vietnam
    ///
    /// `36` (riotapi-schema ID/repr)
    VN2 = 36,

    /// Public Beta Environment, special beta testing platform. Located in North America.
    ///
    /// `31` (riotapi-schema ID/repr)
    PBE1 = 31,
}

impl PlatformRoute {
    #[cfg(feature = "ssr")]
    pub fn to_riven(&self) -> riven::consts::PlatformRoute {
        use std::str::FromStr;
        riven::consts::PlatformRoute::from_str(self.to_string().as_str()).unwrap()
    }

    #[cfg(feature = "ssr")]
    pub fn from_raw_str(value:&str) -> Self{
        match value{
            "BR1" => PlatformRoute::BR1,
            "EUN1" => PlatformRoute::EUN1,
            "EUW1" => PlatformRoute::EUW1,
            "JP1" => PlatformRoute::JP1,
            "KR" => PlatformRoute::KR,
            "LA1" => PlatformRoute::LA1,
            "LA2" => PlatformRoute::LA2,
            "ME1" => PlatformRoute::ME1,
            "NA1" => PlatformRoute::NA1,
            "OC1" => PlatformRoute::OC1,
            "PH2" => PlatformRoute::PH2,
            "RU" => PlatformRoute::RU,
            "SG2" => PlatformRoute::SG2,
            "TH2" => PlatformRoute::TH2,
            "TR1" => PlatformRoute::TR1,
            "TW2" => PlatformRoute::TW2,
            "VN2" => PlatformRoute::VN2,
            "PBE1" => PlatformRoute::PBE1,
            _ => PlatformRoute::EUW1,
        }
    }

}

impl From<&str> for PlatformRoute {
    fn from(value: &str) -> Self {
        match value {
            "BR" => PlatformRoute::BR1,
            "EUNE" => PlatformRoute::EUN1,
            "EUW" => PlatformRoute::EUW1,
            "JP" => PlatformRoute::JP1,
            "KR" => PlatformRoute::KR,
            "LAN" => PlatformRoute::LA1,
            "LAS" => PlatformRoute::LA2,
            "MENA" => PlatformRoute::ME1,
            "NA" => PlatformRoute::NA1,
            "OCE" => PlatformRoute::OC1,
            "PH" => PlatformRoute::PH2,
            "RU" => PlatformRoute::RU,
            "SG" => PlatformRoute::SG2,
            "TH" => PlatformRoute::TH2,
            "TR" => PlatformRoute::TR1,
            "TW" => PlatformRoute::TW2,
            "VN" => PlatformRoute::VN2,
            "PBE" => PlatformRoute::PBE1,
            _ => PlatformRoute::EUW1,
        }
    }
}

impl std::fmt::Display for PlatformRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            PlatformRoute::BR1 => "BR",
            PlatformRoute::EUN1 => "EUNE",
            PlatformRoute::EUW1 => "EUW",
            PlatformRoute::JP1 => "JP",
            PlatformRoute::KR => "KR",
            PlatformRoute::LA1 => "LAN",
            PlatformRoute::LA2 => "LAS",
            PlatformRoute::ME1 => "MENA",
            PlatformRoute::NA1 => "NA",
            PlatformRoute::OC1 => "OCE",
            PlatformRoute::PH2 => "PH",
            PlatformRoute::RU => "RU",
            PlatformRoute::SG2 => "SG",
            PlatformRoute::TH2 => "TH",
            PlatformRoute::TR1 => "TR",
            PlatformRoute::TW2 => "TW",
            PlatformRoute::VN2 => "VN",
            PlatformRoute::PBE1 => "PBE",
        })
    }
}

