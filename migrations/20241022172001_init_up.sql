CREATE EXTENSION IF NOT EXISTS pg_trgm;

--Region Enum
CREATE TYPE platform_type AS ENUM (
    'BR',
    'EUNE',
    'EUW',
    'JP',
    'KR',
    'LAN',
    'LAS',
    'MENA',
    'NA',
    'OCE',
    'PH',
    'RU',
    'SG',
    'TH',
    'TR',
    'TW',
    'VN',
    'PBE'
    );



-- Table: summoners
CREATE TABLE IF NOT EXISTS summoners
(
    id              SERIAL PRIMARY KEY,
    profile_icon_id INTEGER       NOT NULL,
    summoner_level  INTEGER       NOT NULL,
    updated_at      TIMESTAMP     NOT NULL DEFAULT NOW(),
    platform        platform_type NOT NULL,
    pro_player_slug VARCHAR(16)            DEFAULT NULL,
    puuid           VARCHAR(78)   NOT NULL UNIQUE,
    game_name       VARCHAR(16)   NOT NULL,
    tag_line        VARCHAR(5)    NOT NULL
);


-- Table: lol_matches
CREATE TABLE IF NOT EXISTS lol_matches
(
    id             SERIAL PRIMARY KEY,
    map_id         INTEGER,
    queue_id       INTEGER,
    match_duration INTEGER,
    match_creation TIMESTAMP,
    match_end      TIMESTAMP,
    updated        BOOLEAN DEFAULT FALSE NOT NULL,
    trashed        BOOLEAN DEFAULT FALSE NOT NULL,
    platform       platform_type,
    match_id       VARCHAR(17)           NOT NULL UNIQUE,
    game_mode      VARCHAR(15),
    version        VARCHAR(5)
);


-- Table: lol_match_participants
CREATE TABLE IF NOT EXISTS lol_match_participants
(
    id                         SERIAL PRIMARY KEY,
    lol_match_id               INTEGER       NOT NULL REFERENCES lol_matches (id) ON DELETE CASCADE,
    summoner_id                INTEGER       NOT NULL REFERENCES summoners (id) ON DELETE CASCADE,
    champion_id                INTEGER       NOT NULL,
    team_id                    INTEGER       NOT NULL,
    champ_level                INTEGER       NOT NULL,
    kills                      INTEGER       NOT NULL,
    deaths                     INTEGER       NOT NULL,
    assists                    INTEGER       NOT NULL,
    damage_dealt_to_champions  INTEGER       NOT NULL,
    damage_taken               INTEGER       NOT NULL,
    gold_earned                INTEGER       NOT NULL,
    wards_placed               INTEGER       NOT NULL,
    cs                         INTEGER       NOT NULL,
    cs_per_minute              DECIMAL,
    double_kills               INTEGER       NOT NULL,
    triple_kills               INTEGER       NOT NULL,
    quadra_kills               INTEGER       NOT NULL,
    penta_kills                INTEGER       NOT NULL,
    summoner_spell1_id         INTEGER,
    summoner_spell2_id         INTEGER,
    perk_defense_id            INTEGER,
    perk_flex_id               INTEGER,
    perk_offense_id            INTEGER,
    perk_primary_style_id      INTEGER,
    perk_sub_style_id          INTEGER,
    perk_primary_selection_id  INTEGER,
    perk_primary_selection1_id INTEGER,
    perk_primary_selection2_id INTEGER,
    perk_primary_selection3_id INTEGER,
    perk_sub_selection1_id     INTEGER,
    perk_sub_selection2_id     INTEGER,
    item0_id                   BIGINT,
    item1_id                   BIGINT,
    item2_id                   BIGINT,
    item3_id                   BIGINT,
    item4_id                   BIGINT,
    item5_id                   BIGINT,
    item6_id                   BIGINT,
    won                        BOOLEAN       NOT NULL,
    kill_participation         DECIMAL(5, 2) NOT NULL,
    kda                        DECIMAL(5, 2) NOT NULL
);

CREATE TABLE IF NOT EXISTS lol_match_timelines
(
    id                   SERIAL PRIMARY KEY,
    lol_match_id         INTEGER NOT NULL REFERENCES lol_matches (id) ON DELETE CASCADE,
    summoner_id          INTEGER NOT NULL REFERENCES summoners (id) ON DELETE CASCADE,
    skills_timeline      int[]   NOT NULL,
    items_event_timeline JSONB   NOT NULL
);

CREATE INDEX idx_summoners_game_name_trgm ON summoners USING gin (game_name gin_trgm_ops);
CREATE INDEX idx_summoners_game_name_tag_line_platform ON summoners (game_name, lower(tag_line), platform);
-- 2


-- Composite Index for lmp2


-- Covering Index for lmp2
CREATE INDEX idx_lmp4_covering ON lol_match_participants (summoner_id, lol_match_id, team_id, won)
    INCLUDE (
        champion_id, champ_level, kills, deaths, assists, kda, kill_participation,
        summoner_spell1_id, summoner_spell2_id, perk_primary_selection_id,
        perk_sub_style_id, item0_id, item1_id, item2_id, item3_id, item4_id,
        item5_id, item6_id
        ); -- 3
CREATE INDEX idx_lmp_lol_match_id_summoner_id
    ON lol_match_participants (lol_match_id, summoner_id)
    INCLUDE (team_id, won); -- 1
CREATE INDEX idx_lmp2_match_id ON lol_match_participants (lol_match_id); -- 2
CREATE INDEX idx_lmp2_summoner_id ON lol_match_participants (summoner_id); -- 1
CREATE INDEX idx_lmp2_match_id_summoner_id
    ON lol_match_participants (lol_match_id, summoner_id);-- 1
-- Enhanced Index on lol_matches
CREATE INDEX idx_lm_covering_full
    ON lol_matches (match_end DESC)
    INCLUDE (platform, queue_id, match_duration, match_id); -- 2
CREATE INDEX idx_lm_id_covering_full
    ON lol_matches (id, match_end DESC)
    INCLUDE (platform, queue_id, match_duration, match_id);-- 1
