ALTER TABLE authority DROP CONSTRAINT fk__authority__institution_id;

ALTER TABLE authority DROP COLUMN IF EXISTS institution_id;
