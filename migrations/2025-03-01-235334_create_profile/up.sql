CREATE TYPE PROFILE_STATE AS ENUM ('pending_email_verification', 'active', 'disabled');

CREATE TABLE profile (
	id                              SERIAL        PRIMARY KEY,
	username                        TEXT          COLLATE "case_insensitive" NOT NULL UNIQUE,
	password_hash                   TEXT          NOT NULL,
	password_reset_token            TEXT          UNIQUE           DEFAULT NULL,
	password_reset_token_expiry     TIMESTAMP                      DEFAULT NULL,
	email                           TEXT          COLLATE "case_insensitive" UNIQUE           DEFAULT NULL,
	pending_email                   TEXT          COLLATE "case_insensitive" UNIQUE,
	email_confirmation_token        TEXT          UNIQUE,
	email_confirmation_token_expiry TIMESTAMP,
	admin                           BOOLEAN       NOT NULL         DEFAULT false,
	state                           PROFILE_STATE NOT NULL         DEFAULT 'pending_email_verification',
	created_at                      TIMESTAMP     NOT NULL         DEFAULT NOW(),
	last_login_at                   TIMESTAMP     NOT NULL         DEFAULT NOW()
);
