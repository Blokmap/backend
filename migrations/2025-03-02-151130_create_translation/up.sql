CREATE TYPE Language AS ENUM ('nl', 'en', 'fr', 'de');

CREATE TABLE translation (
	id         SERIAL    PRIMARY KEY,
	language   Language  NOT NULL,
	key        UUID      NOT NULL,
	text       TEXT      NOT NULL,
	created_at TIMESTAMP NOT NULL     DEFAULT now(),
	updated_at TIMESTAMP NOT NULL     DEFAULT now(),

	UNIQUE (language, key)
);

CREATE OR REPLACE FUNCTION fn__update_translation_updated_at()
	RETURNS TRIGGER
	LANGUAGE plpgsql
AS $BODY$
BEGIN
	NEW.updated_at = now();
	RETURN NEW;
END;
$BODY$;

CREATE OR REPLACE TRIGGER tr__update_translation_updated_at
	BEFORE UPDATE
	ON translation
	FOR EACH ROW
	EXECUTE PROCEDURE fn__update_translation_updated_at();
