use bitcode::{Decode, Encode};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

#[repr(u16)]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Hash, Encode, Decode,
    IntoPrimitive, TryFromPrimitive, EnumIter, IntoStaticStr
)]
#[allow(non_camel_case_types)]
pub enum SummonerSpell {
    #[num_enum(default)]
    #[strum(serialize = "UNKNOWN")]
    UNKNOWN = 0,

    #[strum(serialize = "SummonerBarrier")]                 SummonerBarrier = 21,
    #[strum(serialize = "SummonerBoost")]                   SummonerBoost = 1,
    #[strum(serialize = "SummonerCherryFlash")]             SummonerCherryFlash = 2202,
    #[strum(serialize = "SummonerCherryHold")]              SummonerCherryHold = 2201,
    #[strum(serialize = "SummonerDot")]                     SummonerDot = 14,
    #[strum(serialize = "SummonerExhaust")]                 SummonerExhaust = 3,
    #[strum(serialize = "SummonerFlash")]                   SummonerFlash = 4,
    #[strum(serialize = "SummonerHaste")]                   SummonerHaste = 6,
    #[strum(serialize = "SummonerHeal")]                    SummonerHeal = 7,
    #[strum(serialize = "SummonerMana")]                    SummonerMana = 13,
    #[strum(serialize = "SummonerPoroRecall")]              SummonerPoroRecall = 30,
    #[strum(serialize = "SummonerPoroThrow")]               SummonerPoroThrow = 31,
    #[strum(serialize = "SummonerSmite")]                   SummonerSmite = 11,
    #[strum(serialize = "SummonerSnowURFSnowball_Mark")]    SummonerSnowURFSnowball_Mark = 39,
    #[strum(serialize = "SummonerSnowball")]                SummonerSnowball = 32,
    #[strum(serialize = "SummonerTeleport")]                SummonerTeleport = 12,
    #[strum(serialize = "Summoner_UltBookPlaceholder")]     Summoner_UltBookPlaceholder = 54,
    #[strum(serialize = "Summoner_UltBookSmitePlaceholder")]Summoner_UltBookSmitePlaceholder = 55,
}

impl SummonerSpell {
    #[inline]
    pub fn id(self) -> u16 { self.into() }

    #[inline]
    pub fn label(self) -> &'static str { self.into() }

    /// Replacement for the old `SUMMONER_SPELL_OPTIONS`.
    #[inline]
    pub fn ids_non_unknown() -> Vec<u16> {
        SummonerSpell::iter()
            .filter(|s| *s != SummonerSpell::UNKNOWN)
            .map(|s| s.id())
            .collect()
    }
}

pub static SUMMONER_SPELL_OPTIONS: Lazy<Vec<u16>> =
    Lazy::new(|| SummonerSpell::ids_non_unknown());
