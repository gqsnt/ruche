
pub type GameName= [u8;16];
pub type TagLine= [u8;5];
pub type Puuid= [u8;78];
pub type ProPlayerSlug= [u8;20];
pub type SummonerSlug = [u8;22];
pub type RiotMatchId = [u8;17];
pub type GameMode= [u8;15];
pub type Version= [u8;5];


pub trait FixedToString{
    fn to_string(&self) -> String;
    fn to_str(&self) -> &str;
}

// impl fiex to string for u8 N
impl<const N: usize> FixedToString for [u8; N] {
    fn to_string(&self) -> String {
        String::from_utf8_lossy(self.trim_end_zeros()).to_string()
    }

    fn to_str(&self) -> &str {
        std::str::from_utf8(self.trim_end_zeros()).unwrap()
    }
}

trait TrimEndZeros {
    fn trim_end_zeros(&self) -> &[u8];
}

impl TrimEndZeros for [u8] {
    fn trim_end_zeros(&self) -> &[u8] {
        let end = self.iter().rposition(|&b| b != 0).map_or(0, |pos| pos + 1);
        &self[..end]
    }
}



pub fn version_to_major_minor(version: &str) -> String {
    let mut split = version.split(".");
    if split.clone().count() < 2 {
        panic!("version_to_major_minor: version: {}", version);
    }
    let major = split.next().unwrap();
    let minor = split.next().unwrap();
    format!("{}.{}", major, minor)
}

pub fn format_with_spaces(number: u32) -> String {
    let mut num_str = number.to_string();
    let mut result = String::new();

    while num_str.len() > 3 {
        let split_at = num_str.len() - 3;
        result.insert_str(0, &format!(" {}", &num_str[split_at..]));
        num_str.truncate(split_at);
    }
    result.insert_str(0, &num_str);

    result
}


pub fn summoner_to_slug(game_name: String, tag_line: String) -> String {
    format!(
        "{}-{}",
        urlencoding::encode(game_name.as_str()),
        urlencoding::encode(tag_line.as_str())
    )
}

pub fn parse_summoner_slug(slug: &str) -> (String, String) {
    let parts: Vec<&str> = slug.split('-').collect();
    let len = parts.len();
    let game_name = urlencoding::decode(parts[0]).ok().unwrap().into_owned();
    if len == 2 {
        return (game_name, urlencoding::decode(parts[1]).ok().unwrap().into_owned());
    }
    (game_name, String::new())
}

pub fn summoner_url(platform: String, game_name: String, tag_line: String) -> String {
    format!("/platform/{}/summoners/{}", platform, summoner_to_slug(game_name, tag_line))
}

pub fn summoner_not_found_url(platform: String, game_name: String, tag_line: String) -> String {
    format!("/platform/{}?game_name={}&tag_line={}", platform, game_name, tag_line)
}

pub fn summoner_encounter_url(platform: String, game_name: String, tag_line: String, encounter_platform: String, encounter_game_name: String, encounter_tag_line: String) -> String {
    format!("/platform/{}/summoners/{}?tab=encounter&encounter_slug={}&encounter_platform={}", platform, summoner_to_slug(game_name, tag_line), summoner_to_slug(encounter_game_name, encounter_tag_line), encounter_platform)
}


pub fn round_to_2_decimal_places(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub fn string_to_fixed_array<const N: usize>(input: &str) -> [u8; N] {
    let mut result = [0u8; N]; // Initialize the fixed-size array with zeros.
    let bytes = input.as_bytes(); // Get the string as a slice of bytes.
    let len = bytes.len().min(N); // Determine how much of the string fits into the array.
    result[..len].copy_from_slice(&bytes[..len]); // Copy the bytes into the fixed-size array.
    result
}

pub fn format_float_to_2digits(value: f32) -> String {
    let value = (value * 100.0).round()/100.0;
    value.to_string()
}