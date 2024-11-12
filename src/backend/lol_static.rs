use crate::backend::ssr::AppResult;
use crate::consts::champion::CHAMPION_OPTIONS;
use crate::consts::summoner_spell::{SummonerSpell, SUMMONER_SPELL_OPTIONS};
use futures::StreamExt;
use image::EncodableLayout;
use leptos::server_fn::serde::{Deserialize, Serialize};
use ravif::{Encoder, Img};
use reqwest;
use rgb::FromSlice;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

pub fn get_assets_path() -> PathBuf {
    Path::new("public").join("assets")
}
pub fn get_assets_dest_path() -> PathBuf {
    Path::new("target").join("site").join("assets")
}

#[derive(Debug, Clone)]
pub struct ImageToDownload {
    url: String,
    path: PathBuf,
    size: (u32, u32),
    to_avif: bool,
}


pub async fn init_static_data() -> AppResult<()> {
    let default_assets_path = get_assets_path();
    create_dir_all(default_assets_path.join("items"))?;
    create_dir_all(default_assets_path.join("profile_icons"))?;
    create_dir_all(default_assets_path.join("perks"))?;
    create_dir_all(default_assets_path.join("champions"))?;
    create_dir_all(default_assets_path.join("summoner_spells"))?;
    let version = get_current_version().await?;
    let t = std::time::Instant::now();
    let (
        item_images,
        profile_icons_images,
        perks,
    ) = tokio::join!(
        get_items(version.clone()),
        update_profile_icons_image(version.clone()),
        get_perks(version.clone())
    );

    let mut images_to_download = Vec::new();
    images_to_download.extend(item_images?);
    images_to_download.extend(profile_icons_images?);
    images_to_download.extend(perks?);
    for (id, _) in CHAMPION_OPTIONS {
        let path = get_assets_path().join("champions").join(format!("{}.avif", id));
        let path_dest = get_assets_dest_path().join("champions").join(format!("{}.avif", id));
        if !path_dest.exists() && !path.exists() {
            let image_url = format!(
                "https://cdn.communitydragon.org/{}/champion/{}/square",
                version.clone(),
                riven::consts::Champion::from(*id as i16).identifier().unwrap()
            );
            images_to_download.push(ImageToDownload {
                url: image_url,
                path: path.clone(),
                size: (60, 60),
                to_avif: true,
            });
        }
    }

    for summoner_spell in SUMMONER_SPELL_OPTIONS {
        if *summoner_spell == 0 {
            continue;
        }
        let path = get_assets_path().join("summoner_spells").join(format!("{}.avif", summoner_spell));
        let path_dest = get_assets_dest_path().join("summoner_spells").join(format!("{}.avif", summoner_spell));
        if !path_dest.exists() && !path.exists() {
            let url = format!(
                "https://ddragon.leagueoflegends.com/cdn/{}/img/spell/{}.png",
                version.clone(),
                SummonerSpell::from(*summoner_spell),
            );
            images_to_download.push(ImageToDownload {
                url,
                path,
                size: (22, 22),
                to_avif: true,
            });
        }
    }
    // Download and save all images concurrently
    for image in &images_to_download {
        println!("Downloading image: {:?} to {:?}", image.url, image.path);
    }
    println!("Downloading and saving {} images...", images_to_download.len());
    download_and_save_images(images_to_download).await?;

    println!("Time to load static data: {:?}", t.elapsed());
    Ok(())
}


pub async fn encode_and_save_image(image_data: &[u8], file_path: &Path, size: (u32, u32), to_avif: bool) -> AppResult<()> {
    println!("Saving image: {:?}", file_path);
    let img = image::load_from_memory_with_format(image_data, image::ImageFormat::Png).unwrap();
    let resized = img.resize_exact(size.0, size.1, image::imageops::FilterType::Lanczos3);
    //tokio::fs::create_dir_all(file_path.parent().unwrap()).await.unwrap();
    if to_avif {
        let result = Encoder::new()
            .with_quality(75.0)
            .with_speed(1)
            .encode_rgba(Img::new(resized.to_rgba8().as_bytes().as_rgba(), resized.width() as usize, resized.height() as usize))?;


        tokio::fs::write(file_path, result.avif_file).await?;
    } else {
        resized.save(file_path).unwrap();
    }
    Ok(())
}

async fn download_and_save_images(images_to_download: Vec<ImageToDownload>) -> AppResult<()> {
    let client = reqwest::Client::new();

    futures::stream::iter(images_to_download)
        .for_each_concurrent(10, |image| {
            let client = client.clone();
            async move {
                // Download the image
                match client.get(&image.url).send().await {
                    Ok(response) => {
                        let image_data = response.bytes().await.unwrap();
                        let image_data_vec = image_data.to_vec();
                        let path = image.path.clone();
                        let size = image.size;
                        let to_avif = image.to_avif;
                        encode_and_save_image(&image_data_vec, &path, size, to_avif).await.unwrap()
                    }
                    Err(e) => {
                        eprintln!("Error downloading image: {}", e);
                    }
                }
            }
        })
        .await;
    Ok(())
}


pub async fn get_perks(version: String) -> AppResult<Vec<ImageToDownload>> {
    let raw_perks = StaticUrl::Perks.get().await?;
    let all_perks: Vec<JsonPerk> = serde_json::from_str(raw_perks.as_str())?;
    let mut images_to_download = Vec::new();
    for perk in all_perks {
        let path = get_assets_path().join("perks").join(format!("{}.avif", perk.id));
        let path_dest = get_assets_dest_path().join("perks").join(format!("{}.avif", perk.id));
        if !path_dest.exists() && !path.exists() {
            let image_url = format!("https://raw.communitydragon.org/latest/game/assets/perks/{}", perk.icon_path.replace("/lol-game-data/assets/v1/perk-images/", "").to_lowercase());
            images_to_download.push(ImageToDownload {
                url: image_url,
                path,
                size: (22, 22),
                to_avif: true,
            });
        }
    }
    let main_perks = StaticUrl::Perks2 { version }.get().await?;
    let main_perks: Vec<JsonPerk2> = serde_json::from_str(main_perks.as_str())?;
    for perk in main_perks {
        let path = get_assets_path().join("perks").join(format!("{}.avif", perk.id));
        let path_dest = get_assets_dest_path().join("perks").join(format!("{}.avif", perk.id));
        if !path.exists() && !path_dest.exists() {
            let image_url = format!("https://ddragon.leagueoflegends.com/cdn/img/{}", perk.icon);
            images_to_download.push(ImageToDownload {
                url: image_url,
                path,
                size: (22, 22),
                to_avif: true,
            });
        }
    }
    Ok(images_to_download)
}

pub async fn get_items(version: String) -> AppResult<Vec<ImageToDownload>> {
    let mut images_to_download = Vec::new();
    let raw_items = StaticUrl::Items { version: version.clone() }.get().await?;
    let items_json: ItemData = serde_json::from_str(raw_items.as_str())?;
    for (id, _) in items_json.data {
        let path = get_assets_path().join("items").join(format!("{}.avif", id));
        let path_dest = get_assets_dest_path().join("items").join(format!("{}.avif", id));
        if !path_dest.exists() && !path.exists() {
            let image_url = format!(
                "https://ddragon.leagueoflegends.com/cdn/{}/img/item/{}.png",
                version.clone(),
                id
            );
            images_to_download.push(ImageToDownload {
                url: image_url,
                path: path.clone(),
                size: (22, 22),
                to_avif: true,
            });
        }
    }
    Ok(images_to_download)
}


pub async fn get_current_version() -> AppResult<String> {
    let versions: Vec<String> = serde_json::from_str(StaticUrl::Versions.get().await?.as_str())?;
    Ok(versions[0].clone())
}


pub async fn update_profile_icons_image(version: String) -> AppResult<Vec<ImageToDownload>> {
    let mut images_to_download = Vec::new();
    let raw_champions = StaticUrl::ProfileIcons { version: version.clone() }.get().await?;
    let data: ChampionData = serde_json::from_str(raw_champions.as_str())?;
    for (id, _) in data.data {
        let path = get_assets_path().join("profile_icons").join(format!("{}.avif", id));
        let path_dest = get_assets_dest_path().join("profile_icons").join(format!("{}.avif", id));

        if !path_dest.exists() && !path.exists() {
            let image_url = format!(
                "https://ddragon.leagueoflegends.com/cdn/{}/img/profileicon/{}.png",
                version.clone(),
                id
            );
            images_to_download.push(ImageToDownload {
                url: image_url,
                path: path.clone(),
                size: (64, 64),
                to_avif: true,
            });
        }
    }
    Ok(images_to_download)
}


pub enum StaticUrl {
    Versions,
    Champions { version: String },
    Items { version: String },
    SummonerSpells { version: String },
    Perks,
    Perks2 { version: String },
    ProfileIcons { version: String },
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
            StaticUrl::Perks => "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perks.json".to_string(),
            StaticUrl::Perks2 { version } => format!("https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/runesReforged.json", version),
            StaticUrl::ProfileIcons { version } => format!("https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/profileicon.json", version).to_string(),
            StaticUrl::Maps => "https://static.developer.riotgames.com/docs/lol/maps.json".to_string(),
            StaticUrl::Queues => "https://static.developer.riotgames.com/docs/lol/queues.json".to_string(),
            StaticUrl::Modes => "https://static.developer.riotgames.com/docs/lol/gameModes.json".to_string(),
        }
    }

    pub async fn get(&self) -> AppResult<String> {
        let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build()?;
        client.get(self.url().as_str()).send().await?.text().await.map_err(|e| e.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChampionData {
    data: HashMap<i32, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemData {
    data: HashMap<i32, Value>,
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
    pub description: String,
    pub tags: Vec<String>,
    pub into_items: Vec<i32>,
    pub from_items: Vec<i32>,
    pub gold: ItemGoldInfo,
    pub stats: Value,
    pub depth: i32,
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
pub struct JsonItem {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub image: JsonImage,
    pub stats: Value,
    pub gold: ItemGoldInfo,
    pub into: Option<Vec<String>>,
    pub from: Option<Vec<String>>,
    pub maps: HashMap<i32, bool>,
    pub depth: Option<i32>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SummonerIconJson {
    pub id: i32,
    pub img_url: String,
}


#[derive(Serialize, Deserialize)]
struct JsonPerk {
    id: i32,
    #[serde(rename = "iconPath")]
    icon_path: String,
}

#[derive(Serialize, Deserialize)]
struct JsonPerk2 {
    id: i32,
    icon: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileIconJson {
    pub id: i32,
}
