DROP TABLE IF EXISTS families;
DROP TABLE IF EXISTS family_members;

ALTER TABLE IF EXISTS users CREATE INDEX idx_users_username ON username;

CREATE TABLE IF NOT EXISTS families (
    id SERIAL   PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    creator_id  INTEGER,
    active      BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE IF EXISTS families
   ADD CONSTRAINT families_creator_id_fkey
   FOREIGN KEY (creator_id)
   REFERENCES users(id);

CREATE TABLE IF NOT EXISTS family_members (
    id SERIAL   PRIMARY KEY,
    family_id   INTEGER REFERENCES families(id),
    user_id     INTEGER REFERENCES users(id),
    relation    VARCHAR(100),
    is_admin    BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


