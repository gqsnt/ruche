-- search summoner : default
EXPLAIN (ANALYZE, VERBOSE, BUFFERS, TIMING, SUMMARY, SETTINGS)
SELECT id, game_name, tag_line, platform
FROM summoners
WHERE game_name like 'random iron'
  AND lower(tag_line) like lower('euw')
  AND platform = 'EUW';

-- search summoner:id by ppuid
EXPLAIN (ANALYZE, VERBOSE, BUFFERS, TIMING, SUMMARY, SETTINGS)
SELECT id
FROM summoners
WHERE puuid = 'qFuBwNfOdqhisiAtW-P5rTv1cxRE1FiFSTXf1M_LH7CYvwqnAjZ2U2voUv-lIYS83-c4zhxgr2HRdA'
  and platform = 'EUW';


-- get summoner: default
EXPLAIN (ANALYZE, VERBOSE, BUFFERS, TIMING, SUMMARY, SETTINGS)
SELECT ss.id              as id,
       ss.game_name       as game_name,
       ss.tag_line        as tag_line,
       ss.platform        as platform,
       ss.profile_icon_id as profile_icon_id,
       ss.summoner_level  as summoner_level,
       ss.puuid           as puuid,
       ss.updated_at      as updated_at,
       ss.pro_player_slug as pro_slug
FROM summoners as ss
WHERE ss.game_name = 'Random Iron'
  AND lower(ss.tag_line) = lower('EUW')
  AND ss.platform = 'EUW';


-- get match_details: default
EXPLAIN (ANALYZE, BUFFERS)
SELECT lmp.id,
       lmp.lol_match_id,
       lmp.summoner_id,
       ss.game_name,
       ss.tag_line,
       ss.platform,
       ss.summoner_level,
       ss.profile_icon_id,
       ss.pro_player_slug,
       lmp.champion_id,
       lmp.team_id,
       lmp.won,
       lmp.kills,
       lmp.deaths,
       lmp.assists,
       lmp.champ_level,
       lmp.kda,
       lmp.kill_participation,
       lmp.damage_dealt_to_champions,
       lmp.damage_taken,
       lmp.gold_earned,
       lmp.wards_placed,
       lmp.cs,
       lmp.summoner_spell1_id,
       lmp.summoner_spell2_id,
       lmp.perk_defense_id,
       lmp.perk_flex_id,
       lmp.perk_offense_id,
       lmp.perk_primary_style_id,
       lmp.perk_sub_style_id,
       lmp.perk_primary_selection_id,
       lmp.perk_primary_selection1_id,
       lmp.perk_primary_selection2_id,
       lmp.perk_primary_selection3_id,
       lmp.perk_sub_selection1_id,
       lmp.perk_sub_selection2_id,
       lmp.item0_id,
       lmp.item1_id,
       lmp.item2_id,
       lmp.item3_id,
       lmp.item4_id,
       lmp.item5_id,
       lmp.item6_id
FROM lol_match_participants as lmp
         left JOIN summoners as ss ON ss.id = lmp.summoner_id
WHERE lmp.lol_match_id = 2575;


-- get match_details: match timeline
EXPLAIN (ANALYZE, BUFFERS)
SELECT *
FROM lol_match_timelines
WHERE lol_match_id = 2575;

ANALYZE lol_match_participants;
VACUUM ANALYZE lol_match_participants;

--get matches - live encounters
EXPLAIN (ANALYZE, BUFFERS)
SELECT lmp.summoner_id,
       COUNT(*)                                                              AS match_count
       from lol_match_participants lmp
            JOIN lol_match_participants tm ON lmp.lol_match_id = tm.lol_match_id AND tm.summoner_id = 14526
where lmp.summoner_id = ANY(ARRAY[1,2,3,4,5,6,7,8,9,10])
group by lmp.summoner_id ;


-- get encounters : default
EXPLAIN (ANALYZE, BUFFERS)
SELECT lmp.summoner_id,
       COUNT(*)                                                              AS match_count,
       COUNT(*) OVER ()                                                      AS total_count,
       COUNT(*) FILTER (WHERE lmp.team_id = tm.team_id)                      AS with_match_count,
       SUM(CASE WHEN lmp.team_id = tm.team_id AND tm.won THEN 1 ELSE 0 END)  AS with_win_count,
       COUNT(*) FILTER (WHERE lmp.team_id != tm.team_id)                     AS vs_match_count,
       SUM(CASE WHEN lmp.team_id != tm.team_id AND tm.won THEN 1 ELSE 0 END) AS vs_win_count
FROM lol_match_participants lmp
         JOIN lol_match_participants tm ON lmp.lol_match_id = tm.lol_match_id AND tm.summoner_id = 14526
WHERE lmp.summoner_id != 14526
GROUP BY lmp.summoner_id
ORDER BY match_count DESC
LIMIT 40 OFFSET 0;


-- get encounters : default with like
EXPLAIN (ANALYZE,BUFFERS)
SELECT lmp.summoner_id,
       COUNT(*)                                                              AS match_count,
       COUNT(*) OVER ()                                                      AS total_count,
       COUNT(*) FILTER (WHERE lmp.team_id = tm.team_id)                      AS with_match_count,
       SUM(CASE WHEN lmp.team_id = tm.team_id AND tm.won THEN 1 ELSE 0 END)  AS with_win_count,
       COUNT(*) FILTER (WHERE lmp.team_id != tm.team_id)                     AS vs_match_count,
       SUM(CASE WHEN lmp.team_id != tm.team_id AND tm.won THEN 1 ELSE 0 END) AS vs_win_count
FROM lol_match_participants lmp
         left join summoners ss on lmp.summoner_id = ss.id
         JOIN
     lol_match_participants tm
     ON lmp.lol_match_id = tm.lol_match_id
         AND tm.summoner_id = 14526 and tm.champion_id = 67
WHERE lmp.summoner_id != 14526
  and ss.game_name like '%ben%'
GROUP BY lmp.summoner_id
ORDER BY match_count DESC
LIMIT 40 OFFSET 0;


-- get encounter : default
EXPLAIN (ANALYZE, BUFFERS)
SELECT lmp1.lol_match_id,
       lm.match_end,
       lm.platform,
       lm.queue_id,
       lm.match_duration,
       lm.match_id                    AS riot_match_id,
       lmp1.summoner_id,
       lmp1.won,
       lmp1.champion_id,
       lmp1.champ_level,
       lmp1.kills,
       lmp1.deaths,
       lmp1.assists,
       lmp1.kda,
       lmp1.kill_participation,
       lmp1.summoner_spell1_id,
       lmp1.summoner_spell2_id,
       lmp1.perk_primary_selection_id,
       lmp1.perk_sub_style_id,
       lmp1.item0_id,
       lmp1.item1_id,
       lmp1.item2_id,
       lmp1.item3_id,
       lmp1.item4_id,
       lmp1.item5_id,
       lmp1.item6_id,
       lmp2.summoner_id               AS encounter_summoner_id,
       lmp2.won                       AS encounter_won,
       lmp2.champion_id               AS encounter_champion_id,
       lmp2.champ_level               AS encounter_champ_level,
       lmp2.kills                     AS encounter_kills,
       lmp2.deaths                    AS encounter_deaths,
       lmp2.assists                   AS encounter_assists,
       lmp2.kda                       AS encounter_kda,
       lmp2.kill_participation        AS encounter_kill_participation,
       lmp2.summoner_spell1_id        AS encounter_summoner_spell1_id,
       lmp2.summoner_spell2_id        AS encounter_summoner_spell2_id,
       lmp2.perk_primary_selection_id AS encounter_perk_primary_selection_id,
       lmp2.perk_sub_style_id         AS encounter_perk_sub_style_id,
       lmp2.item0_id                  AS encounter_item0_id,
       lmp2.item1_id                  AS encounter_item1_id,
       lmp2.item2_id                  AS encounter_item2_id,
       lmp2.item3_id                  AS encounter_item3_id,
       lmp2.item4_id                  AS encounter_item4_id,
       lmp2.item5_id                  AS encounter_item5_id,
       lmp2.item6_id                  AS encounter_item6_id
FROM lol_match_participants lmp1
         left JOIN lol_matches lm ON lm.id = lmp1.lol_match_id
         JOIN lol_match_participants lmp2 ON lmp2.lol_match_id = lmp1.lol_match_id and lmp2.summoner_id = 8365
    AND lmp2.won = lmp1.won
where lmp1.summoner_id = 14526
order by lm.match_end desc
limit 20 offset 0;


--get champions : default
EXPLAIN (ANALYZE, BUFFERS )
SELECT lmp.champion_id,
       count(lmp.lol_match_id)                  as total_matches,
       sum(CASE WHEN lmp.won THEN 1 ELSE 0 END) AS total_wins,
       avg(lmp.kda)                             as avg_kda,
       avg(lmp.kill_participation)              as avg_kill_participation,
       avg(lmp.kills)                           as avg_kills,
       avg(lmp.deaths)                          as avg_deaths,
       avg(lmp.assists)                         as avg_assists,
       avg(lmp.gold_earned)                     as avg_gold_earned,
       avg(lmp.cs)                              as avg_cs,
       avg(lmp.damage_dealt_to_champions)       as avg_damage_dealt_to_champions,
       avg(lmp.damage_taken)                    as avg_damage_taken,
       sum(lmp.double_kills)                    AS total_double_kills,
       sum(lmp.triple_kills)                    AS total_triple_kills,
       sum(lmp.quadra_kills)                    AS total_quadra_kills,
       sum(lmp.penta_kills)                     AS total_penta_kills
FROM lol_match_participants as lmp
         left JOIN lol_matches lm ON lm.id = lmp.lol_match_id
WHERE lmp.summoner_id = 14526
GROUP BY lmp.champion_id
ORDER BY total_matches DESC;


-- get live game : default
EXPLAIN (ANALYZE, VERBOSE, BUFFERS, TIMING, SUMMARY, SETTINGS)
select summoner_id,
       champion_id,
       count(lmp.lol_match_id)              as total_match,
       sum(CASE WHEN won THEN 1 ELSE 0 END) as total_win,
       avg(lmp.kills)                       as avg_kills,
       avg(lmp.deaths)                      as avg_deaths,
       avg(lmp.assists)                     as avg_assists
from lol_match_participants as lmp
         join lol_matches as lm on lmp.lol_match_id = lm.id
where lmp.summoner_id = ANY (ARRAY [14616, 14812, 15099, 15109, 14884, 14528, 14623, 14526, 14576, 15080])
  and lm.queue_id = 420
  and lm.match_end >= '2024-09-25 12:00:00'
group by lmp.summoner_id, lmp.champion_id;

-- get matches : default
EXPLAIN (ANALYZE, BUFFERS)
SELECT lmp.id,
       lmp.lol_match_id,
       lmp.champion_id,
       lmp.summoner_id,
       lmp.team_id,
       lmp.won,
       lmp.champ_level,
       lmp.kill_participation,
       lmp.kda,
       lmp.kills,
       lmp.deaths,
       lmp.assists,
       lmp.summoner_spell1_id,
       lmp.summoner_spell2_id,
       lmp.perk_primary_selection_id,
       lmp.perk_sub_style_id,
       lmp.item0_id,
       lmp.item1_id,
       lmp.item2_id,
       lmp.item3_id,
       lmp.item4_id,
       lmp.item5_id,
       lmp.item6_id,
       lm.match_id       AS riot_match_id,
       lm.platform       AS platform,
       lm.queue_id       AS lol_match_queue_id,
       lm.match_end      AS lol_match_match_end,
       lm.match_duration AS lol_match_match_duration
FROM lol_match_participants as lmp
         JOIN lol_matches as lm
              ON lm.id = lmp.lol_match_id
WHERE lmp.summoner_id = 14526
ORDER BY lm.match_end DESC
LIMIT 20 OFFSET 0;


-- get matches : default stats
EXPLAIN (ANALYZE, VERBOSE, BUFFERS, TIMING, SUMMARY, SETTINGS)
SELECT sum(CASE WHEN lmp.won THEN 1 ELSE 0 END) as total_wins,
       avg(lmp.kills)                           as avg_kills,
       avg(lmp.deaths)                          as avg_deaths,
       avg(lmp.assists)                         as avg_assists,
       avg(lmp.kda)                             as avg_kda,
       avg(lmp.kill_participation)              as avg_kill_participation
FROM lol_match_participants as lmp
         left JOIN lol_matches as lm ON lm.id = lmp.lol_match_id
WHERE lmp.summoner_id = 14526
LIMIT 20 OFFSET 0;

-- get matches : default participants
EXPLAIN (ANALYZE, BUFFERS)
SELECT lmp.lol_match_id,
       lmp.summoner_id,
       lmp.champion_id,
       lmp.team_id
FROM lol_match_participants as lmp
WHERE lmp.lol_match_id = ANY
      (ARRAY [2544, 2545, 2546, 2547, 2548, 2549, 2550, 2551, 2552, 2553, 2554, 2555, 2556, 2557, 2558, 2559, 2560, 2561, 2562, 2563])
ORDER BY lmp.team_id;

