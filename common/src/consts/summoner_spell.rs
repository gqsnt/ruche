use std::fmt::Formatter;

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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

impl From<u16> for SummonerSpell {
    fn from(value: u16) -> Self {
        match value {
            0 => SummonerSpell::UNKNOWN,
            21 => SummonerSpell::SummonerBarrier,
            1 => SummonerSpell::SummonerBoost,
            2202 => SummonerSpell::SummonerCherryFlash,
            2201 => SummonerSpell::SummonerCherryHold,
            14 => SummonerSpell::SummonerDot,
            3 => SummonerSpell::SummonerExhaust,
            4 => SummonerSpell::SummonerFlash,
            6 => SummonerSpell::SummonerHaste,
            7 => SummonerSpell::SummonerHeal,
            13 => SummonerSpell::SummonerMana,
            30 => SummonerSpell::SummonerPoroRecall,
            31 => SummonerSpell::SummonerPoroThrow,
            11 => SummonerSpell::SummonerSmite,
            39 => SummonerSpell::SummonerSnowURFSnowball_Mark,
            32 => SummonerSpell::SummonerSnowball,
            12 => SummonerSpell::SummonerTeleport,
            54 => SummonerSpell::Summoner_UltBookPlaceholder,
            55 => SummonerSpell::Summoner_UltBookSmitePlaceholder,
            _ => SummonerSpell::UNKNOWN,
        }
    }
}

pub static SUMMONER_SPELL_OPTIONS: &[u16] = &[
    SummonerSpell::SummonerBarrier as u16,
    SummonerSpell::SummonerBoost as u16,
    SummonerSpell::SummonerCherryFlash as u16,
    SummonerSpell::SummonerCherryHold as u16,
    SummonerSpell::SummonerDot as u16,
    SummonerSpell::SummonerExhaust as u16,
    SummonerSpell::SummonerFlash as u16,
    SummonerSpell::SummonerHaste as u16,
    SummonerSpell::SummonerHeal as u16,
    SummonerSpell::SummonerMana as u16,
    SummonerSpell::SummonerPoroRecall as u16,
    SummonerSpell::SummonerPoroThrow as u16,
    SummonerSpell::SummonerSmite as u16,
    SummonerSpell::SummonerSnowURFSnowball_Mark as u16,
    SummonerSpell::SummonerSnowball as u16,
    SummonerSpell::SummonerTeleport as u16,
    SummonerSpell::Summoner_UltBookPlaceholder as u16,
    SummonerSpell::Summoner_UltBookSmitePlaceholder as u16,
];

impl std::fmt::Display for SummonerSpell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SummonerSpell::UNKNOWN => "UNKNOWN",
            SummonerSpell::SummonerBarrier => "SummonerBarrier",
            SummonerSpell::SummonerBoost => "SummonerBoost",
            SummonerSpell::SummonerCherryFlash => "SummonerCherryFlash",
            SummonerSpell::SummonerCherryHold => "SummonerCherryHold",
            SummonerSpell::SummonerDot => "SummonerDot",
            SummonerSpell::SummonerExhaust => "SummonerExhaust",
            SummonerSpell::SummonerFlash => "SummonerFlash",
            SummonerSpell::SummonerHaste => "SummonerHaste",
            SummonerSpell::SummonerHeal => "SummonerHeal",
            SummonerSpell::SummonerMana => "SummonerMana",
            SummonerSpell::SummonerPoroRecall => "SummonerPoroRecall",
            SummonerSpell::SummonerPoroThrow => "SummonerPoroThrow",
            SummonerSpell::SummonerSmite => "SummonerSmite",
            SummonerSpell::SummonerSnowURFSnowball_Mark => "SummonerSnowURFSnowball_Mark",
            SummonerSpell::SummonerSnowball => "SummonerSnowURFSnowball_Mark",
            SummonerSpell::SummonerTeleport => "SummonerSnowball",
            SummonerSpell::Summoner_UltBookPlaceholder => "SummonerTeleport",
            SummonerSpell::Summoner_UltBookSmitePlaceholder => "Summoner_UltBookSmitePlaceholder",
        };
        write!(f, "{}", str)
    }
}
