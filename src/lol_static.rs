use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::num::ParseIntError;
use once_cell::sync::OnceCell;
use riven::endpoints::ChampionV3;
use crate::lol_static;

pub static VERSION: OnceCell<String> = OnceCell::new();
pub static CHAMPIONS: OnceCell<HashMap<String, Champion>> = OnceCell::new();
pub static ITEMS: OnceCell<HashMap<i32, Item>> = OnceCell::new();
pub static SUMMONER_SPELLS: OnceCell<HashMap<i32, SummonerSpell>> = OnceCell::new();

pub static PERKS: OnceCell<HashMap<i32, Perk>> = OnceCell::new();
pub static MAPS: OnceCell<HashMap<i32, Map>> = OnceCell::new();
pub static QUEUES: OnceCell<HashMap<i32, Queue>> = OnceCell::new();
pub static GAME_MODES: OnceCell<HashMap<String, GameMode>> = OnceCell::new();


pub async fn init_static_data(){
    let version = get_current_version().await.unwrap();
    let t = std::time::Instant::now();
    let (
        champions,
        items,
        summoner_spells,
        perks,
        maps,
        queues,
        game_modes,
    ) = tokio::join!(
    get_champions(version.clone()),
    get_items(version.clone()),
    get_summoner_spells(version.clone()),
    get_perks(version.clone()),
    get_maps(),
    get_queues(),
    get_game_modes(),
);
    VERSION.set(version.clone()).unwrap();
    CHAMPIONS.set(champions.unwrap()).unwrap();
    ITEMS.set(items.unwrap()).unwrap();
    SUMMONER_SPELLS.set(summoner_spells.unwrap()).unwrap();
    PERKS.set(perks.unwrap()).unwrap();
    MAPS.set(maps.unwrap()).unwrap();
    QUEUES.set(queues.unwrap()).unwrap();
    GAME_MODES.set(game_modes.unwrap()).unwrap();
    println!("Time to load static data: {:?}", t.elapsed());
}

pub fn get_version() -> String {
    VERSION.get().unwrap().clone()
}

pub fn get_champion_by_name(name:&str) -> Champion{
    match CHAMPIONS.get().unwrap().get(name){
        None => {
            let champions = CHAMPIONS.get().unwrap();
            for (k, v) in champions.iter(){
                // check if k match name
                if name.contains(k) || name.to_lowercase() == v.name.to_lowercase(){
                    return v.clone();
                }
            }
            panic!("Champion: {} not found", name);
        }
        Some(c) => c.clone(),
    }
}

pub fn get_champion_by_id(id:i32) -> Champion{
    let champions = CHAMPIONS.get().unwrap();
    for (_, v) in champions.iter(){
        if v.id == id{
            return v.clone();
        }
    }
    panic!("Champion with id: {} not found", id);
}




pub fn get_item(id:i32) -> Option<Item>{
    if id == 0{
        return None;
    }
    ITEMS.get().unwrap().get(&id).cloned()
}

pub fn get_summoner_spell(id:i32) -> Option<SummonerSpell>{
    if id == 0{
        return None;
    }
    SUMMONER_SPELLS.get().unwrap().get(&id).cloned()
}

pub fn get_perk(id:i32) -> Option<Perk>{
    if id == 0{
        return None;
    }
    PERKS.get().unwrap().get(&id).cloned()
}

pub fn get_map(id:i32) -> Map{
    MAPS.get().unwrap().get(&id).unwrap().clone()
}

pub fn get_queue(id:i32) -> Queue{
    QUEUES.get().unwrap().get(&id).unwrap().clone()
}

pub fn get_game_mode(name:&str) -> GameMode{
    GAME_MODES.get().unwrap().get(name).unwrap().clone()
}











pub async fn get_current_version() -> Result<String, reqwest::Error> {
    let mut versions: Vec<String> = serde_json::from_value(StaticUrl::Versions.get().await?).unwrap();
    Ok(versions[0].clone())
}

pub async fn get_champions(version:String) -> Result<HashMap<String, Champion>, reqwest::Error> {
    let mut champions = HashMap::new();
    let raw_champions = StaticUrl::Champions { version:version.clone() }.get().await?;
    let champions_json = raw_champions["data"].as_object().unwrap();
    for (key, value) in champions_json {
        let champion: JsonChampion = serde_json::from_value(value.clone()).unwrap();
        let id = champion.key.parse::<i32>().unwrap();
        champions.insert(key.clone(), Champion {
            id,
            name: champion.name,
            info: champion.info,
            tags: champion.tags,
            img_url: format!("https://ddragon.leagueoflegends.com/cdn/{}/img/champion/{}", version.clone(), champion.image.full),
            stats: champion.stats,
            slug: champion.id,
        });
    }
    Ok(champions)
}


pub async fn get_items(version:String) -> Result<HashMap<i32, Item>, reqwest::Error> {
    let mut items = HashMap::new();
    let raw_items = StaticUrl::Items { version : version.clone()}.get().await?;
    let items_json = raw_items["data"].as_object().unwrap();
    for (key, value) in items_json {
        let item: JsonItem = serde_json::from_value(value.clone()).unwrap();
        let id = key.parse::<i32>().unwrap();
        items.insert(id, Item {
            id,
            name: item.name,
            description: item.description,
            tags: item.tags,
            into_items: item.into.unwrap_or_default().into_iter().map(|e| e.parse::<i32>().unwrap()).collect::<Vec<_>>(),
            from_items: item.from.unwrap_or_default().into_iter().map(|e| e.parse::<i32>().unwrap()).collect::<Vec<_>>(),
            depth: item.depth.unwrap_or_default(),
            img_url: format!("https://ddragon.leagueoflegends.com/cdn/{}/img/item/{}", version.clone(), item.image.full),
            stats: item.stats,
            gold: item.gold,
        });
    }
    Ok(items)
}

pub async fn get_summoner_spells(version:String) -> Result<HashMap<i32, SummonerSpell>, reqwest::Error> {
    let mut summoner_spells = HashMap::new();
    let raw_summoner_spells = StaticUrl::SummonerSpells { version: version.clone()}.get().await?;
    let summoner_spells_json = raw_summoner_spells["data"].as_object().unwrap();
    for (_, value) in summoner_spells_json {
        let summoner_spell: JsonSummonerSpell = serde_json::from_value(value.clone()).unwrap();
        let id = summoner_spell.key.parse::<i32>().unwrap();
        summoner_spells.insert(id, SummonerSpell {
            id,
            name: summoner_spell.name,
            description: summoner_spell.description,
            img_url: format!("https://ddragon.leagueoflegends.com/cdn/{}/img/spell/{}", version.clone(), summoner_spell.image.full),
        });
    }
    Ok(summoner_spells)
}
pub async  fn get_perks(version:String) -> Result<HashMap<i32, Perk>, reqwest::Error> {
    let mut perks = HashMap::new();
    let raw_perks = StaticUrl::Perks { version : version.clone()}.get().await?;
    let main_perks: Vec<JsonParentPerk> = serde_json::from_value(raw_perks).unwrap();
    for main_perk in main_perks {
        for slot in main_perk.slots {
            for rune in slot.runes {
                perks.insert(rune.id, Perk {
                    id: rune.id,
                    name: rune.name,
                    img_url: format!("https://ddragon.leagueoflegends.com/cdn/img/{}", rune.icon),
                });
            }
        }
        perks.insert(main_perk.id, Perk {
            id: main_perk.id,
            name: main_perk.name,
            img_url: format!("https://ddragon.leagueoflegends.com/cdn/img/{}", main_perk.icon),
        });
    }
    Ok(perks)
}

pub async fn get_maps() -> Result<HashMap<i32, Map>, reqwest::Error> {
    let mut maps = HashMap::new();
    let raw_maps = StaticUrl::Maps.get().await?;
    let maps_json: Vec<JsonMap> = serde_json::from_value(raw_maps).unwrap();
    for map in maps_json {
        maps.insert(map.map_id, Map {
            id: map.map_id,
            name: map.map_name,
            description: map.notes,
        });
    }
    Ok(maps)
}


pub async fn get_queues() -> Result<HashMap<i32, Queue>, reqwest::Error> {
    let mut queues = HashMap::new();
    let raw_queues = StaticUrl::Queues.get().await?;
    let queues_json: Vec<JsonQueue> = serde_json::from_value(raw_queues).unwrap();
    for queue in queues_json {
        queues.insert(queue.queue_id, Queue {
            id: queue.queue_id,
            name: queue.map,
            description: Some(queue.description.clone().unwrap_or_default().replace(" games", "")),
            notes: queue.notes,
        });
    }
    Ok(queues)
}

pub async fn get_game_modes() -> Result<HashMap<String, GameMode>, reqwest::Error> {
    let mut modes = HashMap::new();
    let raw_modes = StaticUrl::Modes.get().await?;
    let modes_json: Vec<JsonMode> = serde_json::from_value(raw_modes).unwrap();
    for mode in modes_json {
        modes.insert(mode.game_mode.clone(), GameMode {
            name: mode.game_mode,
            description: mode.description,
        });
    }
    Ok(modes)
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SummonerIcon{
    pub id: i32,
    pub img_url: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMode {
    pub name: String,
    pub description: String,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Perk {
    pub id: i32,
    pub name: String,
    pub img_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SummonerSpell {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub img_url: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionInfo {
    pub attack: i32,
    pub defense: i32,
    pub magic: i32,
    pub difficulty: i32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionStats {
    pub hp: i32,
    #[serde(rename = "hpperlevel")]
    pub hp_per_level: i32,
    pub mp: i32,
    #[serde(rename = "mpperlevel")]
    pub mp_per_level: f32,
    #[serde(rename = "movespeed")]
    pub move_speed: i32,
    pub armor: i32,
    #[serde(rename = "armorperlevel")]
    pub armor_per_level: f32,
    #[serde(rename = "spellblock")]
    pub spell_block: i32,
    #[serde(rename = "spellblockperlevel")]
    pub spell_block_per_level: f32,
    #[serde(rename = "attackrange")]
    pub attack_range: i32,
    #[serde(rename = "hpregen")]
    pub hp_regen: f32,
    #[serde(rename = "hpregenperlevel")]
    pub hp_regen_per_level: f32,
    #[serde(rename = "mpregen")]
    pub mp_regen: f32,
    #[serde(rename = "mpregenperlevel")]
    pub mp_regen_per_level: f32,
    pub crit: i32,
    #[serde(rename = "critperlevel")]
    pub crit_per_level: i32,
    #[serde(rename = "attackdamage")]
    pub attack_damage: i32,
    #[serde(rename = "attackdamageperlevel")]
    pub attack_damage_per_level: f32,
    #[serde(rename = "attackspeedperlevel")]
    pub attack_speed_per_level: f32,
    #[serde(rename = "attackspeed")]
    pub attack_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Champion {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub img_url: String,
    pub info: ChampionInfo,
    pub tags: Vec<String>,
    pub stats: ChampionStats,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemGoldInfo {
    pub base: i32,
    pub total: i32,
    pub sell: i32,
    pub purchasable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub img_url: String,
    pub description: String,
    pub tags: Vec<String>,
    pub into_items: Vec<i32>,
    pub from_items: Vec<i32>,
    pub gold: ItemGoldInfo,
    pub stats: serde_json::Value,
    pub depth: i32,
}

pub enum StaticUrl {
    Versions,
    Champions { version: String },
    Items { version: String },
    SummonerSpells { version: String },
    Perks { version: String },
    Maps,
    Queues,
    Modes,
}

impl StaticUrl {
    pub fn url(&self) -> String {
        match self {
            StaticUrl::Versions => "https://ddragon.leagueoflegends.com/api/versions.json".to_string(),
            StaticUrl::Champions { version } => format!("https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/champion.json", version),
            StaticUrl::Items { version } => format!("https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/item.json", version),
            StaticUrl::SummonerSpells { version } => format!("https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/summoner.json", version),
            StaticUrl::Perks { version } => format!("https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/runesReforged.json", version),
            StaticUrl::Maps => "https://static.developer.riotgames.com/docs/lol/maps.json".to_string(),
            StaticUrl::Queues => "https://static.developer.riotgames.com/docs/lol/queues.json".to_string(),
            StaticUrl::Modes => "https://static.developer.riotgames.com/docs/lol/gameModes.json".to_string(),
        }
    }

    pub async fn get(&self) -> Result<Value, reqwest::Error> {
        let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap();
        client.get(self.url().as_str()).send().await?.json().await
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonImage {
    pub full: String,
    pub sprite: String,
    pub group: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}


#[derive(Serialize, Deserialize)]
pub struct JsonChampion {
    pub id: String,
    pub key: String,
    pub name: String,
    pub title: String,
    pub info: ChampionInfo,
    pub tags: Vec<String>,
    pub image: JsonImage,
    pub stats: ChampionStats,
}


#[derive(Serialize, Deserialize)]
pub struct JsonItem {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub image: JsonImage,
    pub stats: Value,
    pub gold: ItemGoldInfo,
    pub into: Option<Vec<String>>,
    pub from: Option<Vec<String>>,
    pub depth: Option<i32>,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonMap {
    pub map_id: i32,
    pub map_name: String,
    pub notes: String,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonMode {
    pub game_mode: String,
    pub description: String,
}


#[derive(Serialize, Deserialize)]
struct JsonParentPerk {
    id: i32,
    key: String,
    name: String,
    icon: String,
    slots: Vec<JsonPerkSlot>,
}

#[derive(Serialize, Deserialize)]
struct JsonPerkSlot {
    runes: Vec<JsonChildPerk>,
}

#[derive(Serialize, Deserialize)]
struct JsonChildPerk {
    id: i32,
    key: String,
    icon: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonQueue {
    pub queue_id: i32,
    pub map: String,
    pub description: Option<String>,
    pub notes: Option<String>,
}


#[derive(Serialize, Deserialize)]
pub struct JsonSummonerSpell {
    pub id: String,
    pub key: String,
    pub name: String,
    pub description: String,
    pub image: JsonImage,
}

