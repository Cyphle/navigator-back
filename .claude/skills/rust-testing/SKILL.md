---
name: rust-testing
description: Use when writing tests in this Rust backend. Covers unit tests in domain, mocked tests in application with mockall, integration tests with sqlx::test, and end-to-end with actix_web::test. Triggers on keywords like test, mock, integration test, unit test, fixture, mockall, sqlx test.
allowed-tools: Read, Grep, Glob, Edit, Write, Bash
---

# Rust Testing Strategy

## Test pyramid (per crate)

### `domain/` — unit tests (fastest, no I/O)

```rust
// crates/domain/src/user.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_email_parses() {
        let email = Email::parse("foo@bar.com".into()).unwrap();
        assert_eq!(email.as_str(), "foo@bar.com");
    }

    #[test]
    fn invalid_email_fails() {
        assert!(matches!(
            Email::parse("not-an-email".into()),
            Err(DomainError::InvalidEmail)
        ));
    }
}
```

### `application/` — use-case tests with mocks

Enable mock on the trait in `domain/`:

```rust
// crates/domain/src/ports/user_repository.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DynUserRepository: Send + Sync { /* ... */ }
```

Cargo.toml:
```toml
# crates/application/Cargo.toml
[dev-dependencies]
mockall = { workspace = true }
tokio = { workspace = true, features = ["macros", "rt"] }
domain = { path = "../domain", features = ["testing"] }
```

Test:
```rust
// crates/application/tests/create_user_test.rs
use application::CreateUser;
use domain::ports::MockDynUserRepository;
use std::sync::Arc;

#[tokio::test]
async fn create_user_saves_and_returns() {
    let mut mock = MockDynUserRepository::new();
    mock.expect_save().times(1).returning(|_, _| Ok(()));

    let use_case = CreateUser::new(Arc::new(mock));
    let user = use_case.execute(&mut /* fake conn */, "foo@bar.com".into()).await.unwrap();
    assert_eq!(user.email.as_str(), "foo@bar.com");
}
```

### `infrastructure/` — `#[sqlx::test]` integration

```rust
// crates/infrastructure/tests/user_repository_test.rs
use sqlx::PgPool;
use domain::{User, Email, ports::UserRepository};
use infrastructure::UserRepositorySqlx;

#[sqlx::test]
async fn save_and_retrieve(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let repo = UserRepositorySqlx;

    let user = User::new(Email::parse("test@example.com".into()).unwrap());
    repo.save(&mut conn, &user).await.unwrap();

    let found = repo.find_by_id(&mut conn, user.id).await.unwrap();
    assert_eq!(found.unwrap().id, user.id);
}

#[sqlx::test(fixtures("users"))]
async fn finds_seeded_user(pool: PgPool) {
    // fixtures/users.sql loaded pre-test
    let mut conn = pool.acquire().await.unwrap();
    let repo = UserRepositorySqlx;
    let users = repo.find_all(&mut conn).await.unwrap();
    assert!(!users.is_empty());
}
```

### `api/` — end-to-end

```rust
// crates/api/tests/users_e2e_test.rs
use actix_web::{test, web, App};
use api::{handlers, state::AppState};
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test]
async fn post_users_creates_user(pool: PgPool) {
    let state = AppState {
        pool: pool.clone(),
        users: Arc::new(infrastructure::UserRepositorySqlx),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .route("/users", web::post().to(handlers::users::create_user))
    ).await;

    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(serde_json::json!({ "email": "new@example.com" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
}
```

## Running tests

```bash
cargo test --workspace
cargo test -p domain
cargo test -p infrastructure save_and_retrieve
cargo test -- --nocapture        # show println / tracing output
cargo test -- --ignored          # slow/opt-in tests
```

## Never do

- ❌ Hit a real external API in unit tests — mock it
- ❌ Share mutable state between tests (order-dependent)
- ❌ `unwrap()` on `.await` without understanding failure modes
- ❌ `#[tokio::main]` in tests — use `#[tokio::test]`

## Always do

- ✅ `#[sqlx::test]` for anything touching the DB (auto-rollback)
- ✅ `mockall` for trait-based dependencies
- ✅ One assertion target per test (test one thing)
- ✅ Descriptive names: `create_user_returns_conflict_when_email_exists()`
