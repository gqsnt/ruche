pub fn version_to_major_minor(version: &str) -> String {
    let mut split = version.split(".");
    if split.clone().count() < 2 {
        panic!("version_to_major_minor: version: {}", version);
    }
    let major = split.next().unwrap();
    let minor = split.next().unwrap();
    format!("{}.{}", major, minor)
}


pub fn summoner_to_slug(game_name: &str, tag_line: &str) -> String {
    format!(
        "{}-{}",
        urlencoding::encode(game_name),
        urlencoding::encode(tag_line)
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

pub fn summoner_url(platform: &str, game_name: &str, tag_line: &str) -> String {
    format!("/platform/{}/summoners/{}", platform, summoner_to_slug(game_name, tag_line))
}

pub fn summoner_not_found_url(platform: &str, game_name: &str, tag_line: &str) -> String {
    format!("/platform/{}?game_name={}&tag_line={}", platform, game_name, tag_line)
}

pub fn summoner_encounter_url(platform:&str, game_name:&str, tag_line:&str, encounter_platform:&str, encounter_game_name:&str, encounter_tag_line:&str) -> String {
    format!("/platform/{}/summoners/{}?tab=encounter&encounter={}&encounter_platform={}", platform, summoner_to_slug(game_name, tag_line), summoner_to_slug(encounter_game_name, encounter_tag_line), encounter_platform)
}



pub fn round_to_2_decimal_places(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}