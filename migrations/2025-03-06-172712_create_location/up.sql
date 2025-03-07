-- Create locations table
CREATE TABLE location (
    id              SERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    description_key UUID NOT NULL,
    excerpt_key     UUID NOT NULL,
    seat_count      INTEGER NOT NULL,
    is_reservable   BOOLEAN NOT NULL,
    is_visible         BOOLEAN NOT NULL,
    street          TEXT NOT NULL,
    number          TEXT NOT NULL,
    zip             TEXT NOT NULL,
    city            TEXT NOT NULL,
    province        TEXT NOT NULL,
    latitude        DOUBLE PRECISION NOT NULL,
    longitude       DOUBLE PRECISION NOT NULL,
    cell_idx        INTEGER NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for locations
CREATE INDEX idx_location_cell ON location(cell_idx);

-- Automatically update `updated_at`
SELECT diesel_manage_updated_at('location');

-- Create opening_times table
CREATE TABLE opening_time (
    id            SERIAL PRIMARY KEY,
    location_id   INTEGER NOT NULL REFERENCES location(id) ON DELETE CASCADE,
    start_time    TIMESTAMPTZ NOT NULL,
    end_time      TIMESTAMPTZ NOT NULL,
    seat_count    INTEGER,
    is_reservable BOOLEAN,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for opening_times
CREATE INDEX idx_opening_times_location_id ON opening_time(location_id);
CREATE INDEX idx_opening_times_start_end ON opening_time(start_time, end_time);

-- Automatically update `updated_at`
SELECT diesel_manage_updated_at('opening_time');
