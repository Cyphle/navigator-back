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

#### 3. Layered error handling with automatic propagation

Each layer can produce its own errors. All errors implement the `ApplicationError` trait defined in `domains/common/errors/`:

```rust
pub trait ApplicationError: Debug {
    fn get_message(&self) -> String;
    fn status_code(&self) -> u16 { 500 }  // default: 500
}
```

**Errors by layer:**
- **Repository**: `RepositoryError` (DB errors, status 500)
- **Domain**: Domain-specific business errors (e.g. `MagicListAccessDeniedError` → 403, `MagicListError` → 404)
- **Middleware**: `MiddlewareError` for HTTP errors (401 missing session, 400 invalid format)

**Propagation:** Usecases and repositories return `Result<T, Box<dyn ApplicationError>>`. The middleware automatically converts via `From<Box<dyn ApplicationError>>` into `MiddlewareError`, which implements Actix's `ResponseError` to produce the HTTP response with the correct status code and JSON error message.

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

### When creating a new domain
1. Create the folder `src/domains/<name>/` with subfolders `http/`, `usecases/`, `domain/`, `repositories/`
2. Define entities and value objects in `domain/`
3. Define the repository trait in `domain/`
4. Define domain-specific errors in `domain/errors.rs`, implementing `ApplicationError`
5. Implement the SQLx repository in `repositories/`
6. Create usecases in `usecases/`
7. Create middlewares in `http/` (session extraction, validation, error mapping)
8. Create controllers in `http/` (Actix routes, inject the usecase into the middleware)
9. Create mocks in `src/testing/repositories/`
10. Add the repository to `ActixState` and wire it in `main.rs`

### When adding an endpoint to an existing domain
1. Create the command/query in `domain/` if needed (the usecase will build it internally from raw fields)
2. Create the usecase in `usecases/` — receives raw input fields, parses them, builds the domain command, calls the repository
3. Create the middleware in `http/` — unpacks the request DTO into raw fields and forwards them to the usecase
4. Add the route in the controller
5. Add the method to the repository trait and its SQLx implementation
6. Update the mock repository
7. Test each layer
