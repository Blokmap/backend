CREATE TABLE permission (
	id SERIAL PRIMARY KEY,
	name TEXT NOT NULL
);

CREATE TABLE institution_role (
	id SERIAL PRIMARY KEY,
	institution_id INTEGER NOT NULL,
	name TEXT NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by INTEGER,
	updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by INTEGER,

	CONSTRAINT fk__institution_role__institution_id
	FOREIGN KEY (institution_id) REFERENCES institution(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution_role__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__institution_role__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('institution_role');

CREATE TABLE authority_role (
	id SERIAL PRIMARY KEY,
	authority_id INTEGER NOT NULL,
	name TEXT NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by INTEGER,
	updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by INTEGER,

	CONSTRAINT fk__authority_role__authority_id
	FOREIGN KEY (authority_id) REFERENCES authority(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_role__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__authority_role__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('authority_role');

CREATE TABLE location_role (
	id SERIAL PRIMARY KEY,
	location_id INTEGER NOT NULL,
	name TEXT NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by INTEGER,
	updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by INTEGER,

	CONSTRAINT fk__location_role__location_id
	FOREIGN KEY (location_id) REFERENCES location(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_role__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__location_role__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('location_role');

CREATE TABLE institution_role_permission (
	institution_role_id INTEGER NOT NULL,
	permission_id INTEGER NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by INTEGER,

	CONSTRAINT pk__institution_role_permission
	PRIMARY KEY (institution_role_id, permission_id),

	CONSTRAINT fk__institution_role_permission__institution_role_id
	FOREIGN KEY (institution_role_id) REFERENCES institution_role(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution_role_permission__permission_id
	FOREIGN KEY (permission_id) REFERENCES permission(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution_role_permission__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

CREATE TABLE authority_role_permission (
	authority_role_id INTEGER NOT NULL,
	permission_id INTEGER NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by INTEGER,

	CONSTRAINT pk__authority_role_permission
	PRIMARY KEY (authority_role_id, permission_id),

	CONSTRAINT fk__authority_role_permission__authority_role_id
	FOREIGN KEY (authority_role_id) REFERENCES authority_role(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_role_permission__permission_id
	FOREIGN KEY (permission_id) REFERENCES permission(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_role_permission__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

CREATE TABLE location_role_permission (
	location_role_id INTEGER NOT NULL,
	permission_id INTEGER NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by INTEGER,

	CONSTRAINT pk__location_role_permission
	PRIMARY KEY (location_role_id, permission_id),

	CONSTRAINT fk__location_role_permission__location_role_id
	FOREIGN KEY (location_role_id) REFERENCES location_role(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_role_permission__permission_id
	FOREIGN KEY (permission_id) REFERENCES permission(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_role_permission__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

ALTER TABLE institution_profile DROP COLUMN permissions;
ALTER TABLE institution_profile DROP CONSTRAINT pk__institution_profile;
ALTER TABLE institution_profile ADD COLUMN id SERIAL NOT NULL;
CREATE UNIQUE INDEX idx__institution_profile__pk ON institution_profile(id);
ALTER TABLE institution_profile ADD CONSTRAINT pk__institution_profile
	PRIMARY KEY USING INDEX idx__institution_profile__pk;

ALTER TABLE authority_profile DROP COLUMN permissions;
ALTER TABLE authority_profile DROP CONSTRAINT pk__authority_profile;
ALTER TABLE authority_profile ADD COLUMN id SERIAL NOT NULL;
CREATE UNIQUE INDEX idx__authority_profile__pk ON authority_profile(id);
ALTER TABLE authority_profile ADD CONSTRAINT pk__authority_profile
	PRIMARY KEY USING INDEX idx__authority_profile__pk;

ALTER TABLE location_profile DROP COLUMN permissions;
ALTER TABLE location_profile DROP CONSTRAINT pk__location_profile;
ALTER TABLE location_profile ADD COLUMN id SERIAL NOT NULL;
CREATE UNIQUE INDEX idx__location_profile__pk ON location_profile(id);
ALTER TABLE location_profile ADD CONSTRAINT pk__location_profile
	PRIMARY KEY USING INDEX idx__location_profile__pk;


CREATE TABLE institution_profile_role (
	institution_profile_id INTEGER NOT NULL,
	institution_role_id INTEGER NOT NULL,
	added_at TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by INTEGER,

	CONSTRAINT pk__institution_profile_role
	PRIMARY KEY (institution_profile_id, institution_role_id),

	CONSTRAINT fk__institution_profile_role__institution_profile_id
	FOREIGN KEY (institution_profile_id) REFERENCES institution_profile(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution_profile_role__institution_role_id
	FOREIGN KEY (institution_role_id) REFERENCES institution_role(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution_profile_role__added_by
	FOREIGN KEY (added_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

CREATE TABLE authority_profile_role (
	authority_profile_id INTEGER NOT NULL,
	authority_role_id INTEGER NOT NULL,
	added_at TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by INTEGER,

	CONSTRAINT pk__authority_profile_role
	PRIMARY KEY (authority_profile_id, authority_role_id),

	CONSTRAINT fk__authority_profile_role__authority_profile_id
	FOREIGN KEY (authority_profile_id) REFERENCES authority_profile(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_profile_role__authority_role_id
	FOREIGN KEY (authority_role_id) REFERENCES authority_role(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_profile_role__added_by
	FOREIGN KEY (added_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

CREATE TABLE location_profile_role (
	location_profile_id INTEGER NOT NULL,
	location_role_id INTEGER NOT NULL,
	added_at TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by INTEGER,

	CONSTRAINT pk__location_profile_role
	PRIMARY KEY (location_profile_id, location_role_id),

	CONSTRAINT fk__location_profile_role__location_profile_id
	FOREIGN KEY (location_profile_id) REFERENCES location_profile(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_profile_role__location_role_id
	FOREIGN KEY (location_role_id) REFERENCES location_role(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_profile_role__added_by
	FOREIGN KEY (added_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

