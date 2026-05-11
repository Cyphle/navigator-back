---
name: models-and-data-contracts
description: Use to extract a domain data model from a business spec/ticket, derive API contracts (REST or BFF), and design the matching SQL schema. Bridges plain-text requirements to concrete artifacts at three hexagonal layers — `domain/` types, `api/` DTOs, and `migrations/` SQL — keeping them consistent. Triggers on keywords like data model, domain model, entity, value object, aggregate, invariant, API contract, OpenAPI, DTO, request, response, BFF, back for front, schema, table, column, ERD, modelisation, ticket, spec, business requirement.
allowed-tools: Read, Grep, Glob, Edit, Write
---

# Models & Data Contracts

Three-layer extraction from a business spec to working code: **domain model → API contract → SQL schema**. Each layer has its own concerns; they must stay consistent.

## When to use

- A new ticket / spec / user story landed and you need to translate it into types, endpoints, and tables
- You're starting a feature and need to align domain entities, DTOs, and migrations *before* writing code
- An external client (frontend, mobile, partner) asks for an API contract — REST or BFF-shaped
- You're auditing existing code for drift between domain types, DTOs, and DB columns

## Process (in order)

1. **Read the source carefully.** Spec, ticket, conversation, mockup. Note every noun (→ candidate entity), every verb (→ candidate use-case), every adjective/state (→ value object or enum), every constraint (→ invariant).
2. **Domain first** — extract entities, value objects, aggregates, invariants. No tech concerns yet.
3. **API contract second** — design DTOs and endpoints. Decide: REST resource-shaped, or BFF screen-shaped?
4. **SQL schema last** — translate the domain into tables, FKs, indexes. Coordinate with `sqlx-migration` skill.
5. **Cross-check** — every domain field has a DB column AND a DTO field (or an explicit reason it doesn't). Every invariant has a DB constraint AND a domain check.

## Step 1 — Domain model

### What to extract

- **Entity** — has identity that persists over time (`User`, `Order`). Mutable.
- **Value Object** — defined by its value, no identity (`Email`, `Money`, `PostalCode`). Immutable, validated at construction.
- **Aggregate root** — the entity through which the cluster is mutated. Enforces invariants across its members.
- **Domain enum** — finite set of states (`OrderStatus::{Pending, Paid, Shipped, Cancelled}`).
- **Invariant** — a rule that must always hold (`order.total >= 0`, `email contains @`).
- **Domain event** (if applicable) — a fact that occurred (`OrderShipped`).

### Heuristics for ticket reading

| Phrase in ticket                         | Likely artifact                          |
| ---------------------------------------- | ---------------------------------------- |
| "A user can have many orders"            | Entity `User`, FK from `Order`           |
| "Email must be unique"                   | Value Object `Email` + DB unique index   |
| "Status can be one of …"                 | `enum` + CHECK constraint                |
| "Cannot be negative" / "must be positive"| Value Object with parsing validation     |
| "Once shipped, cannot be cancelled"      | State-machine invariant in domain method |
| "Soft-delete"                            | `deleted_at: Option<DateTime<Utc>>`      |

### Domain skeleton (Rust)

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus { Pending, Paid, Shipped, Cancelled }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money(i64); // cents — never f64 for money

impl Money {
    pub fn from_cents(n: i64) -> Result<Self, DomainError> {
        if n < 0 { return Err(DomainError::InvalidAmount); }
        Ok(Self(n))
    }
    pub fn cents(&self) -> i64 { self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub user_id: crate::UserId,
    pub status: OrderStatus,
    pub total: Money,
    pub created_at: DateTime<Utc>,
    pub shipped_at: Option<DateTime<Utc>>,
}

impl Order {
    /// Invariant: only Paid orders can ship.
    pub fn ship(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        match self.status {
            OrderStatus::Paid => {
                self.status = OrderStatus::Shipped;
                self.shipped_at = Some(now);
                Ok(())
            }
            _ => Err(DomainError::InvalidStateTransition),
        }
    }
}
```

### Domain checklist

- [ ] Every identifier is a newtype (`OrderId(Uuid)`, never raw `Uuid` in signatures)
- [ ] Every constrained string is a value object (`Email`, `Slug`) with `parse() -> Result<Self, DomainError>`
- [ ] Money never `f64` — `i64` cents or `rust_decimal::Decimal`
- [ ] Enums for finite states; never `String` with magic values
- [ ] State transitions live as methods on the entity, not free functions
- [ ] No `sqlx`, `actix_web`, or `serde_json::Value` leaks here

## Step 2 — API contract

### Decide the shape

- **REST / API-driven** — resource-shaped, stable, reusable across many clients. DTOs mirror the domain closely. One endpoint per CRUD action. Versioned (`/v1/orders`).
- **BFF (Back-for-Front)** — screen-shaped, tailored to one frontend's view. DTOs aggregate / denormalize across multiple domain entities. One endpoint per screen or interaction. May change with the UI.

A single backend can do both: keep `/v1/...` clean REST, plus `/bff/<app>/<screen>` for tailored views.

### DTOs are not domain types

DTOs live in outside of domain. They are wire-format only — Serde shapes that deserialize requests and serialize responses. Never reuse a domain struct as a DTO directly: it couples the wire format to internal evolution.

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: Uuid,
    pub total_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub total_cents: i64,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipped_at: Option<DateTime<Utc>>,
}

impl From<domain::Order> for OrderResponse {
    fn from(o: domain::Order) -> Self {
        Self {
            id: o.id.0,
            user_id: o.user_id.0,
            status: format!("{:?}", o.status).to_lowercase(),
            total_cents: o.total.cents(),
            created_at: o.created_at,
            shipped_at: o.shipped_at,
        }
    }
}
```

### Endpoint table (fill before coding)

| Method | Path                          | Use-case               | Request DTO          | Response DTO         | Status codes         |
| ------ | ----------------------------- | ---------------------- | -------------------- | -------------------- | -------------------- |
| POST   | `/v1/orders`                  | `CreateOrder`          | `CreateOrderRequest` | `OrderResponse`      | 201, 400, 409        |
| GET    | `/v1/orders/{id}`             | `GetOrder`             | —                    | `OrderResponse`      | 200, 404             |
| POST   | `/v1/orders/{id}/ship`        | `ShipOrder`            | —                    | `OrderResponse`      | 200, 404, 409        |
| GET    | `/bff/web/order-detail/{id}`  | `OrderDetailScreen`    | —                    | `OrderDetailView`    | 200, 404             |

### Conventions

- Path → kebab-case plural resources (`/v1/order-items`, not `/v1/orderItem`)
- JSON fields → `snake_case` (matches Rust + most clients tolerate it). Document explicitly if the client wants `camelCase` — then use `#[serde(rename_all = "camelCase")]`.
- IDs in JSON → string UUIDs (Serde does this by default when `uuid::Uuid` has `serde` feature).
- Timestamps → RFC3339 UTC (`chrono::DateTime<Utc>` serializes correctly out of the box).
- Money → integer minor units (`total_cents: i64`). Never floats over the wire.
- Errors → uniform shape `{ "error": "...", "code": "..." }` per `rust-error-handling` skill.
- Pagination → cursor-based `?cursor=...&limit=...` for collections you expect to grow.

### API checklist

- [ ] Every endpoint has a use-case in `application/` — handlers stay thin
- [ ] Every request DTO has `Deserialize`, every response DTO has `Serialize`
- [ ] Every DTO ↔ domain conversion lives at the application layer `application/`
- [ ] Every error path maps to `AppError` (see `rust-error-handling`)
- [ ] BFF endpoints documented as such (path prefix + comment) — they may break with UI changes
- [ ] Versioning decided up front (`/v1` prefix or none, but consistent)

## Step 3 — SQL schema

### Translation rules (domain → table)

| Domain                            | SQL                                                                |
| --------------------------------- | ------------------------------------------------------------------ |
| Entity `Order`                    | `CREATE TABLE orders (...)` — plural, snake_case                   |
| `OrderId(Uuid)`                   | `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`                    |
| `user_id: UserId` (FK)            | `user_id UUID NOT NULL REFERENCES users(id)`                       |
| `status: OrderStatus` (enum)      | `status TEXT NOT NULL CHECK (status IN ('pending','paid',...))`    |
| `total: Money` (cents)            | `total_cents BIGINT NOT NULL CHECK (total_cents >= 0)`             |
| `Email` (unique value object)     | `email TEXT NOT NULL` + `CREATE UNIQUE INDEX ... ON users(email)`  |
| `created_at: DateTime<Utc>`       | `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`                    |
| `shipped_at: Option<DateTime>`    | `shipped_at TIMESTAMPTZ` (nullable)                                |

### Migration skeleton

```sql
-- migrations/20260506140000_create_orders.sql
-- Rollback: DROP TABLE orders; (no data preserved)

CREATE TABLE orders (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID         NOT NULL REFERENCES users(id),
    status       TEXT         NOT NULL CHECK (status IN ('pending','paid','shipped','cancelled')),
    total_cents  BIGINT       NOT NULL CHECK (total_cents >= 0),
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    shipped_at   TIMESTAMPTZ
);

CREATE INDEX orders_user_id_idx ON orders (user_id);
CREATE INDEX orders_status_idx  ON orders (status) WHERE status IN ('pending','paid');
```

> For migrations against a populated table (NOT NULL rollouts, FK validation, CONCURRENTLY indexes), defer to the `sqlx-migration` skill — it has the full safety checklist.

### Schema checklist

- [ ] Table name is plural snake_case
- [ ] PK is `id UUID` with default — never auto-increment for distributed systems
- [ ] Every FK has an index (Postgres does NOT auto-index FKs)
- [ ] Every domain enum has a CHECK constraint matching the variant set exactly
- [ ] Every domain invariant that can be expressed in SQL is expressed in SQL (defense in depth)
- [ ] Timestamps are `TIMESTAMPTZ`, never `TIMESTAMP` (no timezone)
- [ ] Money is `BIGINT` cents or `NUMERIC(precision, scale)` — never `REAL`/`DOUBLE`
- [ ] Soft-delete uses `deleted_at TIMESTAMPTZ` (nullable), and read queries filter it
- [ ] After applying: `cargo sqlx prepare --workspace`

## Step 4 — Cross-layer consistency

After all three layers exist, verify:

| Domain field            | DTO field            | DB column            | Match?  |
| ----------------------- | -------------------- | -------------------- | ------- |
| `Order.id: OrderId`     | `id: Uuid`           | `id UUID PK`         | ✅      |
| `Order.total: Money`    | `total_cents: i64`   | `total_cents BIGINT` | ✅      |
| `Order.status` enum     | `status: String`     | `status TEXT CHECK`  | ⚠ verify CHECK list = enum variants |

A divergence here is a bug:

- DB allows `'refunded'` but domain enum doesn't have it → reads will panic at deserialization
- DTO exposes a domain field that's supposed to be internal → leak
- Domain has an invariant the DB can't enforce → race conditions can break it

## Reporting format (when extracting from a ticket)

```
## Model & Contract Extraction — <ticket name>

### Domain
- Entities: <list>
- Value objects: <list>
- Enums: <list>
- Invariants: <list, marked [domain] / [db] / [both]>
- Use-cases: <list>

### API
- Strategy: REST | BFF | mixed
- Endpoints:
  | METHOD | PATH | use-case | request DTO | response DTO | codes |

### SQL
- New tables: <list>
- Modified tables: <list>
- New indexes: <list>
- Migration files: <list>

### Open questions
- <anything ambiguous in the ticket that needs PO clarification before code>
```

## Never do

- ❌ Reuse a domain struct as a DTO (couples wire format to internal evolution)
- ❌ Use `f64` for money anywhere (domain, DTO, or DB)
- ❌ Use `String` in the domain for a finite set of states — use an enum
- ❌ Store enums as Postgres `ENUM` types (hard to migrate) — prefer `TEXT` + `CHECK`
- ❌ Skip the FK index — Postgres does not create one automatically
- ❌ Write code before the three-layer extraction is at least sketched
- ❌ Let the DB schema drive the domain model (DB is an implementation detail)

## Always do

- ✅ Newtype every ID (`OrderId(Uuid)`, not raw `Uuid`)
- ✅ Validate at the boundary — `Email::parse()` in domain, `serde` validation at DTO, `CHECK` in DB
- ✅ Express invariants in *both* domain and DB when possible
- ✅ One use-case per endpoint; handlers stay thin
- ✅ Document BFF endpoints as such — they're allowed to break with the UI
- ✅ Run `cargo sqlx prepare --workspace` after every schema change
- ✅ Surface ambiguities back to the PO/spec author rather than guessing
