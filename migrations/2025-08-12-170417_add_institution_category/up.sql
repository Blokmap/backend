ALTER TABLE institution DROP CONSTRAINT fk__institution__slug_translation_id;
ALTER TABLE institution DROP COLUMN slug_translation_id;

CREATE TYPE INSTITUTION_CATEGORY AS ENUM (
	'education',
	'organisation',
	'government'
);

ALTER TABLE institution
	ADD COLUMN IF NOT EXISTS
	category INSTITUTION_CATEGORY NOT NULL DEFAULT 'education';

ALTER TABLE institution
	ADD COLUMN IF NOT EXISTS
	slug TEXT NOT NULL UNIQUE;
