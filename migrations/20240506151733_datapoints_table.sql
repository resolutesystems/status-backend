CREATE TABLE datapoints (
    id SERIAL PRIMARY KEY,
    cpu REAL NOT NULL,
    memory REAL NOT NULL,
    transmitted DOUBLE PRECISION NOT NULL,
    received DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
