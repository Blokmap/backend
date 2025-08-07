--- Create the status enum
DROP TYPE IF EXISTS reservation_state;
CREATE TYPE reservation_state AS ENUM (
    'created',
    'cancelled',
    'absent',
    'present'
);

--- Add the new enum column to `reservation`
ALTER TABLE reservation 
    ADD COLUMN IF NOT EXISTS state reservation_state NOT NULL DEFAULT 'created';