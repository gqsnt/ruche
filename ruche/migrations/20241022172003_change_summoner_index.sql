-- Add migration script here
drop index if exists idx_summoners_game_name_tag_line_platform;
CREATE INDEX idx_summoners_platform_game_name_tagline ON summoners (platform, game_name, lower(tag_line));