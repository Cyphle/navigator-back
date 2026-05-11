---
name: architecte_feature
description: Trigger when the user expresses a functional or business need to design (e.g. "I want to add X", "how do we implement Y", "we need a feature that…"). Breaks the need down into technical bricks (vertical slices), produces the front and back software architecture, the associated design system, writes an ADR if needed, saves the plan as markdown in `.claude/plans/`, then submits it to the user for review BEFORE any implementation.
---

# Skill: architecte_feature

## Goal

Turn a functional need into a complete technical plan, **reviewed and approved** before any code is written.

## Expected inputs

- A functional need in natural language (user story, business request, fuzzy ticket, etc.).
- If the frontend / backend stack is not explicit, read the accessible `CLAUDE.md` files before assuming. If still ambiguous, ask the user.

## Steps (follow in order)

### 1. Clarification

- Explicitly list the **open questions** (actors, edge cases, non-functional constraints, out-of-scope items).
- **Do NOT proceed** until blocking questions are answered by the user.
- Rephrase the need in your own words to validate understanding.

### 2. Breakdown into technical bricks (vertical slices)

- Decompose the need into vertical slices that are **independently** deliverable and testable.
- Each slice must deliver observable business value (no purely technical slices).
- For each brick: name, business value, dependencies, acceptance criteria.

### 3. Software architecture

#### Backend
- Hexagonal layers impacted: `domain`, `application`, `infrastructure`, `api` (see `.claude/rules/architecture.md`).
- Domain types (entities, value objects, errors).
- Ports (traits) needed and concrete adapters.
- Use-cases / orchestrations.
- Data model + SQLx migrations (list the changes).
- HTTP endpoints + DTOs + error codes.
- Observability strategy (spans, structured logs).

#### Frontend (if applicable)
- Stack and conventions (read in the frontend repo's `CLAUDE.md`; if absent, ask).
- Components to create / modify / reuse.
- Local vs global state, caching strategy.
- Routing and navigation.
- API calls, error handling, loading states.
- Tests (unit, integration, e2e).

### 4. System design (infrastructure architecture)

Map every interaction the feature has with the outside world or with shared infrastructure. For each external touchpoint, document:

- **External services / third-party APIs**
  - Which service, endpoint, version, auth mechanism (token, mTLS, OAuth…).
  - SLA / rate limits / quotas, expected latency, timeout budget.
  - Failure mode: retry policy (idempotency, backoff), circuit breaker, fallback, dead-letter.
  - Secrets management (where they live, rotation).
- **Database**
  - Tables/collections read or written, indexes needed, transaction boundaries.
  - Migration strategy (online/offline, locking, backfill plan).
  - Read/write ratio, expected volume, hot keys.
- **Cache** (Redis, in-memory…)
  - Keys, TTL, invalidation strategy, stampede protection.
- **Messaging / queues / events** (Kafka, RabbitMQ, SQS, webhooks…)
  - Topics/queues, message schema, at-least-once vs exactly-once, ordering, replay.
- **Storage** (object storage, files)
  - Bucket, retention, encryption, public/private access.
- **Scheduling / cron / background jobs**
  - Trigger, isolation, observability.
- **Cross-cutting concerns**
  - Authentication / authorization (who can call what).
  - Multitenancy, data residency, PII handling.
  - Observability: metrics, traces, alerts to add.
  - Performance & capacity budget (RPS, p95 latency, payload size).
  - Cost impact (egress, calls billed per request).
- **Sequence diagram or flow** (ASCII / Mermaid) when ≥ 2 external dependencies interact.

If the feature is purely local (no I/O, no external dep, no DB), write "N/A — no infrastructure impact" and justify briefly.

### 5. Design system

- Identify **reusable** UI components (existing to reuse, or to create).
- Tokens needed (colors, typography, spacing, motion) — only if new.
- Interaction and accessibility patterns (focus, ARIA, keyboard navigation).
- If the feature has no UI impact, explicitly write "N/A — pure backend feature".

### 6. ADR (Architecture Decision Record) — conditional

Create an ADR **only if** the feature involves one of the following:
- Adding a major dependency (new crate, new external service).
- Introducing a new cross-cutting pattern.
- Structural refactoring affecting more than one layer.
- A non-trivial choice between several viable options.
- A deviation from a critical project rule.

Format: `docs/adr/NNNN-<kebab-case-title>.md` with sections:
- **Context** (the problem, the constraints)
- **Decision** (the chosen option, stated in active voice)
- **Alternatives considered** (with their trade-offs)
- **Consequences** (positive, negative, to monitor)
- **Status**: `proposed` | `accepted` | `superseded by ADR-NNNN`

If no ADR is necessary, write in the plan: "ADR: not necessary — reason: …".

### 7. Writing the plan

Create the file: `.claude/plans/<YYYY-MM-DD>-<feature-slug>.md` (use today's date, slug in kebab-case).

Required structure:

```markdown
# Plan: <title>

- **Date**: YYYY-MM-DD
- **Status**: draft
- **Author**: architecte_feature
- **Source need**: <one-sentence summary>

## 1. Functional need
<clear rephrasing of the need and the expected value>

## 2. Open questions and answers
- **Q1**: … → **A**: …
- …

## 3. Technical bricks (vertical slices)
- **Slice 1 — <name>**
  - Business value: …
  - Dependencies: …
  - Acceptance criteria:
    - [ ] …
- **Slice 2 — <name>**
  - …

## 4. Backend architecture
### Domain
…
### Application (use-cases)
…
### Infrastructure (adapters, SQL, migrations)
…
### API (endpoints, DTOs, errors)
…

## 5. Frontend architecture
<or "N/A — pure backend feature">

## 6. System design (infrastructure)
### External services
| Service | Endpoint | Auth | Timeout | Retry | Fallback |
|---------|----------|------|---------|-------|----------|
| …       | …        | …    | …       | …     | …        |

### Database
- Tables touched: …
- Indexes / migrations: …
- Transactional boundaries: …

### Cache / queues / storage / jobs
…

### Cross-cutting
- AuthZ: …
- Observability (metrics / traces / alerts): …
- Capacity budget (RPS, p95): …
- Cost impact: …

### Flow / sequence
<ASCII or Mermaid diagram if ≥ 2 external deps, otherwise omit>

## 7. Design system
<components, tokens, patterns — or "N/A">

## 8. ADR
- [ ] ADR-NNNN: <title>  (or: "not necessary — reason: …")

## 9. Risks and rejected alternatives
…

## 10. Implementation plan by slice
<recommended implementation order, dependencies between slices>
```

### 8. Review before implementation (mandatory GATE)

- Announce to the user the **path of the created plan**.
- Present a short summary (3-6 lines): slices, key architectural choices, possible ADR.
- **Wait for explicit approval** ("ok", "go", "approved", or equivalent).
- If the user asks for adjustments, **update the markdown file** (do not create a new file) and request review again.
- Once approved, update `Status: approved` in the file, and only then propose moving on to implementation.

## Strict rules

- ❌ **Never implement before plan approval.** No source file creation, no migration, no edit outside of the markdown plan.
- ❌ No purely technical slices ("set up layer X"): each slice must deliver a business capability.
- ✅ The plan lives in `.claude/plans/` and remains the reference document throughout implementation.
- ✅ If the frontend stack is not accessible / documented, ask the user before inventing.
- ✅ Reuse the business vocabulary of the need (no gratuitous renaming).

## After approval

Implementation follows the slice order from the plan. For each slice:
1. Plan mode (Shift+Tab) if non-trivial.
2. TDD on the domain.
3. Integration tests on the infrastructure (`#[sqlx::test]`).
4. Delegate to `code-reviewer` before PR.

The markdown plan serves as a checklist: tick acceptance criteria off as you go.
