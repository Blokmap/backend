DROP TABLE location_image;

ALTER TABLE PROFILE DROP CONSTRAINT fk__profile__avatar_image_id;
DROP TABLE IMAGE;

DROP INDEX idx__reservation__confirmed_by;
DROP INDEX idx__reservation__opening_time_id;
DROP INDEX idx__reservation__profile_id;
DROP TABLE reservation;
DROP TYPE RESERVATION_STATE;

DROP INDEX idx__opening_time__day;
DROP INDEX idx__opening_time__start_end;
DROP INDEX idx__opening_time__location_id;
DROP TABLE opening_time;

DROP TABLE location_tag;

DROP TABLE tag;

DROP TABLE review;

DROP TABLE location_member;
DROP TABLE authority_member;
DROP TABLE institution_member;

DROP TABLE location_role;
DROP TABLE authority_role;
DROP TABLE institution_role;

DROP TABLE role;

DROP TABLE location;

DROP TABLE authority;

ALTER TABLE profile DROP CONSTRAINT fk__profile__institution_id;
DROP TABLE institution;
DROP TYPE INSTITUTION_CATEGORY;

DROP TABLE translation;

DROP TABLE profile;
DROP TYPE PROFILE_STATE;

DROP EXTENSION IF EXISTS pg_trgm;
