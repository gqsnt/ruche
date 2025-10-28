use bitcode::{Decode, Encode};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use strum::{EnumIter, IntoStaticStr};


#[repr(u16)]
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Hash, Encode, Decode,
    IntoPrimitive, TryFromPrimitive, EnumIter, IntoStaticStr
)]
#[derive(Default)]
pub enum Perk {
    #[strum(serialize = "UNKNOWN")]
    #[default]
    UNKNOWN = 0,

    #[strum(serialize = "StatsHealScaling")]         StatsHealScaling = 5001,
    #[strum(serialize = "StatsArmor")]               StatsArmor = 5002,
    #[strum(serialize = "StatsMagicResist")]         StatsMagicResist = 5003,
    #[strum(serialize = "StatsAttackSpeed")]         StatsAttackSpeed = 5005,
    #[strum(serialize = "StatsAbilityHaste")]        StatsAbilityHaste = 5007,
    #[strum(serialize = "StatsAdaptiveForce")]       StatsAdaptiveForce = 5008,
    #[strum(serialize = "StatsMovementSpeed")]       StatsMovementSpeed = 5010,
    #[strum(serialize = "StatsHealth")]              StatsHealth = 5011,
    #[strum(serialize = "StatsResistScaling")]       StatsResistScaling = 5012,
    #[strum(serialize = "StatsTenacitySlowResist")]  StatsTenacitySlowResist = 5013,

    #[strum(serialize = "Domination")]               Domination = 8100,
    #[strum(serialize = "Electrocute")]              Electrocute = 8112,
    #[strum(serialize = "DarkHarvest")]              DarkHarvest = 8128,
    #[strum(serialize = "HailOfBlades")]             HailOfBlades = 9923,
    #[strum(serialize = "CheapShot")]                CheapShot = 8126,
    #[strum(serialize = "TasteOfBlood")]             TasteOfBlood = 8139,
    #[strum(serialize = "SuddenImpact")]             SuddenImpact = 8143,
    #[strum(serialize = "ZombieWard")]               ZombieWard = 8136,
    #[strum(serialize = "GhostPoro")]                GhostPoro = 8120,
    #[strum(serialize = "EyeballCollection")]        EyeballCollection = 8138,
    #[strum(serialize = "RavenousHunter")]           RavenousHunter = 8135,
    #[strum(serialize = "IngeniousHunter")]          IngeniousHunter = 8134,
    #[strum(serialize = "RelentlessHunter")]         RelentlessHunter = 8105,
    #[strum(serialize = "UltimateHunter")]           UltimateHunter = 8106,

    #[strum(serialize = "Inspiration")]              Inspiration = 8300,
    #[strum(serialize = "GlacialAugment")]           GlacialAugment = 8351,
    #[strum(serialize = "UnsealedSpellbook")]        UnsealedSpellbook = 8360,
    #[strum(serialize = "FirstStrike")]              FirstStrike = 8369,
    #[strum(serialize = "HextechFlashtraption")]     HextechFlashtraption = 8306,
    #[strum(serialize = "MagicalFootwear")]          MagicalFootwear = 8304,
    #[strum(serialize = "CashBack")]                 CashBack = 8321,
    #[strum(serialize = "PerfectTiming")]            PerfectTiming = 8313,
    #[strum(serialize = "TimeWarpTonic")]            TimeWarpTonic = 8352,
    #[strum(serialize = "BiscuitDelivery")]          BiscuitDelivery = 8345,
    #[strum(serialize = "CosmicInsight")]            CosmicInsight = 8347,
    #[strum(serialize = "ApproachVelocity")]         ApproachVelocity = 8410,
    #[strum(serialize = "JackOfAllTrades")]          JackOfAllTrades = 8316,

    #[strum(serialize = "Precision")]                Precision = 8000,
    #[strum(serialize = "PressTheAttack")]           PressTheAttack = 8005,
    #[strum(serialize = "LethalTempo")]              LethalTempo = 8008,
    #[strum(serialize = "FleetFootwork")]            FleetFootwork = 8021,
    #[strum(serialize = "Conqueror")]                Conqueror = 8010,
    #[strum(serialize = "AbsorbLife")]               AbsorbLife = 9101,
    #[strum(serialize = "Triumph")]                  Triumph = 9111,
    #[strum(serialize = "PresenceOfMind")]           PresenceOfMind = 8009,
    #[strum(serialize = "LegendAlacrity")]           LegendAlacrity = 9104,
    #[strum(serialize = "LegendHaste")]              LegendHaste = 9105,
    #[strum(serialize = "LegendBloodline")]          LegendBloodline = 9103,
    #[strum(serialize = "CoupDeGrace")]              CoupDeGrace = 8014,
    #[strum(serialize = "CutDown")]                  CutDown = 8017,
    #[strum(serialize = "LastStand")]                LastStand = 8299,

    #[strum(serialize = "Resolve")]                  Resolve = 8400,
    #[strum(serialize = "GraspOfTheUndying")]        GraspOfTheUndying = 8437,
    #[strum(serialize = "Aftershock")]               Aftershock = 8439,
    #[strum(serialize = "Guardian")]                 Guardian = 8465,
    #[strum(serialize = "Demolish")]                 Demolish = 8446,
    #[strum(serialize = "FontOfLife")]               FontOfLife = 8463,
    #[strum(serialize = "ShieldBash")]               ShieldBash = 8401,
    #[strum(serialize = "Conditioning")]             Conditioning = 8429,
    #[strum(serialize = "SecondWind")]               SecondWind = 8444,
    #[strum(serialize = "BonePlating")]              BonePlating = 8473,
    #[strum(serialize = "Overgrowth")]               Overgrowth = 8451,
    #[strum(serialize = "Revitalize")]               Revitalize = 8453,
    #[strum(serialize = "Unflinching")]              Unflinching = 8242,

    #[strum(serialize = "Sorcery")]                  Sorcery = 8200,
    #[strum(serialize = "SummonAery")]               SummonAery = 8214,
    #[strum(serialize = "ArcaneComet")]              ArcaneComet = 8229,
    #[strum(serialize = "PhaseRush")]                PhaseRush = 8230,
    #[strum(serialize = "NullifyingOrb")]            NullifyingOrb = 8224,
    #[strum(serialize = "ManaflowBand")]             ManaflowBand = 8226,
    #[strum(serialize = "NimbusCloack")]             NimbusCloack = 8275,
    #[strum(serialize = "Transcendence")]            Transcendence = 8210,
    #[strum(serialize = "Celerity")]                 Celerity = 8234,
    #[strum(serialize = "AbsoluteFocus")]            AbsoluteFocus = 8233,
    #[strum(serialize = "Scorch")]                   Scorch = 8237,
    #[strum(serialize = "Waterwalking")]             Waterwalking = 8232,
    #[strum(serialize = "GatheringStorm")]           GatheringStorm = 8236,
}


impl Perk {
    #[inline]
    pub fn id(self) -> u16 { self.into() }

    #[inline]
    pub fn label(self) -> &'static str { self.into() }
}
