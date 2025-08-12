--- Drop the status column from the reservation table
ALTER TABLE reservation DROP COLUMN state;

--- Drop the status enum type
DROP TYPE reservation_state;
