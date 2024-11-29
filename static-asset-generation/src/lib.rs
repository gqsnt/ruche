use clap::Parser;
use common::consts::champion::CHAMPION_OPTIONS;
use common::consts::summoner_spell::{SummonerSpell, SUMMONER_SPELL_OPTIONS};
use common::AssetType;
use futures::{future, StreamExt};
use image::{DynamicImage, EncodableLayout, ImageFormat};
use ravif::{Encoder, Img};
use rgb::FromSlice;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use image::imageops::FilterType;
use thiserror::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    ///  force rebuild summoner_spells sprite and css
    #[arg(long, default_value_t = false)]
    pub summoner_spells: bool,

    /// force rebuild items sprite and css
    #[arg(long, default_value_t = false)]
    pub items: bool,

    /// force rebuild profile_icons
    #[arg(long, default_value_t = false)]
    pub profile_icons: bool,

    /// force rebuild perks sprite and css
    #[arg(long, default_value_t = false)]
    pub perks: bool,

    /// force rebuild champions sprite and css
    #[arg(long, default_value_t = false)]
    pub champions: bool,

    /// force rebuild logo
    #[arg(long, default_value_t = false)]
    pub logo: bool,
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,
    #[error("Riven Error: {0}")]
    RivenError(Arc<riven::RiotApiError>),
    #[error("Ravif: {0}")]
    RavifError(Arc<ravif::Error>),
    #[error("Reqwest Error: {0}")]
    ReqwestError(Arc<reqwest::Error>),
    #[error("Serde json Error: {0}")]
    SerdeJsonError(Arc<serde_json::Error>),
    #[error("Custom Error: {0}")]
    CustomError(String),
    #[error("Std Io Error: {0}")]
    StdIoError(Arc<std::io::Error>),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::StdIoError(Arc::new(e))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::SerdeJsonError(Arc::new(e))
    }
}

impl From<riven::RiotApiError> for AppError {
    fn from(e: riven::RiotApiError) -> Self {
        AppError::RivenError(Arc::new(e))
    }
}

impl From<ravif::Error> for AppError {
    fn from(e: ravif::Error) -> Self {
        AppError::RavifError(Arc::new(e))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::ReqwestError(Arc::new(e))
    }
}

pub fn get_assets_path() -> PathBuf {
    Path::new("broken-gg").join("public").join("assets")
}

pub fn get_css_path() -> PathBuf {
    Path::new("broken-gg").join("style")
}

pub fn get_temp_path() -> PathBuf {
    Path::new("static-asset-generation").join("tmp")
}

#[derive(Debug, Clone)]
pub struct ImageToDownload {
    url: String,
    path: PathBuf,
}

pub async fn download_images() -> AppResult<(bool, bool, bool, bool, bool)> {
    let version = get_current_version().await?;
    let (item_images, profile_icons_images, perks) = tokio::join!(
        get_items(version.clone()),
        update_profile_icons_image(version.clone()),
        get_perks(version.clone())
    );

    let temp_path = get_temp_path();
    let mut champion_images = Vec::new();
    for (id, _) in CHAMPION_OPTIONS {
        let path = temp_path
            .join(AssetType::Champion.get_path())
            .join(format!("{}.png", id));
        if !path.exists() {
            let image_url = format!(
                "https://cdn.communitydragon.org/{}/champion/{}/square",
                version.clone(),
                riven::consts::Champion::from(*id as i16)
                    .identifier()
                    .unwrap()
            );
            champion_images.push(ImageToDownload {
                url: image_url,
                path: path.clone(),
            });
        }
    }
    let mut summoner_spells_images = Vec::new();
    for summoner_spell in SUMMONER_SPELL_OPTIONS {
        if *summoner_spell == 0 {
            continue;
        }
        let path = temp_path
            .join(AssetType::SummonerSpell.get_path())
            .join(format!("{}.png", summoner_spell));

        if !path.exists() {
            let url = format!(
                "https://ddragon.leagueoflegends.com/cdn/{}/img/spell/{}.png",
                version.clone(),
                SummonerSpell::from(*summoner_spell),
            );
            summoner_spells_images.push(ImageToDownload { url, path });
        }
    }
    let item_images = item_images?;
    let profile_icons_images = profile_icons_images?;
    let perks = perks?;

    let _ = tokio::join!(
        download_and_save_images(&item_images),
        download_and_save_images(&profile_icons_images),
        download_and_save_images(&perks),
        download_and_save_images(&champion_images),
        download_and_save_images(&summoner_spells_images),
    );
    Ok((
        !item_images.is_empty(),
        !profile_icons_images.is_empty(),
        !perks.is_empty(),
        !champion_images.is_empty(),
        !summoner_spells_images.is_empty(),
    ))
}

pub async fn convert_not_found_images_and_rebuild_sprite(
    item_images_modified: bool,
    profile_icons_images_modified: bool,
    perks_modified: bool,
    champion_images_modified: bool,
    summoner_spells_images_modified: bool,
) -> AppResult<()> {
    let _ = tokio::join!(
        rebuild_css_sprite(AssetType::Item, item_images_modified),
        convert_to_avif(AssetType::ProfileIcon, profile_icons_images_modified),
        rebuild_css_sprite(AssetType::Perk, perks_modified),
        rebuild_css_sprite(AssetType::Champion, champion_images_modified),
        rebuild_css_sprite(AssetType::SummonerSpell, summoner_spells_images_modified),
    );
    Ok(())
}

pub async fn convert_to_avif(asset_type: AssetType, modified: bool) -> AppResult<()> {
    if !modified {
        return Ok(());
    }

    let start = Instant::now();
    let default_assets_path = get_assets_path().join(asset_type.get_path());
    let temp_path = get_temp_path().join(asset_type.get_path());

    if !default_assets_path.exists() {
        tokio::fs::create_dir_all(&default_assets_path).await?;
    }

    let size = asset_type.default_size();
    let mut tasks = vec![];

    let mut dir = tokio::fs::read_dir(&temp_path).await?;
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            let name = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let image_data = tokio::fs::read(path.clone()).await?;
            let image_path = default_assets_path.join(format!("{}.avif", name));

            // Spawn a task for each file processing
            let task = tokio::spawn(async move {
                let image = image::load_from_memory_with_format(&image_data, ImageFormat::Png)
                    .map_err(|e| format!("Failed to load image at {}: {}", path.display(), e))?
                    .resize_exact(size.0, size.1, FilterType::Lanczos3)
                    .to_rgba8();

                tokio::fs::write(
                    image_path,
                    Encoder::new()
                        .with_quality(75.0)
                        .with_speed(1)
                        .encode_rgba(Img::new(
                            image.as_bytes().as_rgba(),
                            image.width() as usize,
                            image.height() as usize,
                        ))
                        .unwrap()
                        .avif_file,
                )
                    .await
                    .map_err(|e| format!("Failed to write file: {}", e))?;
                println!("Converted: {}", path.display());

                Ok::<(), String>(())
            });

            tasks.push(task);
        }
    }

    // Maximize concurrency by awaiting all tasks
    let results = futures::future::join_all(tasks).await;

    // Log errors if any
    for result in results {
        if let Err(e) = result {
            eprintln!("Task failed: {:?}", e);
        }
    }

    println!(
        "Time taken to convert {} to AVIF: {:?}",
        asset_type.get_path(),
        start.elapsed()
    );

    Ok(())
}

pub async fn rebuild_css_sprite(asset_type: AssetType, modified: bool) -> AppResult<()> {
    if !modified {
        return Ok(());
    }
    let start = std::time::Instant::now();
    let default_assets_path = get_assets_path();
    let temp_path = get_temp_path().join(asset_type.get_path());
    let size = asset_type.default_size();
    let mut dir = tokio::fs::read_dir(temp_path).await?;
    let mut tasks = vec![];
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            let task = tokio::task::spawn(async move {
                let name = path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("")
                    .to_string();

                let image_data = tokio::fs::read(&path).await.unwrap();
                let img = image::load_from_memory_with_format(&image_data, ImageFormat::Png)
                    .map_err(|e| format!("Failed to load image at {}: {}", path.display(), e)).unwrap()
                    .resize_exact(size.0, size.1, FilterType::Lanczos3);
                println!("Loaded: {}", path.display());
                Ok::<(String, DynamicImage), String>((name, img))
            });

            tasks.push(task);
        }
    }
    // Await all tasks concurrently
    let results = future::join_all(tasks).await;

    // Collect successful results, log or handle errors
    let mut all_images = vec![];
    for result in results {
        match result {
            Ok(Ok(image)) => all_images.push(image),
            Ok(Err(e)) => eprintln!("Error processing image: {}", e),
            Err(e) => eprintln!("Task panicked: {:?}", e),
        }
    }

    // find w h for a square sprite
    let n_images = all_images.len();
    let n_images_sqrt = (n_images as f64).sqrt().ceil() as u32;
    let sprite_size = (n_images_sqrt * size.0, n_images_sqrt * size.1);
    let mut sprite = image::RgbaImage::new(sprite_size.0, sprite_size.1);
    let mut css_classes = Vec::new();
    for (i, (name, img)) in all_images.iter().enumerate() {
        let x = (i as u32 % n_images_sqrt) * size.0;
        let y = (i as u32 / n_images_sqrt) * size.1;
        image::imageops::overlay(&mut sprite, img, x as i64, y as i64);
        css_classes.push(format!(
            r#"
.{}{{
    background-image: url('/assets/{}.avif');
    background-position: -{}px -{}px;
    width: {}px;
    height: {}px;
}}
            "#,
            asset_type.get_class_name(name.parse().unwrap()),
            asset_type.get_path(),
            x,
            y,
            size.0,
            size.1
        ));
    }

    let sprite_path = default_assets_path.join(format!("{}.avif", asset_type.get_path()));
    tokio::fs::create_dir_all(sprite_path.parent().unwrap()).await?;
    let result = Encoder::new()
        .with_quality(75.0)
        .with_speed(1)
        .encode_rgba(Img::new(
            sprite.as_bytes().as_rgba(),
            sprite.width() as usize,
            sprite.height() as usize,
        ))?;

    tokio::fs::write(sprite_path, result.avif_file).await?;
    let css_path = get_css_path().join(format!("{}.css", asset_type.get_path()));
    tokio::fs::write(css_path, css_classes.join("\n")).await?;
    println!(
        "Time taken to rebuild {} sprite and css: {:?}",
        asset_type.get_path(),
        start.elapsed()
    );
    Ok(())
}

async fn download_and_save_images(images_to_download: &Vec<ImageToDownload>) -> AppResult<()> {
    if images_to_download.is_empty() {
        return Ok(());
    }
    let client = reqwest::Client::new();

    futures::stream::iter(images_to_download)
        .for_each_concurrent(10, |image| {
            let client = client.clone();
            async move {
                // Download the image
                match client.get(&image.url).send().await {
                    Ok(response) => {
                        let image_data = response.bytes().await.unwrap();
                        if !image.path.parent().unwrap().exists() {
                            tokio::fs::create_dir_all(image.path.parent().unwrap()).await.unwrap();
                        }
                        let _ = tokio::fs::write(image.path.clone(), image_data.to_vec()).await;
                        println!("Downloaded: {}", image.url);
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
        let path = get_temp_path()
            .join(AssetType::Perk.get_path())
            .join(format!("{}.png", perk.id));
        if !path.exists() {
            let image_url = format!(
                "https://raw.communitydragon.org/latest/game/assets/perks/{}",
                perk.icon_path
                    .replace("/lol-game-data/assets/v1/perk-images/", "")
                    .to_lowercase()
            );
            images_to_download.push(ImageToDownload {
                url: image_url,
                path,
                //size: (22, 22),
            });
        }
    }
    let main_perks = StaticUrl::Perks2 { version }.get().await?;
    let main_perks: Vec<JsonPerk2> = serde_json::from_str(main_perks.as_str())?;
    for perk in main_perks {
        let path = get_temp_path()
            .join(AssetType::Perk.get_path())
            .join(format!("{}.png", perk.id));

        if !path.exists() {
            let image_url = format!("https://ddragon.leagueoflegends.com/cdn/img/{}", perk.icon);
            images_to_download.push(ImageToDownload {
                url: image_url,
                path,
            });
        }
    }
    Ok(images_to_download)
}

pub async fn get_items(version: String) -> AppResult<Vec<ImageToDownload>> {
    let mut images_to_download = Vec::new();
    let raw_items = StaticUrl::Items {
        version: version.clone(),
    }
    .get()
    .await?;
    let items_json: ItemData = serde_json::from_str(raw_items.as_str())?;
    for (id, _) in items_json.data {
        let path = get_temp_path()
            .join(AssetType::Item.get_path())
            .join(format!("{}.png", id));
        if !path.exists() {
            let image_url = format!(
                "https://ddragon.leagueoflegends.com/cdn/{}/img/item/{}.png",
                version.clone(),
                id
            );
            images_to_download.push(ImageToDownload {
                url: image_url,
                path: path.clone(),
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
    let raw_champions = StaticUrl::ProfileIcons {
        version: version.clone(),
    }
    .get()
    .await?;
    let data: HashMapData = serde_json::from_str(raw_champions.as_str())?;
    for (id, _) in data.data {
        let path = get_temp_path()
            .join(AssetType::ProfileIcon.get_path())
            .join(format!("{}.png", id));

        if !path.exists() {
            let image_url = format!(
                "https://ddragon.leagueoflegends.com/cdn/{}/img/profileicon/{}.png",
                version.clone(),
                id
            );
            images_to_download.push(ImageToDownload {
                url: image_url,
                path: path.clone(),
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
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        client
            .get(self.url().as_str())
            .send()
            .await?
            .text()
            .await
            .map_err(|e| e.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HashMapData {
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
