DROP TABLE location_profile_role;
DROP TABLE authority_profile_role;
DROP TABLE institution_profile_role;

ALTER TABLE location_profile DROP CONSTRAINT pk__location_profile;
ALTER TABLE location_profile DROP COLUMN id;
ALTER TABLE location_profile ADD CONSTRAINT pk__location_profile
	PRIMARY KEY (location_id, profile_id);
ALTER TABLE location_profile ADD COLUMN permissions BIGINT NOT NULL DEFAULT 0;

ALTER TABLE authority_profile DROP CONSTRAINT pk__authority_profile;
ALTER TABLE authority_profile DROP COLUMN id;
ALTER TABLE authority_profile ADD CONSTRAINT pk__authority_profile
	PRIMARY KEY (authority_id, profile_id);
ALTER TABLE authority_profile ADD COLUMN permissions BIGINT NOT NULL DEFAULT 0;

ALTER TABLE institution_profile DROP CONSTRAINT pk__institution_profile;
ALTER TABLE institution_profile DROP COLUMN id;
ALTER TABLE institution_profile ADD CONSTRAINT pk__institution_profile
	PRIMARY KEY (institution_id, profile_id);
ALTER TABLE institution_profile ADD COLUMN permissions BIGINT NOT NULL DEFAULT 0;

DROP TABLE location_role_permission;
DROP TABLE authority_role_permission;
DROP TABLE institution_role_permission;

DROP TABLE location_role;
DROP TABLE authority_role;
DROP TABLE institution_role;

DROP TABLE permission;
