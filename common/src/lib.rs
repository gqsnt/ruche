pub mod consts;

pub enum AssetType{
    Item,
    ProfileIcon,
    SummonerSpell,
    Perk,
    Champion
}

impl AssetType {

    pub fn get_class_name(&self, id: i32) -> String {
        format!("{}-{}", self.get_default_class_name(), id)
    }


    pub fn get_path(&self) -> &'static str {
        match self {
            AssetType::Item => "items",
            AssetType::ProfileIcon => "profile_icons",
            AssetType::SummonerSpell => "summoner_spells",
            AssetType::Perk => "perks",
            AssetType::Champion => "champions"
        }
    }

    pub fn get_default_class_name(&self) -> &'static str{
        match self {
            AssetType::Item => "ii",
            AssetType::ProfileIcon => "pi",
            AssetType::SummonerSpell => "ss",
            AssetType::Perk => "pk",
            AssetType::Champion => "cn"
        }
    }


    pub fn default_size(&self) -> (u32,u32) {
        match self {
            AssetType::Item => (22,22),
            AssetType::ProfileIcon => (64,64),
            AssetType::SummonerSpell => (22,22),
            AssetType::Perk => (28,28),
            AssetType::Champion => (48,48)
        }
    }
}