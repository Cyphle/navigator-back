# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Navigator is a family management application ("Navigate through your life swiftly"). The backend is written in Rust with Actix-Web. It exposes a REST API to manage:

- Families and their members
- Magic Lists (shared task/shopping lists)
- Calendar/agenda
- Recipes and meal planning
- Bank accounts
- User authentication (OIDC via Keycloak)

## Commands

### Build & Run
```bash
cargo build
cargo run
```

### Tests
```bash
cargo test                          # run all tests
cargo test <test_name>              # run a single test by name
cargo test -p navigator-back        # run all tests in the package
```

### Infrastructure (local dev)
```bash
# From infra/local/docker-compose/
docker compose up -d                # start PostgreSQL (port 5434), Keycloak (port 8181), Redis (port 6379)
docker compose down
```

### Database migrations
Migrations run automatically on startup via `sqlx::migrate!("./migrations")`. To add a new migration, create a numbered SQL file in `migrations/`.

## Architecture

### Overview

```
src/
├── main.rs              # Bootstrap: config, DB, OIDC, session, repositories, routes
├── config/              # Config loading and shared Actix state
├── domains/             # Independent business domains (magic_list, family, bank_account, user, ...)
│   ├── magic_list/      # Example of a complete domain
│   ├── family/
│   ├── bank_account/
│   ├── user/
│   └── common/          # Shared types and errors across domains
├── security/            # OIDC, session management, login/logout/register routes
└── testing/             # Test infrastructure (mocks, helpers)
```

### Architecture rules

#### 1. Each domain is an independent functional module

Each folder in `src/domains/` represents a self-contained functional scope. A domain contains all its layers: HTTP, usecases, domain, repositories. Domains do not depend on each other (except through `common/`).

Typical domain structure (example: `magic_list`):
```
domains/magic_list/
├── http/                              # HTTP layer (entry point)
│   ├── magic_list_controller.rs       # Actix endpoints (routes)
│   ├── magic_list_middleware.rs        # Request/session extraction + error mapping
│   ├── magic_list_item_middleware.rs
│   ├── magic_list_requests.rs         # Request DTOs (JSON deserialization)
│   └── magic_list_views.rs            # Response DTOs (JSON serialization)
├── usecases/                          # Business logic orchestration
│   ├── create_magic_list_use_case.rs
│   ├── get_magic_list_summary_use_case.rs
│   ├── add_item_to_magic_list_use_case.rs
│   └── check_magic_list_access.rs     # Cross-cutting access verification
├── domain/                            # Pure business objects
│   ├── magic_list.rs                  # Entity
│   ├── magic_list_item.rs             # Entity
│   ├── magic_list_type.rs             # Value object (enum)
│   ├── magic_list_repository.rs       # Repository trait (interface)
│   ├── create_magic_list_command.rs   # Command (business intent)
│   ├── magic_list_summary.rs          # Read projection
│   └── errors.rs                      # Domain-specific errors
└── repositories/                      # Infrastructure implementation
    └── sqlx_magic_list_repository.rs  # SQLx implementation of the trait
```

#### 2. Request flow: Controller → Middleware → UseCase → Domain → Repository

Each request traverses the layers in this strict order:

- **Controller**: Actix route handler function. Only exposes the endpoint, extracts path/body parameters, and delegates to the middleware. The controller injects the usecase function as a parameter to the middleware.
- **Middleware**: Extracts session information (username), unpacks the request DTO into raw input fields (`String`, `Option<i32>`, etc.), performs HTTP-level validations that map to a 400 BadRequest (e.g. date format parsing), then forwards everything to the usecase and maps the result to an HTTP response. The middleware does NOT construct domain commands.
- **UseCase**: Receives raw input fields, parses them into domain types (enums, value objects), builds the domain command internally, then orchestrates the business logic. Checks access rights, coordinates repository calls, applies business rules. Has no knowledge of HTTP.
- **Domain**: Contains entities, value objects, commands, and repository traits. No dependency on infrastructure.
- **Repository**: Implements the traits defined in the domain. Handles database access via SQLx.

**Field passing convention between middleware and usecase:**
- Owned types (`String`, `Vec<...>`) at the function boundary — the async closure injection pattern makes passing references (`&str`, `&[T]`) impractical here (HRTB lifetimes through `Future`).
- References (`&str`, `&[T]`) for internal helpers inside the usecase (parsing functions, repository calls).
- `Copy` types (`i32`, `bool`, `Option<NaiveDate>`, simple enums) passed by value.
- For nested input shapes (e.g. family members), define a usecase-level `XxxInput` struct holding raw strings — the usecase is responsible for parsing them into domain enums.

#### 3. Layered error handling — wrap, never replace

Errors are typed enums built with `thiserror`. There is no `ApplicationError` trait — every error implements `std::error::Error` through `#[derive(thiserror::Error)]`. Each layer wraps the lower one via `#[source]`, attaching the context that only it knows. The cause chain reachable through `Error::source()` stays unbroken from HTTP all the way down to `sqlx::Error`.

**Error layering:**

```
sqlx::Error                                                    (only in repositories/)
   │   RepositoryError::from(sqlx::Error)
   │   matches: unique-violation → Conflict, RowNotFound → NotFound, _ → Technical
   ▼
RepositoryError { NotFound | Conflict(String) | Technical(#[source] Box<dyn Error>) }
   │   .map_err(|source| <UseCase>Error::Repository { entity_id, source })
   ▼
<UseCase>Error  (one per use case, in domain/errors.rs)
   │   #[from] inside MiddlewareError
   ▼
MiddlewareError  →  status_code() dispatches 401/400/403/404/409/500
   │   ResponseError::error_response() walks Error::source() once and logs the chain
   ▼
HTTP response (4xx → typed message, 5xx → "internal server error", chain logged)
```

**Where errors live:**
- **`domains/common/errors/repository_error.rs`**: `RepositoryError` (the only error type that knows about `sqlx::Error`, hidden behind `Box<dyn Error>` in the `Technical` variant)
- **`domains/<X>/domain/errors.rs`** (or `family_errors.rs`): one error enum per use case (e.g. `CreateMagicListError`, `CheckMagicListAccessError`, `AddItemToMagicListError`)
- **`domains/common/errors/middleware_error.rs`**: `MiddlewareError` — the HTTP-shaped exception. Wraps each use-case error via `#[from]`, implements Actix's `ResponseError`. **The only error type defined outside `domain/`.**

**Use-case error variant pattern:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum CreateMagicListError {
    #[error("repository failure while creating magic list (name={name})")]
    Repository {
        name: String,
        #[source]
        source: RepositoryError,
    },
}
```

Each `Repository`-style variant carries (a) the entity context known at that layer (id, name) and (b) the source error via `#[source]` — never via `{source}` in `#[error]` (that would flatten the chain in `Display`).

**Propagation in code:**

```rust
// Repository impl: ? converts sqlx::Error → RepositoryError automatically
sqlx::query(...).execute(&self.pool).await?;

// Use case: wrap RepositoryError with entity context
state.repo.do_thing(id).await
    .map_err(|source| CreateMagicListError::Repository { name, source })?;

// Middleware: ? converts <UseCase>Error → MiddlewareError via #[from]
create_magic_list(state, username, ...).await?;
```

**Status code mapping (in `MiddlewareError::status_code`):**
- `MissingUsername` → 401
- `InvalidDateFormat` → 400
- `AccessDenied` (via `CheckMagicListAccessError`) → 403
- `NotFound` (via `CheckMagicListAccessError`) → 404
- `AlreadyExists` (via `CreateFamilyError`) → 409
- Anything else (Repository variants) → 500

**Logging:** `MiddlewareError::error_response` walks `Error::source()` once to log the full chain (e.g. `["middleware err", "repo failure (id=42)", "technical error", "duplicate key violates …"]`). The HTTP body for 5xx is neutralised to `"internal server error"`; chain details are logged only.

**Anti-patterns banned:**
- ❌ `RepositoryError { error: String }` (stringification — flattens the chain)
- ❌ `#[error("...{source}")]` (the parent `Display` must not include the source's message)
- ❌ Returning `sqlx::Error` from a repository trait
- ❌ Returning `Box<dyn ApplicationError>` / `Box<dyn Error>` from a use case (use the typed per-use-case error)
- ❌ Defining error enums outside `domain/` (sole exception: `MiddlewareError` in `common/errors/`)

#### 4. Dependency injection via generic ActixState

`ActixState<DB>` is parameterized on the DB connection type. It holds all repositories. In production, DB = `PgPool`. In tests, DB = `MockPoolPostgres`. The `DbConnection` trait abstracts transactional access.

#### 5. Usecase injection via function references

Middlewares receive usecase functions as parameters. This allows replacing them with spies in middleware tests, fully decoupling the middleware from the concrete usecase.

```rust
// In the controller:
add_item_to_magic_list_middleware(session, state, id, payload, add_item_to_magic_list_use_case).await

// In middleware tests:
let (spy_handler, spy) = spy!();
add_item_to_magic_list_middleware(session, state, id, payload, spy_handler).await
```

#### 6. Repository traits are defined in the domain layer

The `MagicListRepository` trait lives in `domain/magic_list_repository.rs`. The SQLx implementation lives in `repositories/`. The domain never depends on infrastructure.

### Configuration

Config is loaded in priority order: `config/default.yaml` → `config/local.yaml` (git-ignored) → environment variables prefixed with `NAVIGATOR_` (separator `_`, e.g. `NAVIGATOR_DATABASE_HOST`).

### Auth flow

Authentication uses OIDC via Keycloak. The backend exchanges the authorization code for tokens and stores the session in Redis. The session cookie is `actix_cookie`. The `security/token.rs` helpers extract the username from the active session.

## Development

### General rules
- Always unit test all code created or modified
- Always make sure all tests pass
- Use test data factories and avoid boilerplate in tests. Tests should be short and readable.
- **Keep `CLAUDE.md` in sync with architecture changes.** Whenever the structure, layering, error model, naming convention, or any other architectural rule is modified, update this file in the same commit/PR. A stale CLAUDE.md misleads every future contribution. If you're not sure whether a change qualifies, err on the side of updating: examples include renaming a layer, changing a return type convention, introducing/removing a trait, moving where errors live, changing the field-passing convention between layers, adding/removing a step in the request flow.

### When creating a new domain
1. Create the folder `src/domains/<name>/` with subfolders `http/`, `usecases/`, `domain/`, `repositories/`
2. Define entities and value objects in `domain/`
3. Define the repository trait in `domain/`, returning `Result<T, RepositoryError>`
4. Define one error enum per use case in `domain/errors.rs` using `#[derive(thiserror::Error)]`. Each `Repository`-style variant carries entity context (`id`, `name`, …) and the source via `#[source] source: RepositoryError`
5. Implement the SQLx repository in `repositories/` — it relies on the `From<sqlx::Error> for RepositoryError` impl, so internal helpers can use `?` directly
6. Create usecases in `usecases/`, returning the per-use-case typed error; `.map_err(|source| <UseCase>Error::Repository { …, source })` at each repository call
7. Create middlewares in `http/` — extracts session, unpacks request DTO into raw fields, forwards to usecase. The usecase error converts to `MiddlewareError` via `#[from]`
8. Create controllers in `http/` (Actix routes, inject the usecase into the middleware)
9. Create mocks in `src/testing/repositories/`, returning `Result<T, RepositoryError>`
10. Add the repository to `ActixState` and wire it in `main.rs`
11. Add `#[from]` variants in `MiddlewareError` for the new use-case errors, and map their semantics to HTTP statuses in `status_code()`

### When adding an endpoint to an existing domain
1. Create the command/query in `domain/` if needed (the usecase will build it internally from raw fields)
2. Define the use-case-specific error enum in `domain/errors.rs` (one per use case)
3. Create the usecase in `usecases/` — receives raw input fields, parses them, builds the domain command, calls the repository, wraps repository errors with entity context
4. Create the middleware in `http/` — unpacks the request DTO into raw fields and forwards them to the usecase
5. Add the route in the controller
6. Add the method to the repository trait and its SQLx implementation (returning `Result<T, RepositoryError>`)
7. Update the mock repository
8. Add a `#[from]` variant for the new use-case error in `MiddlewareError`, and extend `status_code()` if it has non-500 semantics
9. Test each layer
