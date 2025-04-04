CREATE TABLE translation (
	id         SERIAL    PRIMARY KEY,
	nl         TEXT,
    en         TEXT,
    fr         TEXT,
    de         TEXT,
	created_at TIMESTAMP NOT NULL     DEFAULT now(),
	updated_at TIMESTAMP NOT NULL     DEFAULT now()
);

-- Automatically update `updated_at`
SELECT diesel_manage_updated_at('translation');