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

CREATE TABLE location_image (
	id          SERIAL    PRIMARY KEY,
	location_id INTEGER   NOT NULL,
	file_path   TEXT      NOT NULL,
	uploaded_at TIMESTAMP NOT NULL     DEFAULT now(),
	uploaded_by INTEGER   NOT NULL,
	approved_at TIMESTAMP,
	approved_by INTEGER,

	CONSTRAINT fk_location_id
	FOREIGN KEY (location_id)
	REFERENCES location(id)
	ON DELETE CASCADE,

	CONSTRAINT fk_uploaded_by
	FOREIGN KEY (uploaded_by)
	REFERENCES profile(id)
	ON DELETE SET NULL,

	CONSTRAINT fk_approved_by
	FOREIGN KEY (approved_by)
	REFERENCES profile(id)
	ON DELETE SET NULL
)
