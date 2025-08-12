ALTER TABLE institution DROP COLUMN slug;

ALTER TABLE institution DROP COLUMN category;

DROP TYPE INSTITUTION_CATEGORY;

ALTER TABLE institution
	ADD COLUMN IF NOT EXISTS
    slug_translation_id INTEGER NOT NULL;

ALTER TABLE institution
	ADD CONSTRAINT fk__institution__slug_translation_id FOREIGN KEY (slug_translation_id)
	REFERENCES translation(id) ON DELETE CASCADE;
