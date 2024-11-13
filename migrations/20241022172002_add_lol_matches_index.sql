-- Add migration script here
CREATE INDEX lol_matches_updated_idx ON lol_matches (updated);
CREATE INDEX lol_matches_match_id_desc_idx ON lol_matches (match_id desc);
