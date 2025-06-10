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
	FOREIGN KEY (updated_by)
	REFERENCES profile(id)
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
	FOREIGN KEY (created_by)
	REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__translation__updated_by
	FOREIGN KEY (updated_by)
	REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('translation');



CREATE TABLE institution (
	id                  SERIAL     PRIMARY KEY,
    name_translation_id INTEGER    NOT NULL,
    slug_translation_id INTEGER    NOT NULL,
	email               TEXT       COLLATE "case_insensitive" UNIQUE,
	phone_number        TEXT       COLLATE "case_insensitive",
    street              TEXT       COLLATE "case_insensitive",
	number              TEXT       COLLATE "case_insensitive",
    zip                 TEXT       COLLATE "case_insensitive",
	city                TEXT       COLLATE "case_insensitive",
	province            TEXT       COLLATE "case_insensitive",
	country             VARCHAR(2) COLLATE "case_insensitive",
    created_at          TIMESTAMP  NOT NULL DEFAULT NOW(),
	created_by          INTEGER,
    updated_at          TIMESTAMP  NOT NULL DEFAULT NOW(),
	updated_by          INTEGER,

	CONSTRAINT fk__institution__name_translation_id
	FOREIGN KEY (name_translation_id)
	REFERENCES translation(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution__slug_translation_id
	FOREIGN KEY (slug_translation_id)
	REFERENCES translation(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__institution__created_by
	FOREIGN KEY (created_by)
	REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__institution__updated_by
	FOREIGN KEY (updated_by)
	REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('institution');

ALTER TABLE profile
ADD CONSTRAINT fk__profile__institution_id FOREIGN KEY (institution_id)
REFERENCES institution(id) ON DELETE SET NULL;



CREATE TABLE authority (
    id          SERIAL    PRIMARY KEY,
    name        TEXT      NOT NULL,
    description TEXT,
    created_at  TIMESTAMP NOT NULL DEFAULT NOW(),
	created_by  INTEGER,
    updated_at  TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by  INTEGER,

	CONSTRAINT fk__authority__created_by
	FOREIGN KEY (created_by)
	REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk__authority__updated_by
	FOREIGN KEY (updated_by)
	REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('authority');



CREATE TABLE authority_profile (
	authority_id INTEGER   NOT NULL,
	profile_id   INTEGER   NOT NULL,
	added_at     TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by     INTEGER,
	updated_at   TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by   INTEGER,
	permissions  BIGINT    NOT NULL DEFAULT 0,

	CONSTRAINT pk__authority_profile
	PRIMARY KEY (authority_id, profile_id),

	CONSTRAINT fk__authority_profile__authority_id
	FOREIGN KEY (authority_id)
	REFERENCES authority(id)
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

SELECT diesel_manage_updated_at('authority_profile');



CREATE TABLE location (
    id                     SERIAL  PRIMARY KEY,
    name                   TEXT    NOT NULL,
	authority_id           INTEGER,
    description_id         INTEGER NOT NULL,
    excerpt_id             INTEGER NOT NULL,
    seat_count             INTEGER NOT NULL,
    is_reservable          BOOLEAN NOT NULL,
    reservation_block_size INTEGER NOT NULL,
	min_reservation_length INTEGER,
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
	FOREIGN KEY (authority_id)
	REFERENCES authority(id)
	ON DELETE SET NULL,

    CONSTRAINT fk__location__description_id
    FOREIGN KEY (description_id)
    REFERENCES translation(id)
    ON DELETE CASCADE,

    CONSTRAINT fk__location__excerpt_id
    FOREIGN KEY (excerpt_id)
    REFERENCES translation(id)
    ON DELETE CASCADE,

    CONSTRAINT fk__location__approved_by
    FOREIGN KEY (approved_by)
    REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__location__rejected_by
    FOREIGN KEY (created_by)
    REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__location__created_by
    FOREIGN KEY (created_by)
    REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__location__updated_by
    FOREIGN KEY (updated_by)
    REFERENCES profile(id)
    ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('location');



CREATE TABLE location_profile (
	location_id INTEGER   NOT NULL,
	profile_id  INTEGER   NOT NULL,
	permissions BIGINT    NOT NULL DEFAULT 0,
	added_at    TIMESTAMP NOT NULL DEFAULT NOW(),
	added_by    INTEGER,
	updated_at  TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_by  INTEGER,

	CONSTRAINT pk__location_profile
	PRIMARY KEY (location_id, profile_id),

	CONSTRAINT fk__location_profile__location_id
	FOREIGN KEY (location_id)
	REFERENCES location(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_profile__profile_id
	FOREIGN KEY (profile_id)
	REFERENCES profile(id)
	ON DELETE CASCADE,

    CONSTRAINT fk__tag__added_by
    FOREIGN KEY (added_by)
    REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__tag__updated_by
    FOREIGN KEY (updated_by)
    REFERENCES profile(id)
    ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('location_profile');



CREATE TABLE review (
	id          SERIAL    PRIMARY KEY,
	profile_id  INTEGER   NOT NULL,
	location_id INTEGER   NOT NULL,
	rating      INTEGER   NOT NULL CHECK (0 <= rating AND rating <= 5),
	body        TEXT,
	created_at  TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_at  TIMESTAMP NOT NULL DEFAULT NOW(),

	CONSTRAINT fk__review__profile_id
	FOREIGN KEY (profile_id)
	REFERENCES profile(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__review__location_id
	FOREIGN KEY (location_id)
	REFERENCES location(id)
	ON DELETE CASCADE
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
	FOREIGN KEY (name_translation_id)
	REFERENCES translation(id)
	ON DELETE CASCADE,

    CONSTRAINT fk__tag__created_by
    FOREIGN KEY (created_by)
    REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__tag__updated_by
    FOREIGN KEY (updated_by)
    REFERENCES profile(id)
    ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('tag');



CREATE TABLE location_tag (
	location_id INTEGER NOT NULL,
	tag_id INTEGER NOT NULL,

	CONSTRAINT pk__location_tag
	PRIMARY KEY (location_id, tag_id),

	CONSTRAINT fk__location_tag__location_id
	FOREIGN KEY (location_id)
	REFERENCES location(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_tag__tag_id
	FOREIGN KEY (tag_id)
	REFERENCES tag(id)
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
    FOREIGN KEY (location_id)
    REFERENCES location(id)
    ON DELETE CASCADE,

    CONSTRAINT fk__opening_time__created_by
    FOREIGN KEY (created_by)
    REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk__opening_time__updated_by
    FOREIGN KEY (updated_by)
    REFERENCES profile(id)
    ON DELETE SET NULL
);

CREATE INDEX idx__opening_time__location_id ON opening_time(location_id);
CREATE INDEX idx__opening_time__start_end ON opening_time(start_time, end_time);

SELECT diesel_manage_updated_at('opening_time');



CREATE TABLE reservation (
	id               SERIAL    PRIMARY KEY,
	profile_id       INTEGER   NOT NULL,
	opening_time_id  INTEGER   NOT NULL,
	base_block_index INTEGER   NOT NULL,
	block_count      INTEGER   NOT NULL DEFAULT 1,
	created_at       TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_at       TIMESTAMP NOT NULL DEFAULT NOW(),
	confirmed_at     TIMESTAMP,
	confirmed_by     INTEGER,

	CONSTRAINT fk__reservation__profile_id
	FOREIGN KEY (profile_id)
	REFERENCES profile(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__reservation__opening_time_id
	FOREIGN KEY (opening_time_id)
	REFERENCES opening_time(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__reservation__confirmed_by
	FOREIGN KEY (confirmed_by)
	REFERENCES profile(id)
	ON DELETE SET NULL
);

SELECT diesel_manage_updated_at('reservation');



CREATE TABLE image (
	id          SERIAL    PRIMARY KEY,
	file_path   TEXT      NOT NULL,
	uploaded_at TIMESTAMP NOT NULL     DEFAULT now(),
	uploaded_by INTEGER   NOT NULL,

	CONSTRAINT fk_uploaded_by
	FOREIGN KEY (uploaded_by)
	REFERENCES profile(id)
	ON DELETE SET NULL
);

ALTER TABLE profile
ADD CONSTRAINT fk__profile__avatar_image_id FOREIGN KEY (avatar_image_id)
REFERENCES image(id) ON DELETE SET NULL;



CREATE VIEW simple_profile AS
	SELECT
		p.id, p.username,
		img.file_path AS avatar_url,
		p.email, p.first_name, p.last_name,
		p.state
	FROM profile p
	LEFT OUTER JOIN image img
	ON p.avatar_image_id = img.id;




CREATE TABLE location_image (
	location_id INTEGER   NOT NULL,
	image_id    INTEGER   NOT NULL,
	approved_at TIMESTAMP,
	approved_by INTEGER,

	CONSTRAINT pk__location_image
	PRIMARY KEY (location_id, image_id),

	CONSTRAINT fk__location_image__location_id
	FOREIGN KEY (location_id)
	REFERENCES location(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_image__image_id
	FOREIGN KEY (image_id)
	REFERENCES image(id)
	ON DELETE CASCADE,

	CONSTRAINT fk__location_image__approved_by
	FOREIGN KEY (approved_by)
	REFERENCES profile(id)
	ON DELETE SET NULL
);
