use crate::AssetType;

pub mod champion;
pub mod item;
pub mod map;
pub mod perk;
pub mod platform_route;
pub mod profile_icon;
pub mod queue;
pub mod summoner_spell;

pub trait HasStaticSrcAsset {
    const ASSET_TYPE: AssetType;
    fn get_static_asset_url(&self) -> String {
        format!("/assets/{}/{}.avif", self.get_path(), self.get_id())
    }

    fn get_path(&self) -> &'static str {
       Self::ASSET_TYPE.get_path()
    }

    fn get_id(&self) -> i32;
}

impl HasStaticSrcAsset for profile_icon::ProfileIcon {
    const ASSET_TYPE: AssetType = AssetType::ProfileIcon;

    fn get_id(&self) -> i32 {
        self.0 as i32
    }
}



pub trait HasStaticBgAsset {
    const ASSET_TYPE: AssetType;

    fn get_class_name(&self) -> String {
        Self::ASSET_TYPE.get_class_name(self.get_id())
    }

    fn get_id(&self) -> i32;
}

impl HasStaticBgAsset for item::Item {
     const ASSET_TYPE: AssetType = AssetType::Item;

    fn get_id(&self) -> i32 {
        self.0 as i32
    }
}



impl HasStaticBgAsset for summoner_spell::SummonerSpell {
     const ASSET_TYPE: AssetType = AssetType::SummonerSpell;

    fn get_id(&self) -> i32 {
        (*self as u16) as i32
    }
}

impl HasStaticBgAsset for perk::Perk {
     const ASSET_TYPE: AssetType = AssetType::Perk;

    fn get_id(&self) -> i32 {
        (*self as u16) as i32
    }
}

impl HasStaticBgAsset for champion::Champion {
     const ASSET_TYPE: AssetType = AssetType::Champion;

    fn get_id(&self) -> i32 {
        (*self as u16) as i32
    }
}
