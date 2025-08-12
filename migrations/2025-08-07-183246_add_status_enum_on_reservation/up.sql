--- Create the status enum
CREATE TYPE reservation_state IF NOT EXISTS AS ENUM (
    'created',
    'cancelled',
    'absent',
    'present'
);

--- Add the new enum column to `reservation`
ALTER TABLE reservation
	ADD COLUMN IF NOT EXISTS
	state reservation_state NOT NULL DEFAULT 'created';
