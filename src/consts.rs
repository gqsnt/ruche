pub mod champion;
pub mod game_mode;
pub mod item;
pub mod map;
pub mod perk;
pub mod platform_route;
pub mod profile_icon;
pub mod queue;
pub mod summoner_spell;

pub trait HasStaticAsset {
    const PATH: &'static str;
    fn get_static_asset_url(&self) -> String {
        format!("/assets/{}/{}.avif", Self::PATH, self.get_id())
    }

    fn get_id(&self) -> i32 ;

}

impl HasStaticAsset for item::Item {
    const PATH: &'static str = "items";

    fn get_id(&self) -> i32  {
       self.0 as i32
    }
}

impl HasStaticAsset for profile_icon::ProfileIcon {
    const PATH: &'static str = "profile_icons";

    fn get_id(&self) -> i32 {
        self.0  as i32
    }
}

impl HasStaticAsset for summoner_spell::SummonerSpell {
    const PATH: &'static str = "summoner_spells";

    fn get_id(&self) -> i32 {
        (*self as u16) as i32
    }
}

impl HasStaticAsset for perk::Perk {
    const PATH: &'static str = "perks";

    fn get_id(&self) -> i32 {
        (*self as u16) as i32
    }
}

impl HasStaticAsset for champion::Champion {
    const PATH: &'static str = "champions";

    fn get_id(&self) -> i32 {
        (*self as u16) as i32
    }
}
