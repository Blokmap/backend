CREATE TABLE institution_profile (
	institution_id INTEGER   NOT NULL,
	profile_id     INTEGER   NOT NULL,
	added_at       TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by       INTEGER,
	updated_at     TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by     INTEGER,
	permissions    BIGINT    NOT NULL DEFAULT 0,

	CONSTRAINT pk__institution_profile
	PRIMARY KEY (institution_id, profile_id),

	CONSTRAINT fk__institution_profile__institution_id
	FOREIGN KEY (institution_id)
	REFERENCES institution(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_profile__profile_id
	FOREIGN KEY (profile_id)
	REFERENCES profile(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_profile__added_by
	FOREIGN KEY (added_by)
	REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__authority_profile__updated_by
	FOREIGN KEY (updated_by)
	REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('institution_profile');
