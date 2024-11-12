pub mod champion;
pub mod platform_route;
pub mod item;
pub mod map;
pub mod profile_icon;
pub mod queue;
pub mod game_mode;
pub mod summoner_spell;
pub mod perk;


pub trait HasStaticAsset {
    const PATH: &'static str;
    fn get_static_asset_url(id: u16) -> String {
        format!("/assets/{}/{}.avif", Self::PATH, id)
    }
}


impl HasStaticAsset for item::Item {
    const PATH: &'static str = "items";
}


impl HasStaticAsset for profile_icon::ProfileIcon {
    const PATH: &'static str = "profile_icons";
}


impl HasStaticAsset for summoner_spell::SummonerSpell {
    const PATH: &'static str = "summoner_spells";
}


impl HasStaticAsset for perk::Perk {
    const PATH: &'static str = "perks";
}

impl HasStaticAsset for champion::Champion {
    const PATH: &'static str = "champions";
}


