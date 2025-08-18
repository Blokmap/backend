ALTER TABLE authority ADD COLUMN IF NOT EXISTS institution_id INTEGER;

ALTER TABLE authority
	ADD CONSTRAINT fk__authority__institution_id FOREIGN KEY (institution_id)
	REFERENCES institution(id) ON DELETE SET NULL;
