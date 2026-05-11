---
name: rust-error-handling
description: Use when defining error types, propagating errors across hexagonal layers, or mapping them to HTTP. All error types live in `domain/`; each layer wraps the lower layer's error with `#[source]` to preserve the full cause chain and adds contextual fields (entity id, operation, attempted value) at the wrap point. Triggers on keywords like error, thiserror, anyhow, ResponseError, AppError, error mapping, error wrapping, source chain, cause, propagation, ? operator.
allowed-tools: Read, Grep, Glob, Edit, Write
---

# Rust Error Handling

## The rule

- **All error types are defined in `crates/domain/`** — domain errors, repository-port errors, and per-use-case errors. The domain owns the error vocabulary; other crates consume it.
- **Each layer wraps, never replaces.** When an error crosses a layer boundary, the upper layer wraps it via `#[source]` and adds the context that *only it* knows (entity id, operation name, attempted value). The cause chain reachable through `Error::source()` stays unbroken from HTTP all the way down to `sqlx::Error`.
- **`api/` defines `AppError`** as the one HTTP-shaped exception — it wraps domain errors and implements `ResponseError`.
- **`anyhow`** is allowed only inside `main()` bootstrap. Every other function uses `thiserror`.

## Error layering

```
sqlx::Error                                            (in infrastructure/, external)
   │   infra catches it, boxes the cause
   ▼
RepositoryError::Technical { #[source] cause }         (defined in domain/)
   │   use-case wraps, adds candidate/entity id
   ▼
CreateUserError::Repository { user_id, #[source] src } (defined in domain/)
   │   api/ wraps via #[from], maps to HTTP status
   ▼
AppError::CreateUser(#[from] CreateUserError)          (defined in api/)
   │   ResponseError walks Error::source() and logs the full chain
   ▼
HTTP 4xx/5xx { error, code }
```

The chain works the same way for "not found", "conflict", validation errors — those are *variants* on each layer, not extra layers.

## Domain — repository-port error

`domain/` cannot import `sqlx`/`reqwest`/`redis`. To carry foreign causes without leaking infra types, use `Box<dyn Error + Send + Sync>`:

```rust
// crates/domain/src/errors/repository.rs
use std::error::Error;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("entity not found")]
    NotFound,

    #[error("conflict on {0}")]
    Conflict(String),

    #[error("technical error in repository")]
    Technical(#[source] Box<dyn Error + Send + Sync + 'static>),
}
```

> The `Display` string never includes the source's message — `Error::source()` exposes the chain to the logger. Double-printing produces noisy, redundant log lines.

## Domain — per-use-case error

Each use-case has its own error enum, **also in `domain/`**. Variants carry lower-level errors via `#[source]` and the context the use-case knows about.

```rust
// crates/domain/src/errors/create_user.rs
use uuid::Uuid;
use crate::errors::RepositoryError;

#[derive(Debug, thiserror::Error)]
pub enum CreateUserError {
    #[error("invalid email format")]
    InvalidEmail,

    #[error("email already in use: {email}")]
    EmailAlreadyExists { email: String },

    #[error("repository failure while creating user (candidate id={candidate_id})")]
    Repository {
        candidate_id: Uuid,
        #[source]
        source: RepositoryError,
    },
}
```

Naming convention: one error enum per use-case, named `<UseCase>Error`. Variants reflect domain outcomes (what happened from the caller's POV), not technical taxonomy.

## Infrastructure — wrap `sqlx::Error` at the boundary

```rust
// crates/infrastructure/src/user_repository_sqlx.rs
use async_trait::async_trait;
use domain::{ports::UserRepository, errors::RepositoryError, User};

#[async_trait]
impl UserRepository for UserRepositorySqlx {
    async fn save(&self, user: &User) -> Result<(), RepositoryError> {
        sqlx::query!(
            "INSERT INTO users (id, email) VALUES ($1, $2)",
            user.id.0, user.email.as_str()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db) if db.is_unique_violation() =>
                RepositoryError::Conflict("email".into()),
            sqlx::Error::RowNotFound =>
                RepositoryError::NotFound,
            _ =>
                RepositoryError::Technical(Box::new(e)),
        })?;
        Ok(())
    }
}
```

The infra layer is the **only** place where `sqlx::Error` exists. From this point upward the cause is anonymised behind `Box<dyn Error>` but still reachable through `source()`.

## Application — wrap `RepositoryError` with context

```rust
// crates/application/src/create_user.rs
use std::sync::Arc;
use domain::{
    User, Email,
    ports::UserRepository,
    errors::{CreateUserError, RepositoryError},
};

pub struct CreateUser { repo: Arc<dyn UserRepository> }

impl CreateUser {
    pub async fn execute(&self, email: String) -> Result<User, CreateUserError> {
        let email = Email::parse(email).map_err(|_| CreateUserError::InvalidEmail)?;
        let user = User::new(email.clone());

        self.repo.save(&user).await.map_err(|e| match e {
            RepositoryError::Conflict(_) =>
                CreateUserError::EmailAlreadyExists { email: email.as_str().into() },
            source =>
                CreateUserError::Repository { candidate_id: user.id.0, source },
        })?;

        Ok(user)
    }
}
```

Two things happen at this wrap point:

1. **Semantic re-mapping** — `Conflict` becomes `EmailAlreadyExists` (the domain meaning), with the actual email value attached.
2. **Generic wrap** — anything else is folded into `Repository { candidate_id, source }`, attaching the id of the entity the use-case was operating on. The original `RepositoryError` is preserved through `#[source]`.

## API — `AppError` wraps domain errors, maps to HTTP

```rust
// crates/api/src/error.rs
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use domain::errors::CreateUserError;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("create user failed")]
    CreateUser(#[from] CreateUserError),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,
}

#[derive(Serialize)]
struct ErrorBody { error: String, code: &'static str }

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::CreateUser(CreateUserError::InvalidEmail) => StatusCode::BAD_REQUEST,
            AppError::CreateUser(CreateUserError::EmailAlreadyExists { .. }) => StatusCode::CONFLICT,
            AppError::CreateUser(CreateUserError::Repository { .. }) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        log_error_chain(self);

        let (code, message) = match self {
            AppError::CreateUser(CreateUserError::InvalidEmail) =>
                ("INVALID_EMAIL", "invalid email format".into()),
            AppError::CreateUser(CreateUserError::EmailAlreadyExists { email }) =>
                ("EMAIL_EXISTS", format!("email already in use: {email}")),
            AppError::CreateUser(CreateUserError::Repository { .. }) =>
                ("INTERNAL", "internal server error".into()),
            AppError::BadRequest(msg) => ("BAD_REQUEST", msg.clone()),
            AppError::Unauthorized => ("UNAUTHORIZED", "unauthorized".into()),
        };

        HttpResponse::build(self.status_code())
            .json(ErrorBody { error: message, code })
    }
}
```

The `#[from] CreateUserError` lets the handler use `?` directly. The use-case's context (candidate id, attempted email) flows up *unchanged* — `AppError` doesn't try to re-encode it.

## Logging the full chain

`tracing` does not walk `Error::source()` by default. Walk it explicitly at the API boundary so the log line carries every wrapped layer:

```rust
// crates/api/src/error.rs
use std::error::Error;

fn log_error_chain(err: &dyn Error) {
    let mut chain = vec![format!("{err}")];
    let mut src = err.source();
    while let Some(e) = src {
        chain.push(format!("{e}"));
        src = e.source();
    }
    tracing::error!(error.chain = ?chain, "request failed");
}
```

A representative chain in the logs:

```
error.chain = [
  "create user failed",
  "repository failure while creating user (candidate id=4f6e...)",
  "technical error in repository",
  "error returned from database: duplicate key value violates unique constraint \"users_email_idx\"",
]
```

Each line corresponds to a layer crossing — full path from HTTP boundary down to `sqlx::Error`. Sensitive payloads are *not* in `Display`; client responses use `error_response()` and never the chain.

## Handler signature

```rust
pub async fn create_user(
    body: web::Json<CreateUserDto>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let use_case = CreateUser::new(state.users.clone());
    let user = use_case.execute(body.email.clone()).await?;   // CreateUserError → AppError via #[from]
    Ok(HttpResponse::Created().json(user))
}
```

Handlers stay one-liners. Wrapping happens at the use-case (semantic + entity context) and at `AppError` (HTTP shape). The `?` operator plus `#[from]` propagates the chain intact.

## Anti-patterns

### ❌ Stringifying the source

```rust
// BAD — flattens the chain into one Display line, source() returns None
#[derive(Debug, thiserror::Error)]
pub enum CreateUserError {
    #[error("repository failure: {0}")]
    Repository(String),
}

self.repo.save(&user).await.map_err(|e| CreateUserError::Repository(format!("{e}")))?;
```

```rust
// GOOD — keeps the chain reachable
#[derive(Debug, thiserror::Error)]
pub enum CreateUserError {
    #[error("repository failure (id={candidate_id})")]
    Repository {
        candidate_id: Uuid,
        #[source]
        source: RepositoryError,
    },
}
```

### ❌ Replacing instead of wrapping

```rust
// BAD — original sqlx::Error is lost, you'll never know which constraint fired
.map_err(|_| RepositoryError::Technical(Box::new(MyAdHocError("save failed"))))
```

### ❌ Defining errors outside `domain/`

```rust
// BAD — application/ now owns part of the error vocabulary; domain ports can't reference it
// crates/application/src/errors.rs
pub enum CreateUserError { ... }
```

The use-case error must live in `domain/` so the use-case in `application/` and the handler in `api/` both depend on the same definition without crossing layers backwards.

## Never do

- ❌ Define error types outside `crates/domain/` (sole exception: `AppError` in `api/`)
- ❌ Drop the cause: `MyError::Failed("...")` with no `#[source]` field
- ❌ Embed the source's message in the parent's `#[error("…{source}")]` — flattens the chain in `Display`
- ❌ Re-implement `Display` to walk `source()` — `Display` is the *local* message; chain-walking is the logger's job
- ❌ Use `anyhow::Error` in any function signature outside `main()`
- ❌ `unwrap()` / `expect()` outside tests and `main()` bootstrap
- ❌ Return raw `sqlx::Error` from a repository — wrap into `RepositoryError` at the infra boundary
- ❌ Leak internal details (SQL fragments, file paths, stack traces) into the JSON body — log them, don't serialize them

## Always do

- ✅ Define every error enum in `crates/domain/` (one per use-case)
- ✅ Wrap with `#[source]` (or `#[from]` when there's no extra context to attach); **never** replace
- ✅ At each wrap point, attach the context only that layer knows: entity id, operation name, attempted value
- ✅ Use `Box<dyn Error + Send + Sync + 'static>` to carry foreign causes through `domain/` without leaking infra types
- ✅ Walk `Error::source()` once at the API boundary; log the full chain there
- ✅ One use-case error per use-case; variants reflect domain outcomes, not technical taxonomy

## Refactoring checklist (existing code)

1. [ ] Move every error enum into `crates/domain/src/errors/`
2. [ ] Replace `String`-typed source variants with structured `{ context_field, #[source] source }` shape
3. [ ] Strip `{source}` / `{0}` from any parent `#[error]` strings that point at a wrapped error
4. [ ] At each `.map_err(...)`, ensure the upper-layer variant carries the lower error via `#[source]` plus the context known at that layer
5. [ ] Add `log_error_chain` (or equivalent) in `ResponseError::error_response`
6. [ ] Verify a sample failing request logs at least three `error.chain` entries (api → use-case → repo); add a missing wrap if the chain is shorter than expected
7. [ ] `cargo clippy --all-targets --all-features -- -D warnings` clean; no new `unwrap()` / `unwrap_or_default()` masking errors
