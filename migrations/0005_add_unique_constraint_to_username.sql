-- Add UNIQUE constraint to username to support get_or_create logic
ALTER TABLE users ADD CONSTRAINT users_username_key UNIQUE (username);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);