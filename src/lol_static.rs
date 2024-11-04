use crate::consts::{Champion, SummonerSpell};
use futures::StreamExt;
use image::DynamicImage;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use strum::IntoEnumIterator;
use webp::Encoder;


pub fn get_assets_path() -> std::path::PathBuf {
    let ex_path = std::env::current_exe().unwrap();
    let path = ex_path.parent().unwrap();
    let target_path = path.join("target");
    if target_path.exists() {
        target_path
    }else{
        path.to_path_buf()
    }.join("site").join("assets")
}

#[derive(Debug, Clone)]
pub struct ImageToDownload {
    url: String,
    path: PathBuf,
    size: (u32, u32),
    to_webp: bool,
}


pub async fn init_static_data() {
    let version = get_current_version().await.unwrap();
    let t = std::time::Instant::now();
    create_dir_all(get_assets_path()).unwrap();
    create_dir_all(get_assets_path().join("items")).unwrap();
    create_dir_all(get_assets_path().join("profile_icons")).unwrap();
    create_dir_all(get_assets_path().join("perks")).unwrap();
    create_dir_all(get_assets_path().join("champions")).unwrap();
    create_dir_all(get_assets_path().join("summoner_spells")).unwrap();
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
    images_to_download.extend(item_images.unwrap());
    images_to_download.extend(profile_icons_images.unwrap());
    images_to_download.extend(perks.unwrap());
    for champion in Champion::iter() {
        if champion == Champion::UNKNOWN {
            continue;
        }
        let path = get_assets_path().join("champions").join(format!("{}.webp", champion as i16));
        if !path.exists() {
            let image_url = format!(
                "https://cdn.communitydragon.org/{}/champion/{}/square",
                version.clone(),
                riven::consts::Champion::from(champion as i16).identifier().unwrap()
            );
            images_to_download.push(ImageToDownload {
                url: image_url,
                path: path.clone(),
                size: (60, 60),
                to_webp: true,
            });
        }
    }

    for summoner_spell in SummonerSpell::iter() {
        if summoner_spell == SummonerSpell::UNKNOWN {
            continue;
        }
        let path = get_assets_path().join("summoner_spells").join(format!("{}.webp", summoner_spell as u16));
        if !path.exists() {
            images_to_download.push(ImageToDownload {
                url: summoner_spell.get_url(version.clone()),
                path,
                size: (22, 22),
                to_webp: true,
            });
        }
    }
    // Download and save all images concurrently
    for image in &images_to_download {
        println!("Downloading image: {:?} to {:?}", image.url, image.path);
    }
    println!("Downloading and saving {} images...", images_to_download.len());
    download_and_save_images(images_to_download).await;

    println!("Time to load static data: {:?}", t.elapsed());
}


pub async fn encode_and_save_image(image_data: &[u8], file_path: &Path, size: (u32, u32), to_webp: bool) {
    println!("Saving image: {:?}", file_path);
    let img = image::load_from_memory_with_format(image_data, image::ImageFormat::Png).unwrap();
    let resized = img.resize_exact(size.0, size.1, image::imageops::FilterType::Lanczos3);
    //tokio::fs::create_dir_all(file_path.parent().unwrap()).await.unwrap();
    if to_webp {
        let rgb8 = DynamicImage::ImageRgb8(resized.to_rgb8());
        let encoder = Encoder::from_image(&rgb8).unwrap();

        tokio::fs::write(file_path, encoder.encode(100.0).to_vec()).await.unwrap();
    } else {
        resized.save(file_path).unwrap();
    }
}

async fn download_and_save_images(images_to_download: Vec<ImageToDownload>) {
    let client = reqwest::Client::new();

    futures::stream::iter(images_to_download)
        .for_each_concurrent(10, |image| {
            let client = client.clone();
            async move {
                // Download the image
                match client.get(&image.url).send().await {
                    Ok(response) => {
                        match response.bytes().await {
                            Ok(image_data) => {
                                // Save the image in a blocking task
                                let image_data_vec = image_data.to_vec();
                                let path = image.path.clone();
                                let size = image.size;
                                let to_webp = image.to_webp;
                                encode_and_save_image(&image_data_vec, &path, size, to_webp).await;
                            }
                            Err(e) => {
                                eprintln!("Error getting image data: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error downloading image: {}", e);
                    }
                }
            }
        })
        .await;
}


pub async fn get_perks(version: String) -> Result<Vec<ImageToDownload>, reqwest::Error> {
    let raw_perks = StaticUrl::Perks { version: version.clone() }.get().await?;
    let main_perks: Vec<JsonParentPerk> = serde_json::from_value(raw_perks).unwrap();
    let mut images_to_download = Vec::new();
    for main_perk in main_perks {
        for slot in main_perk.slots {
            for rune in slot.runes {
                let path = get_assets_path().join("perks").join(format!("{}.png", rune.id));
                if !path.exists() {
                    let image_url = format!("https://ddragon.leagueoflegends.com/cdn/img/{}", rune.icon);
                    images_to_download.push(ImageToDownload {
                        url: image_url,
                        path,
                        size: (22, 22),
                        to_webp: false,
                    });
                }
            }
        }
        let path = get_assets_path().join("perks").join(format!("{}.png", main_perk.id));
        if !path.exists() {
            let image_url = format!("https://ddragon.leagueoflegends.com/cdn/img/{}", main_perk.icon);
            images_to_download.push(ImageToDownload {
                url: image_url,
                path,
                size: (22, 22),
                to_webp: false,
            });
        }
    }
    Ok(images_to_download)
}

pub async fn get_items(version: String) -> Result<Vec<ImageToDownload>, reqwest::Error> {
    let mut images_to_download = Vec::new();
    let raw_items = StaticUrl::Items { version: version.clone() }.get().await?;
    let items_json = raw_items["data"].as_object().unwrap();
    for (key, value) in items_json {
        let item: JsonItem = serde_json::from_value(value.clone()).unwrap();
        let id = key.parse::<i32>().unwrap();
        let path = get_assets_path().join("items").join(format!("{}.webp", id));
        if !path.exists() {
            let image_url = format!(
                "https://ddragon.leagueoflegends.com/cdn/{}/img/item/{}",
                version.clone(),
                item.image.full
            );
            images_to_download.push(ImageToDownload {
                url: image_url,
                path: path.clone(),
                size: (22, 22),
                to_webp: true,
            });
        }
    }
    Ok(images_to_download)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileIcon {
    pub id: i32,
}


pub async fn get_current_version() -> Result<String, reqwest::Error> {
    let versions: Vec<String> = serde_json::from_value(StaticUrl::Versions.get().await?).unwrap();
    Ok(versions[0].clone())
}


pub async fn update_profile_icons_image(version: String) -> Result<Vec<ImageToDownload>, reqwest::Error> {
    let mut images_to_download = Vec::new();
    let raw_champions = StaticUrl::ProfileIcons { version: version.clone() }.get().await?;
    let data = raw_champions["data"].as_object().unwrap();
    for (k, _) in data {
        let id = k.clone().parse::<i64>().unwrap() as i32;
        let path = get_assets_path().join("profile_icons").join(format!("{}.webp", id));
        if !path.exists() {
            let image_url = format!(
                "https://ddragon.leagueoflegends.com/cdn/{}/img/profileicon/{}.png",
                version.clone(),
                id
            );
            images_to_download.push(ImageToDownload {
                url: image_url,
                path: path.clone(),
                size: (64, 64),
                to_webp: true,
            });
        }
    }
    Ok(images_to_download)
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SummonerIcon {
    pub id: i32,
    pub img_url: String,
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


pub enum StaticUrl {
    Versions,
    Champions { version: String },
    Items { version: String },
    SummonerSpells { version: String },
    Perks { version: String },
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
            StaticUrl::Perks { version } => format!("https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/runesReforged.json", version),
            StaticUrl::ProfileIcons { version } => format!("https://ddragon.leagueoflegends.com/cdn/{}/data/en_US/profileicon.json", version).to_string(),
            StaticUrl::Maps => "https://static.developer.riotgames.com/docs/lol/maps.json".to_string(),
            StaticUrl::Queues => "https://static.developer.riotgames.com/docs/lol/queues.json".to_string(),
            StaticUrl::Modes => "https://static.developer.riotgames.com/docs/lol/gameModes.json".to_string(),
        }
    }

    pub async fn get(&self) -> Result<Value, reqwest::Error> {
        let client = reqwest::Client::builder().danger_accept_invalid_certs(true).build()?;
        client.get(self.url().as_str()).send().await?.json().await
    }
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






