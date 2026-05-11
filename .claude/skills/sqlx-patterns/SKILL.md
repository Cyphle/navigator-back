---
name: sqlx-patterns
description: Use when writing SQLx queries, migrations, or database tests. Covers compile-time checked queries, batch operations with ANY/UNNEST, transactions, migration ordering, and sqlx::test fixtures. Triggers on keywords like SQLx, query, migration, batch insert, ANY, UNNEST, sqlx test, fetch_all, foreign key.
allowed-tools: Read, Grep, Glob, Edit, Write, Bash
---

# SQLx Patterns

## Batch reads: `= ANY($1)`

Avoid N queries or `IN (?, ?, ?)`:

```rust
pub async fn find_users_by_ids(
    conn: &mut PgConnection,
    ids: &[UserId],
) -> Result<Vec<User>, DomainError> {
    let raw_ids: Vec<Uuid> = ids.iter().map(|id| id.0).collect();
    sqlx::query_as!(
        User,
        r#"SELECT id as "id: _", email, created_at FROM users WHERE id = ANY($1)"#,
        &raw_ids
    )
    .fetch_all(conn)
    .await
    .map_err(|e| DomainError::Infra(e.to_string()))
}
```

## Batch inserts: `UNNEST`

Single round-trip bulk insert:

```rust
pub async fn insert_many_users(
    conn: &mut PgConnection,
    users: &[User],
) -> Result<(), DomainError> {
    let ids: Vec<Uuid> = users.iter().map(|u| u.id.0).collect();
    let emails: Vec<&str> = users.iter().map(|u| u.email.as_str()).collect();
    let created_ats: Vec<DateTime<Utc>> = users.iter().map(|u| u.created_at).collect();

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, created_at)
        SELECT * FROM UNNEST($1::uuid[], $2::text[], $3::timestamptz[])
        "#,
        &ids,
        &emails as &[&str],
        &created_ats
    )
    .execute(conn)
    .await
    .map_err(|e| DomainError::Infra(e.to_string()))?;
    Ok(())
}
```

## Migrations

### Naming & ordering

```bash
sqlx migrate add create_users
sqlx migrate add add_user_email_index
sqlx migrate run
```

Pattern: `xxxx_description.sql`

### FK-aware drops

Drop referencing tables FIRST:

```sql
-- 0001_drop_order_related.sql
DROP TABLE IF EXISTS order_items;  -- references orders
DROP TABLE IF EXISTS orders;       -- references users
-- users can stay or drop next
```

### Safe index creation

```sql
CREATE INDEX CONCURRENTLY idx_users_email ON users(email);
-- CONCURRENTLY = no table lock, safe in production.
-- CANNOT be inside a transaction block.
```

### Non-destructive column additions

```sql
-- Step 1: add nullable
ALTER TABLE users ADD COLUMN status TEXT;

-- Step 2 (next migration): backfill
UPDATE users SET status = 'active' WHERE status IS NULL;

-- Step 3 (next migration): enforce
ALTER TABLE users ALTER COLUMN status SET NOT NULL;
```

## Transactions

Pass `&mut PgConnection` to functions needing to join a transaction:

```rust
let mut tx = pool.begin().await?;

repo_a.save(&mut *tx, &entity_a).await?;
repo_b.save(&mut *tx, &entity_b).await?;

tx.commit().await?;
```

`sqlx::Transaction: DerefMut<Target = PgConnection>` — that's why `&mut *tx` works.

## `#[sqlx::test]` — auto-rollback tests

```rust
use sqlx::PgPool;

#[sqlx::test]
async fn test_save_user(pool: PgPool) {
    // Migrations run against fresh DB. Everything rolled back at end.
    let mut conn = pool.acquire().await.unwrap();
}

// With fixtures (SQL files in `fixtures/`):
#[sqlx::test(fixtures("users"))]
async fn test_with_seed_data(pool: PgPool) {
    // fixtures/users.sql is loaded before the test.
}
```

Setup env:
```bash
# .env
DATABASE_URL=postgres://user:pass@localhost/test_db
```

## Compile-time offline data

Before committing any new query:

```bash
cargo sqlx prepare --workspace
git add .sqlx/
```

Allows CI to build without a live DB.

## Custom types via newtypes

```rust
sqlx::query_as!(
    User,
    r#"SELECT id as "id: UserId", email as "email: Email", created_at FROM users"#,
)
```

Requires `impl sqlx::Type<Postgres> for UserId` + `Decode`/`Encode` impls.

## Never do

- ❌ String-concat SQL (injection risk) — always `$1`, `$2`, …
- ❌ `unwrap()` on `.fetch_one()` — use `.fetch_optional()` + handle None
- ❌ Run migrations manually in test code — use `#[sqlx::test]`
- ❌ Commit without `cargo sqlx prepare` if you added queries

## Always do

- ✅ `query!` or `query_as!` macros (compile-time check)
- ✅ FK-aware ordering in migrations
- ✅ `fetch_optional` for "find_by_id" queries
- ✅ Commit `.sqlx/` directory for offline builds
