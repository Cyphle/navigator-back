CREATE TABLE IF NOT EXISTS bank_accounts(
    id SERIAL       PRIMARY KEY,
    owner_id        INTEGER NOT NULL,
    family_id       INTEGER,
    name            VARCHAR(255) NOT NULL,
    description     VARCHAR(255),
    starting_amount DECIMAL(20, 2) NOT NULL,
    start_date      DATE NOT NULL
);

ALTER TABLE IF EXISTS bank_accounts
    ADD CONSTRAINT bank_accounts_owner_id_fkey
    FOREIGN KEY (owner_id)
    REFERENCES users(id);

CREATE TABLE IF NOT EXISTS budgets(
    id                  SERIAL PRIMARY KEY,
    bank_account_id     INTEGER NOT NULL,
    name                VARCHAR(255) NOT NULL,
    description         VARCHAR(255),
    start_date          DATE NOT NULL,
    end_date            DATE,
    initial_amount      DECIMAL(20, 2) NOT NULL
);

ALTER TABLE IF EXISTS budgets
    ADD CONSTRAINT budgets_bank_account_id_fkey
    FOREIGN KEY (bank_account_id)
    REFERENCES bank_accounts(id);

CREATE TABLE IF NOT EXISTS transactions(
    ID  SERIAL          PRIMARY KEY,
    bank_account_id     INTEGER NOT NULL,
    budget_id           INTEGER,
    type                VARCHAR(10) NOT NULL,
    periodicity         VARCHAR(10),
    description         VARCHAR(255) NOT NULL,
    start_date          DATE NOT NULL,
    end_date            DATE NOT NULL,
    amount              DECIMAL(20, 2) NOT NULL
);

ALTER TABLE IF EXISTS transactions
    ADD CONSTRAINT transactions_bank_account_id_fkey
    FOREIGN KEY (bank_account_id)
    REFERENCES bank_accounts(id);

ALTER TABLE IF EXISTS transactions
    ADD CONSTRAINT transactions_budget_id_fkey
    FOREIGN KEY (budget_id)
    REFERENCES budgets(id);

