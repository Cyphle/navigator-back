# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

The project is a Rust/Actix-Web backend with a layered architecture:

```
src/
├── main.rs              # App bootstrap: wires config, DB, OIDC, session, repositories
├── config/              # Config loading and shared Actix state
│   ├── application.rs   # AppConfig (loads config/default.yaml → config/local.yaml → env NAVIGATOR_*)
│   └── actix.rs         # ActixState<DB, U, F> — shared app state with generic type params for testability
├── domain/              # Pure domain models (User, Family, CreateFamilyCommand, FamilyRole)
├── repositories/        # Trait definitions + Sqlx implementations (UserRepository, FamilyRepository)
├── application/         # Business logic functions (get_families, create_family, get_users_me)
├── http/
│   ├── controllers/     # Actix route handlers — thin, delegate to middleware layer
│   ├── middlewares/     # Session extraction + application call + HTTP response mapping
│   └── requests/        # JSON request structs (e.g. CreateFamilyRequest)
├── security/
│   ├── controllers/     # login, logout, register routes (OIDC flow + Keycloak admin API)
│   ├── oidc.rs          # OIDC client setup
│   └── token.rs         # Session token helpers (get_username_from_session, get_connected_username)
└── testing/             # In-memory mocks for unit/integration tests
    ├── actix/           # mock_actix_state() helper — builds ActixState with mock repos
    └── repositories/    # MockUserRepository, MockFamilyRepository, MockPoolPostgres
```

### Key design patterns

**Generic ActixState**: `ActixState<DB, U, F>` is parameterized over database connection, user repository, and family repository types. This allows injecting mock implementations in tests without runtime overhead.

**Controller → Middleware → Application layering**: Controllers (`http/controllers/`) are thin Actix route handlers that call middleware functions (`http/middlewares/`). Middlewares handle session extraction and HTTP response mapping. Application functions (`application/`) contain the actual business logic and work with domain types.

**Repository traits with transaction injection**: Repository methods take `&mut Tx` (a transaction handle) as a parameter. The application layer controls transaction lifecycle (begin/commit/rollback). Traits use `async_trait` and the `Tx` type is generic, keeping the real and mock impls interchangeable.

**Testing approach**: Use `mock_actix_state()` with `MockStateConfig` to build test state. Middleware tests use `actix_web::test::init_service` + `spy!` macro (from the `spy` crate) to verify the correct application function is called. Application-layer unit tests use mock repos directly.

### Configuration

Config is loaded in priority order: `config/default.yaml` → `config/local.yaml` (git-ignored, for local overrides) → environment variables with prefix `NAVIGATOR_` and `_` separator (e.g. `NAVIGATOR_DATABASE_HOST`).

### Auth flow

Authentication uses OIDC (Keycloak). Login redirects to Keycloak, which returns a code; the backend exchanges it for tokens and stores the user session in Redis. The session cookie is named `actix_cookie` by default. The `security/token.rs` helpers extract the username from the active session for use in protected endpoints.

## Development

- Always unit test all the code you create or update
- Always make sure that all tests pass
- Use mock data factories for tests and avoid too many code lines with boilerplate in tests. Tests should be short and easily readable.