ALTER TABLE IF EXISTS bank_accounts ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
ALTER TABLE IF EXISTS bank_accounts ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

ALTER TABLE IF EXISTS bank_accounts
    ADD CONSTRAINT bank_accounts_family_id_fkey
    FOREIGN KEY (family_id)
    REFERENCES families(id);


ALTER TABLE DROP CONSTRAINT IF EXISTS budgets_bank_account_id_fkey;
ALTER TABLE IF EXISTS budgets
    ADD CONSTRAINT budgets_bank_account_id_fkey
    FOREIGN KEY (bank_account_id)
    REFERENCES bank_accounts(id)
    ON DELETE CASCADE;

ALTER TABLE DROP CONSTRAINT IF EXISTS transactions_bank_account_id_fkey;
ALTER TABLE IF EXISTS transactions
    ADD CONSTRAINT transactions_bank_account_id_fkey
    FOREIGN KEY (bank_account_id)
    REFERENCES bank_accounts(id)
    ON DELETE CASCADE;

ALTER TABLE DROP CONSTRAINT IF EXISTS transactions_budget_id_fkey;
ALTER TABLE IF EXISTS transactions
    ADD CONSTRAINT transactions_budget_id_fkey
    FOREIGN KEY (budget_id)
    REFERENCES budgets(id)
    ON DELETE CASCADE;

-- Magic List
CREATE TABLE IF NOT EXISTS magic_list (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(30) NOT NULL,
    visibility VARCHAR(30) NOT NULL,
    owner_id: INTEGER NOT NULL,
    family_id INTEGER,
    excluded_user_ids INTEGER[],
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE IF EXISTS magic_list ADD CONSTRAINT owner_id_fkey FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE IF EXISTS magic_list ADD CONSTRAINT family_id_fkey FOREIGN KEY (family_id) REFERENCES families(id) ON DELETE CASCADE;

CREATE TABLE IF NOT EXISTS magic_list_item (
    id SERIAL PRIMARY KEY,
    magic_list_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    checked BOOLEAN DEFAULT FALSE,
    due_date DATE,
    status VARCHAR(30),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE IF EXISTS magic_list_item ADD CONSTRAINT magic_list_id_fkey FOREIGN KEY (magic_list_id) REFERENCES magic_list(id) ON DELETE CASCADE;
