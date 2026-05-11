---
name: sqlx-migration
description: Use when authoring, reviewing, or about to run a SQLx migration — especially before `sqlx migrate run` against any production-adjacent environment. Covers naming/ordering, FK-safe drops, concurrent index creation, NOT NULL column rollouts, idempotent backfills, reversibility, and keeping the Rust side in sync (`cargo sqlx prepare`). Triggers on keywords like migration, sqlx migrate, ALTER TABLE, ADD COLUMN, DROP COLUMN, CREATE INDEX, foreign key, FK, backfill, schema change, rollback, CONCURRENTLY.
allowed-tools: Read, Grep, Glob, Edit, Write, Bash
---

# SQLx migration safety

## When to use

- Writing a new file under `migrations/`
- Reviewing pending migrations before `sqlx migrate run`
- Modifying schema in a way that touches production data
- Investigating why a migration failed or rolled back

## Process

1. List migrations in `migrations/` (newest filenames last alphabetically thanks to the
   `YYYYMMDDHHMMSS_` prefix).
2. Identify which are pending. If a DB is reachable, compare against `_sqlx_migrations`:
   ```bash
   sqlx migrate info
   ```
3. Read each pending migration in full.
4. Walk the safety checklist below — every item, every time.
5. Decide: safe / safe-with-prereqs / do-not-run, and report findings.

## Safety checklist

### Naming & ordering

- File matches `XXXX_description.sql` (UTC timestamp, snake_case description).
- Strict run order — never insert a migration "between" two existing ones; increment by one the new script.
- A drop migration must respect FK direction: **drop the referencing table / FK first**,
  then the referenced table.
- No migration assumes state from a *later* migration (no forward references).

### DDL safety

- `CREATE INDEX` on any table that may be large in prod → use `CREATE INDEX CONCURRENTLY`
  and run **outside** a transaction (`-- sqlx:no-transaction` if your tooling allows,
  or split the index creation out of any wrapping `BEGIN`).
- `ALTER TABLE … ADD COLUMN … NOT NULL` is dangerous on a populated table. Either:
  1. Provide a constant `DEFAULT` (PG ≥11 makes this fast), **or**
  2. Split into three migrations: add nullable → backfill (idempotent, batched) → `SET NOT NULL`.
- `DROP COLUMN` on a large/active table: confirm the application has already stopped
  reading it (deploy ordering matters). Prefer a "deprecate first, drop later" cycle. If column is not empty, use a 'archive' column `archive_deleted` that stores the data and key in JSON `{ <column_name>_delete: <value> }`.
- `ALTER TABLE … ALTER COLUMN TYPE …` may rewrite the table — flag for off-hours.
- No `TRUNCATE` without explicit human approval. Same for `DROP TABLE` when data exists.
- `ADD CONSTRAINT … FOREIGN KEY …` validates existing rows by default — for big tables,
  prefer `NOT VALID` then `VALIDATE CONSTRAINT` in a follow-up migration.
- New `CHECK` constraints on big tables: same pattern (`NOT VALID` then `VALIDATE`).

### DML in migrations

- Backfills must be **idempotent** (re-runnable safely — use `WHERE col IS NULL` or
  similar guards).
- Long `UPDATE`s on big tables must be **batched**
  (`WHERE id BETWEEN $low AND $high`, looped from app code or a follow-up script — not
  a single migration scanning millions of rows under one lock).
- Backfills on tables with live traffic need a plan: lock-light batches, throttle, or
  do it from a script outside the migration.
- No `INSERT` of data the app should own (seed scripts belong elsewhere, not in
  migrations — except true reference data with stable IDs).

### Reversibility

- Comment at the top of the file documenting the **rollback plan** (or "no rollback —
  destructive change, requires restore from backup").
- For destructive changes, confirm a backup is taken (prod) before run.
- Avoid mixing reversible DDL and irreversible data deletion in the same migration —
  separate them so a partial failure leaves a clean state.

### Consistency with Rust code

- Any new/changed column has matching struct updates in `crates/infrastructure/` —
  otherwise compile-time query checks will explode.
- `CHECK` constraints align with the matching domain invariants
  (don't let the DB allow values the domain forbids, or vice-versa).
- After applying the migration locally:
  ```bash
  cargo sqlx prepare --workspace
  ```
  Commit the resulting `.sqlx/` files. CI relies on them.

### Pre-run checklist (production)

- [ ] Backup taken
- [ ] Migration tested on staging against a prod-shaped dataset
- [ ] Rollback plan documented in the migration file header
- [ ] Team notified, deploy window agreed
- [ ] App version that *requires* the new schema is **not yet** deployed (so the old app
      still runs against the new schema), or the migration is backwards-compatible
- [ ] `cargo sqlx prepare --workspace` will run as part of the same change set

## Common patterns

### Add NOT NULL column safely (3-step)

```sql
-- 0001_add_users_status_step1.sql
ALTER TABLE users ADD COLUMN status TEXT;
```

```sql
-- 0002_add_users_status_step2.sql
-- Idempotent backfill. Safe to re-run.
UPDATE users SET status = 'active' WHERE status IS NULL;
```

```sql
-- 0003_add_users_status_step3.sql
ALTER TABLE users ALTER COLUMN status SET NOT NULL;
ALTER TABLE users ADD CONSTRAINT users_status_check CHECK (status IN ('active','suspended','deleted'));
```

### Concurrent index (must be outside a transaction)

```sql
-- 004_add_users_email_index.sql
-- Run outside a transaction. `sqlx migrate run` defaults to wrapping in one;
-- this file must opt out (project convention: filename suffix `_no_tx`, or a marker
-- comment your runner respects). Verify before merging.
CREATE INDEX CONCURRENTLY IF NOT EXISTS users_email_idx ON users (email);
```

### FK on a large table without long lock

```sql
-- 0005_add_orders_user_fk.sql
ALTER TABLE orders
  ADD CONSTRAINT orders_user_id_fkey
  FOREIGN KEY (user_id) REFERENCES users(id)
  NOT VALID;
```

```sql
-- 0006_validate_orders_user_fk.sql
ALTER TABLE orders VALIDATE CONSTRAINT orders_user_id_fkey;
```

## Reporting format (when reviewing)

```
## Migration Review

### Pending migrations
1. <filename>
2. <filename>

### 🔴 Blocking
- [file] [issue] → [fix]

### ⚠️ Warnings
- [file] [concern]

### 📋 Pre-run checklist
- [ ] Backup taken (prod)
- [ ] Tested on staging
- [ ] Rollback plan documented
- [ ] Team notified (prod)
- [ ] `cargo sqlx prepare --workspace` will run after

### Verdict
[ ] SAFE to run
[ ] SAFE with prerequisites met
[ ] DO NOT RUN — fix issues first
```

## Never do

- ❌ `CREATE INDEX` (non-concurrent) on a large prod table
- ❌ `ADD COLUMN ... NOT NULL` without DEFAULT or 3-step rollout
- ❌ Single-statement backfill scanning millions of rows under one lock
- ❌ Back-dating a timestamp to slot a migration between existing ones
- ❌ Mixing destructive DML and DDL in one migration
- ❌ Forgetting `cargo sqlx prepare --workspace` after schema change
- ❌ `DROP COLUMN` without archiving it in `archive_deleted` column

## Always do

- ✅ Verify FK direction before any drop
- ✅ Make backfills idempotent and batched
- ✅ Document rollback plan at the top of every migration
- ✅ Re-run `cargo sqlx prepare --workspace` and commit `.sqlx/`
- ✅ Test against a prod-shaped dataset on staging before prod
