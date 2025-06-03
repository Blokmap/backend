DROP TABLE location_image;

ALTER TABLE PROFILE DROP CONSTRAINT fk__profile__avatar_image_id;
DROP TABLE IMAGE;

DROP TABLE RESERVATION;

DROP INDEX idx__opening_time__start_end;
DROP INDEX idx__opening_time__location_id;
DROP TABLE opening_time;

DROP TABLE location_tag;

DROP TABLE tag;

DROP TABLE review;

DROP TABLE location_profile;

DROP TABLE location;

DROP TABLE authority_profile;

DROP TABLE authority;

ALTER TABLE profile DROP CONSTRAINT fk__profile__institution_id;
DROP TABLE institution;

DROP TABLE translation;

DROP TABLE profile;
DROP TYPE PROFILE_STATE;

DROP EXTENSION IF EXISTS pg_trgm;
