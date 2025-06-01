CREATE TYPE PROFILE_STATE AS ENUM ('pending_email_verification', 'active', 'disabled');

CREATE TABLE profile (
	id                              SERIAL        PRIMARY KEY,
	username                        TEXT          COLLATE "case_insensitive" NOT NULL UNIQUE,
	first_name                      TEXT,
	last_name                       TEXT,
	avatar_image_id                 INTEGER,
	institution_name                TEXT,
	password_hash                   TEXT          NOT NULL,
	password_reset_token            TEXT          UNIQUE           DEFAULT NULL,
	password_reset_token_expiry     TIMESTAMP                      DEFAULT NULL,
	email                           TEXT          COLLATE "case_insensitive" UNIQUE           DEFAULT NULL,
	pending_email                   TEXT          COLLATE "case_insensitive" UNIQUE,
	email_confirmation_token        TEXT          UNIQUE,
	email_confirmation_token_expiry TIMESTAMP,
	is_admin                        BOOLEAN       NOT NULL         DEFAULT false,
	block_reason                    TEXT,
	state                           PROFILE_STATE NOT NULL         DEFAULT 'pending_email_verification',
	created_at                      TIMESTAMP     NOT NULL         DEFAULT NOW(),
	updated_at                      TIMESTAMP     NOT NULL         DEFAULT NOW(),
	updated_by                      INTEGER,
	last_login_at                   TIMESTAMP     NOT NULL         DEFAULT NOW(),

	CONSTRAINT fk_profile_updated_by
	FOREIGN KEY (updated_by)
	REFERENCES profile(id)
	ON DELETE SET NULL
);

-- Automatically update `updated_at`
SELECT diesel_manage_updated_at('profile');
