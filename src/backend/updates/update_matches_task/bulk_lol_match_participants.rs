use crate::backend::ssr::AppResult;
use crate::backend::updates::update_matches_task::TempParticipant;
use unzip_n::unzip_n;

unzip_n!(23);
unzip_n!(11);
unzip_n!(7);

pub async fn bulk_insert_lol_match_participants(db: &sqlx::PgPool, participants: &[TempParticipant]) -> AppResult<()> {
    let total_items = participants.len();
    let (
        mut champion_ids,
        mut summoner_ids,
        mut match_ids,
        mut summoner_spell1_ids,
        mut summoner_spell2_ids,
        mut team_ids,
        mut won_flags,
        mut champ_levels,
        mut kill_participations,
        mut kdas,
        mut killss,
        mut deathss,
        mut assistss,
        mut damage_dealt_to_championss,
        mut damage_takens,
        mut gold_earneds,
        mut wards_placeds,
        mut css,
        mut css_per_minute,
        mut double_kills,
        mut triple_kills,
        mut quadra_kills,
        mut penta_kills,
        mut perk_defense_ids,
        mut perk_flex_ids,
        mut perk_offense_ids,
        mut perk_primary_style_ids,
        mut perk_sub_style_ids,
        mut perk_primary_selection_ids,
        mut perk_primary_selection1_ids,
        mut perk_primary_selection2_ids,
        mut perk_primary_selection3_ids,
        mut perk_sub_selection1_ids,
        mut perk_sub_selection2_ids,
        mut item0_ids,
        mut item1_ids,
        mut item2_ids,
        mut item3_ids,
        mut item4_ids,
        mut item5_ids,
        mut item6_ids,
    ) = (
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
        Vec::with_capacity(total_items),
    );
    for participant in participants {
        champion_ids.push(participant.champion_id);
        summoner_ids.push(participant.summoner_id);
        match_ids.push(participant.lol_match_id);
        summoner_spell1_ids.push(participant.summoner_spell1_id);
        summoner_spell2_ids.push(participant.summoner_spell2_id);
        team_ids.push(participant.team_id);
        won_flags.push(participant.won);
        champ_levels.push(participant.champ_level);
        kill_participations.push(participant.kill_participation);
        kdas.push(participant.kda);
        killss.push(participant.kills);
        deathss.push(participant.deaths);
        assistss.push(participant.assists);
        damage_dealt_to_championss.push(participant.damage_dealt_to_champions);
        damage_takens.push(participant.damage_taken);
        gold_earneds.push(participant.gold_earned);
        wards_placeds.push(participant.wards_placed);
        css.push(participant.cs);
        css_per_minute.push(participant.cs_per_minute);
        double_kills.push(participant.double_kills);
        triple_kills.push(participant.triple_kills);
        quadra_kills.push(participant.quadra_kills);
        penta_kills.push(participant.penta_kills);
        perk_defense_ids.push(participant.perk_defense_id);
        perk_flex_ids.push(participant.perk_flex_id);
        perk_offense_ids.push(participant.perk_offense_id);
        perk_primary_style_ids.push(participant.perk_primary_style_id);
        perk_sub_style_ids.push(participant.perk_sub_style_id);
        perk_primary_selection_ids.push(participant.perk_primary_selection_id);
        perk_primary_selection1_ids.push(participant.perk_primary_selection1_id);
        perk_primary_selection2_ids.push(participant.perk_primary_selection2_id);
        perk_primary_selection3_ids.push(participant.perk_primary_selection3_id);
        perk_sub_selection1_ids.push(participant.perk_sub_selection1_id);
        perk_sub_selection2_ids.push(participant.perk_sub_selection2_id);
        item0_ids.push(participant.item0_id);
        item1_ids.push(participant.item1_id);
        item2_ids.push(participant.item2_id);
        item3_ids.push(participant.item3_id);
        item4_ids.push(participant.item4_id);
        item5_ids.push(participant.item5_id);
        item6_ids.push(participant.item6_id);
    }

    let sql = r#"
            INSERT INTO lol_match_participants (
                champion_id,
                summoner_id,
                lol_match_id,
                summoner_spell1_id,
                summoner_spell2_id,
                team_id,
                won,
                champ_level,
                kill_participation,
                kda,
                kills,
                deaths,
                assists,
                damage_dealt_to_champions,
                damage_taken,
                gold_earned,
                wards_placed,
                cs,
                cs_per_minute,
                double_kills,
                triple_kills,
                quadra_kills,
                penta_kills,
                perk_defense_id,
                perk_flex_id,
                perk_offense_id,
                perk_primary_style_id,
                perk_sub_style_id,
                perk_primary_selection_id,
                perk_primary_selection1_id,
                perk_primary_selection2_id,
                perk_primary_selection3_id,
                perk_sub_selection1_id,
                perk_sub_selection2_id,
                item0_id,
                item1_id,
                item2_id,
                item3_id,
                item4_id,
                item5_id,
                item6_id
            )
            SELECT * FROM UNNEST (
                $1::INT[],
                $2::INT[],
                $3::INT[],
                $4::INT[],
                $5::INT[],
                $6::INT[],
                $7::BOOL[],
                $8::INT[],
                $9::FLOAT8[],
                $10::FLOAT8[],
                $11::INT[],
                $12::INT[],
                $13::INT[],
                $14::INT[],
                $15::INT[],
                $16::INT[],
                $17::INT[],
                $18::INT[],
                $19::FLOAT8[],
                $20::INT[],
                $21::INT[],
                $22::INT[],
                $23::INT[],
                $24::INT[],
                $25::INT[],
                $26::INT[],
                $27::INT[],
                $28::INT[],
                $29::INT[],
                $30::INT[],
                $31::INT[],
                $32::INT[],
                $33::INT[],
                $34::INT[],
                $35::INT[],
                $36::INT[],
                $37::INT[],
                $38::INT[],
                $39::INT[],
                $40::INT[],
                $41::INT[]
            );
        "#;

    sqlx::query(sql)
        .bind(&champion_ids)
        .bind(&summoner_ids)
        .bind(&match_ids)
        .bind(&summoner_spell1_ids)
        .bind(&summoner_spell2_ids)
        .bind(&team_ids)
        .bind(&won_flags)
        .bind(&champ_levels)
        .bind(&kill_participations)
        .bind(&kdas)
        .bind(&killss)
        .bind(&deathss)
        .bind(&assistss)
        .bind(&damage_dealt_to_championss)
        .bind(&damage_takens)
        .bind(&gold_earneds)
        .bind(&wards_placeds)
        .bind(&css)
        .bind(&css_per_minute)
        .bind(&double_kills)
        .bind(&triple_kills)
        .bind(&quadra_kills)
        .bind(&penta_kills)
        .bind(&perk_defense_ids)
        .bind(&perk_flex_ids)
        .bind(&perk_offense_ids)
        .bind(&perk_primary_style_ids)
        .bind(&perk_sub_style_ids)
        .bind(&perk_primary_selection_ids)
        .bind(&perk_primary_selection1_ids)
        .bind(&perk_primary_selection2_ids)
        .bind(&perk_primary_selection3_ids)
        .bind(&perk_sub_selection1_ids)
        .bind(&perk_sub_selection2_ids)
        .bind(&item0_ids)
        .bind(&item1_ids)
        .bind(&item2_ids)
        .bind(&item3_ids)
        .bind(&item4_ids)
        .bind(&item5_ids)
        .bind(&item6_ids)
        .execute(db)
        .await?;

    Ok(())
}