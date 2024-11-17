
-- Table: pro_players
CREATE TABLE IF NOT EXISTS pro_players
(
    id              SERIAL PRIMARY KEY,
    pro_uuid            UUID NOT NULL UNIQUE,
    slug            VARCHAR(50) NOT NULL
);

-- Table: summoners
CREATE TABLE IF NOT EXISTS summoners
(
    id              SERIAL PRIMARY KEY,
    puuid           VARCHAR(78) NOT NULL UNIQUE,
    game_name       VARCHAR(16) NOT NULL,
    tag_line        VARCHAR(5)  NOT NULL,
    profile_icon_id INTEGER     NOT NULL,
    summoner_level  BIGINT      NOT NULL,
    updated_at      TIMESTAMP   NOT NULL DEFAULT NOW(),
    platform        VARCHAR(4)  NOT NULL,
    pro_player_id  INTEGER     REFERENCES pro_players (id) DEFAULT null
);


-- Table: lol_matches
CREATE TABLE IF NOT EXISTS lol_matches
(
    id             SERIAL PRIMARY KEY,
    match_id       VARCHAR(17)           NOT NULL UNIQUE,
    game_mode      VARCHAR(15),
    map_id         INTEGER,
    queue_id       INTEGER,
    version        VARCHAR(5),
    platform       VARCHAR(4),
    updated        BOOLEAN DEFAULT FALSE NOT NULL,
    trashed        BOOLEAN DEFAULT FALSE NOT NULL,
    match_creation TIMESTAMP,
    match_end      TIMESTAMP,
    match_duration INTEGER
);


-- Table: lol_match_participants
CREATE TABLE IF NOT EXISTS lol_match_participants
(
    id                         SERIAL PRIMARY KEY,
    lol_match_id               INTEGER       NOT NULL REFERENCES lol_matches (id) ON DELETE CASCADE,
    summoner_id                INTEGER       NOT NULL REFERENCES summoners (id) ON DELETE CASCADE,
    champion_id                INTEGER       NOT NULL,
    team_id                    INTEGER       NOT NULL,
    won                        BOOLEAN       NOT NULL,
    champ_level                INTEGER       NOT NULL,
    kill_participation         DECIMAL(5, 2) NOT NULL,
    kda                        DECIMAL(5, 2) NOT NULL,
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
    item6_id                   BIGINT
);

CREATE TABLE IF NOT EXISTS lol_match_timelines
(
    id                   SERIAL PRIMARY KEY,
    lol_match_id         INTEGER NOT NULL REFERENCES lol_matches (id) ON DELETE CASCADE,
    summoner_id          INTEGER NOT NULL REFERENCES summoners (id) ON DELETE CASCADE,
    items_event_timeline JSONB   NOT NULL,
    skills_timeline      int[]   NOT NULL
);






create index lol_match_participants_summoner_id_idx on lol_match_participants (summoner_id);
create index lol_match_participants__match_id_idx on lol_match_participants (lol_match_id);
create index lol_match_participants_summoner_lol_match_id_idx on lol_match_participants (summoner_id, lol_match_id);
create index lol_match_participants_summoners_champion_id_idx on lol_match_participants (summoner_id, champion_id);
create index lol_match_participants_lol_match_id_champion_id_idx on lol_match_participants (lol_match_id, champion_id);
create index lol_match_match_end_idx on lol_matches (match_end);
create index lol_match_match_end_queue_id_idx on lol_matches (queue_id, match_end);
create index lol_match_queue_id_idx on lol_matches (queue_id);

CREATE INDEX summoners_game_tag_platform_idx ON summoners (game_name, tag_line, platform);

CREATE INDEX summoners_puuid_idx ON summoners (puuid);
CREATE INDEX lol_matches_updated_idx ON lol_matches (updated);
CREATE INDEX lol_matches_match_id_desc_idx ON lol_matches (match_id desc);

CREATE INDEX pro_players_puuid_idx ON pro_players (pro_uuid);
