CREATE TABLE IF NOT EXISTS families (
    id SERIAL   PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS family_members (
    id SERIAL   PRIMARY KEY,
    family_id   INTEGER REFERENCES families(id),
    user_id     INTEGER REFERENCES users(id),
    role        VARCHAR(100),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);