CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TYPE PROFILE_STATE AS ENUM ('pending_email_verification', 'active', 'disabled');

CREATE TABLE profile (
	id                              SERIAL        PRIMARY KEY,
	username                        TEXT          COLLATE "case_insensitive" NOT NULL UNIQUE,
	first_name                      TEXT          COLLATE "case_insensitive",
	last_name                       TEXT          COLLATE "case_insensitive",
	avatar_image_id                 INTEGER,
	institution_id                  INTEGER,
	password_hash                   TEXT          NOT NULL,
	password_reset_token            TEXT          UNIQUE DEFAULT NULL,
	password_reset_token_expiry     TIMESTAMP     DEFAULT NULL,
	email                           TEXT          COLLATE "case_insensitive" UNIQUE DEFAULT NULL,
	pending_email                   TEXT          COLLATE "case_insensitive" UNIQUE,
	email_confirmation_token        TEXT          UNIQUE,
	email_confirmation_token_expiry TIMESTAMP,
	is_admin                        BOOLEAN       NOT NULL DEFAULT false,
	block_reason                    TEXT,
	state                           PROFILE_STATE NOT NULL DEFAULT 'pending_email_verification',
	created_at                      TIMESTAMP     NOT NULL DEFAULT NOW(),
	updated_at                      TIMESTAMP     NOT NULL DEFAULT NOW(),
	updated_by                      INTEGER,
	last_login_at                   TIMESTAMP     NOT NULL DEFAULT NOW(),

	CONSTRAINT fk__profile__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('profile');



CREATE TABLE translation (
	id         SERIAL    PRIMARY KEY,
	nl         TEXT,
    en         TEXT,
    fr         TEXT,
    de         TEXT,
	created_at TIMESTAMP NOT NULL     DEFAULT now(),
	created_by INTEGER,
	updated_at TIMESTAMP NOT NULL     DEFAULT now(),
	updated_by INTEGER,

	CONSTRAINT fk__translation__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__translation__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('translation');



CREATE TYPE INSTITUTION_CATEGORY AS ENUM (
	'education',
	'organisation',
	'government'
);

CREATE TABLE institution (
	id                  SERIAL               PRIMARY KEY,
    name_translation_id INTEGER              NOT NULL,
	slug                TEXT                 NOT NULL UNIQUE,
	category            INSTITUTION_CATEGORY NOT NULL DEFAULT 'education',
	email               TEXT                 COLLATE "case_insensitive" UNIQUE,
	phone_number        TEXT                 COLLATE "case_insensitive",
    street              TEXT                 COLLATE "case_insensitive",
	number              TEXT                 COLLATE "case_insensitive",
    zip                 TEXT                 COLLATE "case_insensitive",
	city                TEXT                 COLLATE "case_insensitive",
	province            TEXT                 COLLATE "case_insensitive",
	country             VARCHAR(2)           COLLATE "case_insensitive",
    created_at          TIMESTAMP            NOT NULL DEFAULT NOW(),
	created_by          INTEGER,
    updated_at          TIMESTAMP            NOT NULL DEFAULT NOW(),
	updated_by          INTEGER,

	CONSTRAINT fk__institution__name_translation_id
	FOREIGN KEY (name_translation_id) REFERENCES translation(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__institution__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('institution');

ALTER TABLE profile
	ADD CONSTRAINT fk__profile__institution_id
	FOREIGN KEY (institution_id) REFERENCES institution(id)
	ON DELETE SET NULL;



CREATE TABLE institution_member (
	id             SERIAL    PRIMARY KEY,
	institution_id INTEGER   NOT NULL,
	profile_id     INTEGER   NOT NULL,
	added_at       TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by       INTEGER,
	updated_at     TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by     INTEGER,

	CONSTRAINT fk__institution_member__institution_id
	FOREIGN KEY (institution_id) REFERENCES institution(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution_member__profile_id
	FOREIGN KEY (profile_id) REFERENCES profile(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution_member__added_by
	FOREIGN KEY (added_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__institution_member__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('institution_member');



CREATE TABLE authority (
    id             SERIAL    PRIMARY KEY,
    name           TEXT      NOT NULL,
    description    TEXT,
	institution_id INTEGER,
    created_at     TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by     INTEGER,
    updated_at     TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by     INTEGER,

	CONSTRAINT fk__authority__institution_id
	FOREIGN KEY (institution_id) REFERENCES institution(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__authority__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__authority__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('authority');



CREATE TABLE authority_member (
	id           SERIAL    PRIMARY KEY,
	authority_id INTEGER   NOT NULL,
	profile_id   INTEGER   NOT NULL,
	added_at     TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by     INTEGER,
	updated_at   TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by   INTEGER,

	CONSTRAINT fk__authority_member__authority_id
	FOREIGN KEY (authority_id) REFERENCES authority(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_member__profile_id
	FOREIGN KEY (profile_id) REFERENCES profile(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_member__added_by
	FOREIGN KEY (added_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__authority_member__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('authority_member');



CREATE TABLE location (
    id                     SERIAL  PRIMARY KEY,
    name                   TEXT    NOT NULL,
	authority_id           INTEGER,
    description_id         INTEGER NOT NULL,
    excerpt_id             INTEGER NOT NULL,
    seat_count             INTEGER NOT NULL,
    is_reservable          BOOLEAN NOT NULL,
	max_reservation_length INTEGER,
    is_visible             BOOLEAN NOT NULL DEFAULT TRUE,
    street                 TEXT       COLLATE "case_insensitive" NOT NULL,
    number                 TEXT       COLLATE "case_insensitive" NOT NULL,
    zip                    TEXT       COLLATE "case_insensitive" NOT NULL,
    city                   TEXT       COLLATE "case_insensitive" NOT NULL,
    province               TEXT       COLLATE "case_insensitive" NOT NULL,
	country                VARCHAR(2) COLLATE "case_insensitive" NOT NULL,
    latitude               DOUBLE PRECISION NOT NULL,
    longitude              DOUBLE PRECISION NOT NULL,
    approved_at            TIMESTAMP,
    approved_by            INTEGER,
    rejected_at            TIMESTAMP,
    rejected_by            INTEGER,
    rejected_reason        TEXT,
    created_at             TIMESTAMP NOT NULL    DEFAULT now(),
    created_by             INTEGER,
    updated_at             TIMESTAMP NOT NULL    DEFAULT now(),
	updated_by             INTEGER,

	CONSTRAINT fk__location__authority_id
	FOREIGN KEY (authority_id) REFERENCES authority(id)
	ON DELETE SET NULL,

    CONSTRAINT fk__location__description_id
    FOREIGN KEY (description_id) REFERENCES translation(id)
    ON DELETE CASCADE,

    CONSTRAINT fk__location__excerpt_id
    FOREIGN KEY (excerpt_id) REFERENCES translation(id)
    ON DELETE CASCADE,

    CONSTRAINT fk__location__approved_by
    FOREIGN KEY (approved_by) REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__location__rejected_by
    FOREIGN KEY (rejected_by) REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__location__created_by
    FOREIGN KEY (created_by) REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__location__updated_by
    FOREIGN KEY (updated_by) REFERENCES profile(id)
    ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('location');



CREATE TABLE location_member (
	id          SERIAL    PRIMARY KEY,
	location_id INTEGER   NOT NULL,
	profile_id  INTEGER   NOT NULL,
	added_at    TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by    INTEGER,
	updated_at  TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by  INTEGER,

	CONSTRAINT fk__location_member__location_id
	FOREIGN KEY (location_id)
	REFERENCES location(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_member__profile_id
	FOREIGN KEY (profile_id)
	REFERENCES profile(id)
	ON DELETE CASCADE,

    CONSTRAINT fk__location_member__added_by
    FOREIGN KEY (added_by) REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__location_member__updated_by
    FOREIGN KEY (updated_by) REFERENCES profile(id)
    ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('location_member');



CREATE TABLE review (
    id          SERIAL    PRIMARY KEY,
	profile_id  INTEGER   NOT NULL,
	location_id INTEGER   NOT NULL,
	rating      INTEGER   NOT NULL CHECK (0 <= rating AND rating <= 5),
	body        TEXT,
	created_at  TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_at  TIMESTAMP NOT NULL DEFAULT NOW(),
	hidden_at   TIMESTAMP,
	hidden_by   INTEGER,

	CONSTRAINT unq__review
	UNIQUE (profile_id, location_id),

	CONSTRAINT fk__review__profile_id
	FOREIGN KEY (profile_id) REFERENCES profile(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__review__location_id
	FOREIGN KEY (location_id) REFERENCES location(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__review__hidden_by
	FOREIGN KEY (hidden_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('review');



CREATE TABLE tag (
	id                  SERIAL    PRIMARY KEY,
	name_translation_id INTEGER   NOT NULL,
	created_at          TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by          INTEGER,
	updated_at          TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by          INTEGER,

	CONSTRAINT fk__tag__name_translation_id
	FOREIGN KEY (name_translation_id) REFERENCES translation(id)
	ON DELETE CASCADE,

    CONSTRAINT fk__tag__created_by
    FOREIGN KEY (created_by) REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__tag__updated_by
    FOREIGN KEY (updated_by) REFERENCES profile(id)
    ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('tag');



CREATE TABLE location_tag (
	location_id INTEGER NOT NULL,
	tag_id INTEGER NOT NULL,

	CONSTRAINT pk__location_tag
	PRIMARY KEY (location_id, tag_id),

	CONSTRAINT fk__location_tag__location_id
	FOREIGN KEY (location_id) REFERENCES location(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_tag__tag_id
	FOREIGN KEY (tag_id) REFERENCES tag(id)
	ON DELETE CASCADE
);



CREATE TABLE opening_time (
    id               SERIAL    PRIMARY KEY,
    location_id      INTEGER   NOT NULL,
	day              DATE      NOT NULL,
    start_time       TIME      NOT NULL,
    end_time         TIME      NOT NULL,
    seat_count       INTEGER,
    reservable_from  TIMESTAMP,
    reservable_until TIMESTAMP,
    created_at       TIMESTAMP NOT NULL    DEFAULT now(),
	created_by       INTEGER,
    updated_at       TIMESTAMP NOT NULL    DEFAULT now(),
	updated_by       INTEGER,

    CONSTRAINT fk_location_id
    FOREIGN KEY (location_id) REFERENCES location(id)
    ON DELETE CASCADE,

    CONSTRAINT fk__opening_time__created_by
    FOREIGN KEY (created_by) REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__opening_time__updated_by
    FOREIGN KEY (updated_by) REFERENCES profile(id)
    ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('opening_time');

CREATE INDEX idx__opening_time__location_id ON opening_time(location_id);
CREATE INDEX idx__opening_time__start_end ON opening_time(start_time, end_time);
CREATE INDEX idx__opening_time__day ON opening_time(day);



CREATE TYPE RESERVATION_STATE AS ENUM (
    'created',
    'cancelled',
    'absent',
    'present'
);

CREATE TABLE reservation (
	id               SERIAL            PRIMARY KEY,
	profile_id       INTEGER           NOT NULL,
	opening_time_id  INTEGER           NOT NULL,
	base_block_index INTEGER           NOT NULL,
	block_count      INTEGER           NOT NULL DEFAULT 1,
	state            RESERVATION_STATE NOT NULL DEFAULT 'created',
	created_at       TIMESTAMP         NOT NULL DEFAULT NOW(),
	updated_at       TIMESTAMP         NOT NULL DEFAULT NOW(),
	confirmed_at     TIMESTAMP,
	confirmed_by     INTEGER,

	CONSTRAINT fk__reservation__profile_id
	FOREIGN KEY (profile_id) REFERENCES profile(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__reservation__opening_time_id
	FOREIGN KEY (opening_time_id) REFERENCES opening_time(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__reservation__confirmed_by
	FOREIGN KEY (confirmed_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('reservation');

CREATE INDEX idx__reservation__profile_id ON reservation(profile_id);
CREATE INDEX idx__reservation__opening_time_id ON reservation(opening_time_id);
CREATE INDEX idx__reservation__confirmed_by ON reservation(confirmed_by);



CREATE TABLE image (
	id          SERIAL    PRIMARY KEY,
	file_path   TEXT,
	image_url   TEXT,
	uploaded_at TIMESTAMP NOT NULL     DEFAULT now(),
	uploaded_by INTEGER,

	CONSTRAINT fk_uploaded_by
	FOREIGN KEY (uploaded_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

ALTER TABLE profile
	ADD CONSTRAINT fk__profile__avatar_image_id
	FOREIGN KEY (avatar_image_id) REFERENCES image(id)
	ON DELETE SET NULL;



CREATE TABLE location_image (
	location_id INTEGER   NOT NULL,
	image_id    INTEGER   NOT NULL,
	index       INTEGER   NOT NULL DEFAULT 0,
	approved_at TIMESTAMP,
	approved_by INTEGER,

	CONSTRAINT pk__location_image
	PRIMARY KEY (location_id, image_id),

	CONSTRAINT fk__location_image__location_id
	FOREIGN KEY (location_id) REFERENCES location(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_image__image_id
	FOREIGN KEY (image_id) REFERENCES image(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_image__approved_by
	FOREIGN KEY (approved_by) REFERENCES profile(id)
	ON DELETE SET NULL
);



CREATE TABLE institution_role (
	id             SERIAL    PRIMARY KEY,
	institution_id INTEGER   NOT NULL,
	name           TEXT      NOT NULL,
	permissions    BIGINT    NOT NULL DEFAULT 0,
	created_at     TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by     INTEGER,
	updated_at     TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by     INTEGER,

	CONSTRAINT fk__institution_role__institution_id
	FOREIGN KEY (institution_id) REFERENCES institution(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution_role__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__institution_role__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT uniq__institution_role
	UNIQUE (institution_id, name)
);

SELECT diesel_manage_updated_at('institution_role');



CREATE TABLE authority_role (
	id           SERIAL    PRIMARY KEY,
	authority_id INTEGER   NOT NULL,
	name         TEXT      NOT NULL,
	permissions  BIGINT    NOT NULL DEFAULT 0,
	created_at   TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by   INTEGER,
	updated_at   TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by   INTEGER,

	CONSTRAINT fk__authority_role__authority_id
	FOREIGN KEY (authority_id) REFERENCES authority(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_role__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__authority_role__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT uniq__authority_role
	UNIQUE (authority_id, name)
);

SELECT diesel_manage_updated_at('authority_role');



CREATE TABLE location_role (
	id          SERIAL    PRIMARY KEY,
	location_id INTEGER   NOT NULL,
	name        TEXT      NOT NULL,
	permissions BIGINT    NOT NULL DEFAULT 0,
	created_at  TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by  INTEGER,
	updated_at  TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by  INTEGER,

	CONSTRAINT fk__location_role__location_id
	FOREIGN KEY (location_id) REFERENCES location(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_role__created_by
	FOREIGN KEY (created_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__location_role__updated_by
	FOREIGN KEY (updated_by) REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT uniq__location_role
	UNIQUE (location_id, name)
);

SELECT diesel_manage_updated_at('location_role');



CREATE TABLE institution_member_role (
	institution_member_id INTEGER   NOT NULL,
	institution_role_id   INTEGER   NOT NULL,
	added_at              TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by              INTEGER,

	CONSTRAINT pk__institution_member_role
	PRIMARY KEY (institution_member_id, institution_role_id),

	CONSTRAINT fk__institution_member_role__institution_member_id
	FOREIGN KEY (institution_member_id) REFERENCES institution_member(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution_member_role__institution_role_id
	FOREIGN KEY (institution_role_id) REFERENCES institution_role(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution_member_role__added_by
	FOREIGN KEY (added_by) REFERENCES profile(id)
	ON DELETE SET NULL
);



CREATE TABLE authority_member_role (
	authority_member_id INTEGER   NOT NULL,
	authority_role_id   INTEGER   NOT NULL,
	added_at            TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by            INTEGER,

	CONSTRAINT pk__authority_member_role
	PRIMARY KEY (authority_member_id, authority_role_id),

	CONSTRAINT fk__authority_member_role__authority_member_id
	FOREIGN KEY (authority_member_id) REFERENCES authority_member(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_member_role__authority_role_id
	FOREIGN KEY (authority_role_id) REFERENCES authority_role(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__authority_member_role__added_by
	FOREIGN KEY (added_by) REFERENCES profile(id)
	ON DELETE SET NULL
);



CREATE TABLE location_member_role (
	location_member_id INTEGER   NOT NULL,
	location_role_id   INTEGER   NOT NULL,
	added_at           TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by           INTEGER,

	CONSTRAINT pk__location_member_role
	PRIMARY KEY (location_member_id, location_role_id),

	CONSTRAINT fk__location_member_role__location_member_id
	FOREIGN KEY (location_member_id) REFERENCES location_member(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_member_role__location_role_id
	FOREIGN KEY (location_role_id) REFERENCES location_role(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_member_role__added_by
	FOREIGN KEY (added_by) REFERENCES profile(id)
	ON DELETE SET NULL
);

CREATE VIEW institution_member_permissions AS
	SELECT m.*, bit_or(r.permissions) as permissions
	FROM institution_member m
	LEFT JOIN institution_member_role mr
		ON mr.institution_member_id = m.id
	INNER JOIN institution_role r
		ON r.id = mr.institution_role_id
	GROUP BY m.id;



CREATE VIEW authority_member_permissions AS
	SELECT m.*, bit_or(r.permissions) as permissions
	FROM authority_member m
	LEFT JOIN authority_member_role mr
		ON mr.authority_member_id = m.id
	INNER JOIN authority_role r
		ON r.id = mr.authority_role_id
	GROUP BY m.id;



CREATE VIEW location_member_permissions AS
	SELECT m.*, bit_or(r.permissions) as permissions
	FROM location_member m
	LEFT JOIN location_member_role mr
		ON mr.location_member_id = m.id
	INNER JOIN location_role r
		ON r.id = mr.location_role_id
	GROUP BY m.id;
