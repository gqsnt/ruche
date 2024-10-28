use serde::{Deserialize, Serialize};

/// Represents the different regions.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize
)]
pub enum RegionType {
    AMERICAS,
    ASIA,
    EUROPE,
    SEA,
}

#[cfg(feature = "ssr")]
impl RegionType {
    /// Converts to `riven::consts::RegionalRoute`.
    pub fn to_riven(self) -> riven::consts::RegionalRoute {
        match self {
            RegionType::AMERICAS => riven::consts::RegionalRoute::AMERICAS,
            RegionType::ASIA => riven::consts::RegionalRoute::ASIA,
            RegionType::EUROPE => riven::consts::RegionalRoute::EUROPE,
            RegionType::SEA => riven::consts::RegionalRoute::SEA,
        }
    }
}

/// Represents the different platforms.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize
)]
pub enum PlatformType {
    BR1,
    EUN1,
    EUW1,
    JP1,
    KR,
    LA1,
    LA2,
    NA1,
    OC1,
    TR1,
    RU,
    PH2,
    SG2,
    TH2,
    TW2,
    VN2,
    ME1,
    PBE1,
}

impl PlatformType {
    pub fn cases()->&'static[PlatformType]{
        &[
            PlatformType::BR1,
            PlatformType::EUN1,
            PlatformType::EUW1,
            PlatformType::JP1,
            PlatformType::KR,
            PlatformType::LA1,
            PlatformType::LA2,
            PlatformType::NA1,
            PlatformType::OC1,
            PlatformType::TR1,
            PlatformType::RU,
            PlatformType::PH2,
            PlatformType::SG2,
            PlatformType::TH2,
            PlatformType::TW2,
            PlatformType::VN2,
            PlatformType::ME1,
            PlatformType::PBE1,
        ]
    }


    /// Determines the `RegionType` for the platform.
    pub fn region(self) -> RegionType {
        match self {
            PlatformType::BR1 | PlatformType::LA1 | PlatformType::LA2 | PlatformType::NA1 | PlatformType::PBE1 => {
                RegionType::AMERICAS
            }
            PlatformType::JP1 | PlatformType::KR => RegionType::ASIA,
            PlatformType::EUN1 | PlatformType::EUW1 | PlatformType::TR1 | PlatformType::RU | PlatformType::ME1 => {
                RegionType::EUROPE
            }
            _ => RegionType::SEA,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "BR1" => Some(PlatformType::BR1),
            "EUN1" => Some(PlatformType::EUN1),
            "EUW1" => Some(PlatformType::EUW1),
            "JP1" => Some(PlatformType::JP1),
            "KR" => Some(PlatformType::KR),
            "LA1" => Some(PlatformType::LA1),
            "LA2" => Some(PlatformType::LA2),
            "NA1" => Some(PlatformType::NA1),
            "OC1" => Some(PlatformType::OC1),
            "TR1" => Some(PlatformType::TR1),
            "RU" => Some(PlatformType::RU),
            "PH2" => Some(PlatformType::PH2),
            "SG2" => Some(PlatformType::SG2),
            "TH2" => Some(PlatformType::TH2),
            "TW2" => Some(PlatformType::TW2),
            "VN2" => Some(PlatformType::VN2),
            "ME1" => Some(PlatformType::ME1),
            "PBE1" => Some(PlatformType::PBE1),
            _ => None,
        }
    }


    /// Converts from a short code string.
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_uppercase().as_str() {
            "BR" => Some(PlatformType::BR1),
            "EUN" => Some(PlatformType::EUN1),
            "EUW" => Some(PlatformType::EUW1),
            "JP" => Some(PlatformType::JP1),
            "KR" => Some(PlatformType::KR),
            "LA1" => Some(PlatformType::LA1),
            "LA2" => Some(PlatformType::LA2),
            "NA" => Some(PlatformType::NA1),
            "OC" => Some(PlatformType::OC1),
            "TR" => Some(PlatformType::TR1),
            "RU" => Some(PlatformType::RU),
            "PH" => Some(PlatformType::PH2),
            "SG" => Some(PlatformType::SG2),
            "TH" => Some(PlatformType::TH2),
            "TW" => Some(PlatformType::TW2),
            "VN" => Some(PlatformType::VN2),
            "ME" => Some(PlatformType::ME1),
            "PBE" => Some(PlatformType::PBE1),
            _ => None,
        }
    }

    /// Gets the code used in URLs.
    pub fn code(self) -> &'static str {
        match self {
            PlatformType::BR1 => "BR",
            PlatformType::EUN1 => "EUN",
            PlatformType::EUW1 => "EUW",
            PlatformType::JP1 => "JP",
            PlatformType::KR => "KR",
            PlatformType::LA1 => "LA1",
            PlatformType::LA2 => "LA2",
            PlatformType::NA1 => "NA",
            PlatformType::OC1 => "OC",
            PlatformType::TR1 => "TR",
            PlatformType::RU => "RU",
            PlatformType::PH2 => "PH",
            PlatformType::SG2 => "SG",
            PlatformType::TH2 => "TH",
            PlatformType::TW2 => "TW",
            PlatformType::VN2 => "VN",
            PlatformType::ME1 => "ME",
            PlatformType::PBE1 => "PBE",
        }
    }

    #[cfg(feature = "ssr")]
    /// Converts to `riven::consts::PlatformRoute`.
    pub fn to_riven(self) -> riven::consts::PlatformRoute {
        match self {
            PlatformType::BR1 => riven::consts::PlatformRoute::BR1,
            PlatformType::EUN1 => riven::consts::PlatformRoute::EUN1,
            PlatformType::EUW1 => riven::consts::PlatformRoute::EUW1,
            PlatformType::JP1 => riven::consts::PlatformRoute::JP1,
            PlatformType::KR => riven::consts::PlatformRoute::KR,
            PlatformType::LA1 => riven::consts::PlatformRoute::LA1,
            PlatformType::LA2 => riven::consts::PlatformRoute::LA2,
            PlatformType::NA1 => riven::consts::PlatformRoute::NA1,
            PlatformType::OC1 => riven::consts::PlatformRoute::OC1,
            PlatformType::TR1 => riven::consts::PlatformRoute::TR1,
            PlatformType::RU => riven::consts::PlatformRoute::RU,
            PlatformType::PH2 => riven::consts::PlatformRoute::PH2,
            PlatformType::SG2 => riven::consts::PlatformRoute::SG2,
            PlatformType::TH2 => riven::consts::PlatformRoute::TH2,
            PlatformType::TW2 => riven::consts::PlatformRoute::TW2,
            PlatformType::VN2 => riven::consts::PlatformRoute::VN2,
            PlatformType::ME1 => riven::consts::PlatformRoute::ME1,
            PlatformType::PBE1 => riven::consts::PlatformRoute::PBE1,
        }
    }
}

impl std::fmt::Display for PlatformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}