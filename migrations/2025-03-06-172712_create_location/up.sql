-- Create locations table
CREATE TABLE location (
    id             SERIAL           PRIMARY KEY,
    name           TEXT             NOT NULL,
    description_id INTEGER          NOT NULL,
    excerpt_id     INTEGER          NOT NULL,
    seat_count     INTEGER          NOT NULL,
    is_reservable  BOOLEAN          NOT NULL,
    is_visible     BOOLEAN          NOT NULL,
    street         TEXT             NOT NULL,
    number         TEXT             NOT NULL,
    zip            TEXT             NOT NULL,
    city           TEXT             NOT NULL,
    province       TEXT             NOT NULL,
    latitude       DOUBLE PRECISION NOT NULL,
    longitude      DOUBLE PRECISION NOT NULL,
    created_by_id  INTEGER          NOT NULL,
    approved_by_id INTEGER,
    approved_at    TIMESTAMP,
    created_at     TIMESTAMP        NOT NULL    DEFAULT now(),
    updated_at     TIMESTAMP        NOT NULL    DEFAULT now(),

    CONSTRAINT fk_created_by_id
    FOREIGN KEY (created_by_id)
    REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk_approved_by_id
    FOREIGN KEY (approved_by_id)
    REFERENCES profile(id)
    ON DELETE SET NULL,

    CONSTRAINT fk_description_id
    FOREIGN KEY (description_id)
    REFERENCES translation(id)
    ON DELETE CASCADE,

    CONSTRAINT fk_excerpt_id
    FOREIGN KEY (excerpt_id)
    REFERENCES translation(id)
    ON DELETE CASCADE
);

-- Automatically update `updated_at`
SELECT diesel_manage_updated_at('location');

-- Create opening_times table
CREATE TABLE opening_time (
    id            SERIAL    PRIMARY KEY,
    location_id   INTEGER   NOT NULL,
    start_time    TIMESTAMP NOT NULL,
    end_time      TIMESTAMP NOT NULL,
    seat_count    INTEGER,
    is_reservable BOOLEAN,
    created_at    TIMESTAMP NOT NULL    DEFAULT now(),
    updated_at    TIMESTAMP NOT NULL    DEFAULT now(),

    CONSTRAINT fk_location_id
    FOREIGN KEY (location_id)
    REFERENCES location(id)
    ON DELETE CASCADE
);

-- Indexes for opening_times
CREATE INDEX idx_opening_times_location_id ON opening_time(location_id);
CREATE INDEX idx_opening_times_start_end ON opening_time(start_time, end_time);

-- Automatically update `updated_at`
SELECT diesel_manage_updated_at('opening_time');

CREATE VIEW filled_location AS
	SELECT
		l.id, l.name, l.seat_count, l.is_reservable, l.is_visible, l.street,
		l.number, l.zip, l.city, l.province, l.latitude, l.longitude,
		l.created_by_id, l.approved_by_id, l.approved_at, l.created_at,
		l.updated_at,
		JSONB_BUILD_OBJECT(
			'nl', d.nl,
			'en', d.en,
			'fr', d.fr,
			'de', d.de,
			'created_at', d.created_at,
			'updated_at', d.updated_at
		) AS description,
		JSONB_BUILD_OBJECT(
			'nl', e.nl,
			'en', e.en,
			'fr', e.fr,
			'de', e.de,
			'created_at', e.created_at,
			'updated_at', e.updated_at
		) AS excerpt,
		JSONB_AGG(JSONB_BUILD_OBJECT(
			'start_time', t.start_time,
			'end_time', t.end_time,
			'seat_count', t.seat_count,
			'is_reservable', t.is_reservable,
			'created_at', t.created_at,
			'updated_at', t.updated_at
		)) AS opening_times
	FROM location l
	INNER JOIN translation d
		ON l.description_id = d.id
	INNER JOIN translation e
		ON l.excerpt_id = e.id
	INNER JOIN opening_time t
		ON t.location_id = l.id
	GROUP BY
		l.id, l.name, l.seat_count, l.is_reservable, l.is_visible, l.street,
		l.number, l.zip, l.city, l.province, l.latitude, l.longitude,
		l.created_by_id, l.approved_by_id, l.approved_at, l.created_at,
		l.updated_at,
		d.nl, d.en, d.fr, d.de, d.created_at, d.updated_at,
		e.nl, e.en, e.fr, e.de, e.created_at, e.updated_at;
