---
name: functional-style
description: Use when writing or reviewing any Rust code in this project. Favor a functional style — combinators on `Option`/`Result`/`Iterator` (`map`, `and_then`, `filter`, `fold`, `collect`, `?`), pure expressions, immutable bindings — and limit explicit `match` to the cases where it genuinely earns its keep (exhaustive enum dispatch with non-trivial bodies). Triggers on keywords like functional, map, and_then, flat_map, filter, fold, collect, iterator, combinator, match, pattern matching, mutable, mut, for loop, if let, refactor, style.
allowed-tools: Read, Grep, Glob, Edit, Write
---

# Functional Style

## The rule

- **Prefer combinators over manual control flow.** Chain `map` / `and_then` / `or_else` / `filter` / `unwrap_or` / `?` on `Option`, `Result`, and iterators. Reach for `match` only when it provides something a combinator cannot.
- **Code is an expression, not a sequence of statements.** A function body should read as a single value being built, not a script that mutates a result variable.
- **Bindings are immutable by default.** `let mut` is a smell — challenge every occurrence. Push mutation to small, contained scopes (e.g. accumulators inside a `fold`, or the SQLx layer).
- **Pure where possible, effectful at the edges.** Use-cases compose pure parsing + repository calls; side effects live in repositories and the HTTP layer.
- **Iterator chains over `for` loops** when the loop is building a value. `for` is acceptable when its only purpose is side effects (logging, dispatching).

## When to use `match` vs combinators

`match` is the right tool when **all** of these hold:
- You are destructuring an enum with **three or more meaningful variants**, and
- each branch has a **distinct, non-trivial body**, and
- you want the compiler to enforce **exhaustiveness** so a new variant breaks the build.

Otherwise, prefer combinators. Concretely:

| Situation                                            | Use                                |
|------------------------------------------------------|------------------------------------|
| `Option<T>` → `Option<U>`                            | `.map(...)`                        |
| `Option<T>` → `Option<U>` where `U` is `Option<...>` | `.and_then(...)`                   |
| `Option<T>` → `T` with a fallback                    | `.unwrap_or(...)` / `.unwrap_or_else(...)` |
| `Result<T, E>` → `Result<U, E>`                      | `.map(...)`                        |
| `Result<T, E>` → propagate to caller                 | `?`                                |
| `Result<T, E>` → wrap the error                      | `.map_err(...)`                    |
| `Option<T>` → `Result<T, E>`                         | `.ok_or(...)` / `.ok_or_else(...)` |
| Test a boolean condition on a value                  | `.filter(...)`                     |
| Collapse `Option<Option<T>>` / `Result<Result<...>>` | `.flatten()`                       |
| Iterate to build a collection                        | `.iter().map(...).collect()`       |
| Iterate to build a scalar / fold structure           | `.fold(init, |acc, x| ...)`        |

## Patterns to prefer

### Parsing a string into an enum

```rust
// ❌ Pattern-matches even though there are only two outcomes; verbose for what it does.
fn parse_visibility(raw: &str) -> Visibility {
    match raw.to_uppercase().as_str() {
        "PERSONAL" => Visibility::Personal,
        _ => Visibility::Shared,
    }
}

// ✅ Expression-oriented, no `match` needed.
fn parse_visibility(raw: &str) -> Visibility {
    raw.eq_ignore_ascii_case("PERSONAL")
        .then_some(Visibility::Personal)
        .unwrap_or(Visibility::Shared)
}
```

When the enum has more than two variants and each maps to a distinct string, prefer a small lookup or `TryFrom<&str>` implementation that returns `Result<Self, ParseError>` — still no top-level `match` at the call site, since the caller uses `?`.

### Conditional propagation

```rust
// ❌ Statement-oriented, mutates a flag.
let mut allowed = false;
if let Some(member) = membership {
    if member.role == Role::Admin {
        allowed = true;
    }
}
if !allowed {
    return Err(AccessDenied);
}

// ✅ Composes `Option` combinators, returns directly.
membership
    .filter(|m| m.role == Role::Admin)
    .ok_or(AccessDenied)?;
```

### Building a collection

```rust
// ❌ `for` + `push` — manual loop building a Vec.
let mut names = Vec::with_capacity(users.len());
for u in &users {
    if u.active {
        names.push(u.name.clone());
    }
}

// ✅ Iterator chain, one expression.
let names: Vec<_> = users
    .iter()
    .filter(|u| u.active)
    .map(|u| u.name.clone())
    .collect();
```

### Wrapping a repository error

```rust
// ✅ Already idiomatic — combinator chain, no `match`.
state
    .magic_list_repository
    .create(&username, command)
    .await
    .map_err(|source| CreateMagicListError::Repository { name, source })
```

### Chaining fallible operations

```rust
// ❌ Pyramid of nested `match`.
let parsed = match parse(input) {
    Ok(p) => p,
    Err(e) => return Err(MyError::Parse(e)),
};
let validated = match validate(parsed) {
    Ok(v) => v,
    Err(e) => return Err(MyError::Validate(e)),
};
let saved = match repo.save(validated).await {
    Ok(s) => s,
    Err(e) => return Err(MyError::Repository(e)),
};

// ✅ `?` plus `map_err` flattens the chain.
let saved = parse(input)
    .map_err(MyError::Parse)
    .and_then(|p| validate(p).map_err(MyError::Validate))?;
let saved = repo.save(saved).await.map_err(MyError::Repository)?;
```

### Folding instead of a manual accumulator

```rust
// ❌ Manual loop with `mut total`.
let mut total = 0;
for item in &items {
    total += item.quantity * item.price;
}

// ✅ `fold` — total is an immutable expression.
let total: i64 = items
    .iter()
    .map(|i| i.quantity * i.price)
    .sum();
```

## When `match` is the right answer

Keep `match` for exhaustive enum dispatch with substantive branches:

```rust
// ✅ Three real variants, distinct semantics, compiler-enforced exhaustiveness.
impl ResponseError for MiddlewareError {
    fn status_code(&self) -> StatusCode {
        match self {
            MiddlewareError::MissingUsername      => StatusCode::UNAUTHORIZED,
            MiddlewareError::InvalidDateFormat(_) => StatusCode::BAD_REQUEST,
            MiddlewareError::AccessDenied(_)      => StatusCode::FORBIDDEN,
            MiddlewareError::NotFound(_)          => StatusCode::NOT_FOUND,
            MiddlewareError::AlreadyExists(_)     => StatusCode::CONFLICT,
            MiddlewareError::CreateMagicList(_)   => StatusCode::INTERNAL_SERVER_ERROR,
            // ... every variant covered, compiler will fail the build if a new variant is added
        }
    }
}
```

This is precisely where combinators would *lose* information: you want the compiler to break the build when a new variant appears.

## Anti-patterns

### ❌ `match` for a single happy path

```rust
// BAD
let name = match user.name {
    Some(n) => n,
    None => return Err(MissingName),
};

// GOOD
let name = user.name.ok_or(MissingName)?;
```

### ❌ `if let` ladders that should be `and_then`

```rust
// BAD
if let Some(family) = family {
    if let Some(parent) = family.parent {
        if parent.email_verified {
            return Ok(parent.email);
        }
    }
}
Err(NoVerifiedEmail)

// GOOD
family
    .and_then(|f| f.parent)
    .filter(|p| p.email_verified)
    .map(|p| p.email)
    .ok_or(NoVerifiedEmail)
```

### ❌ Mutable accumulator when a combinator would do

```rust
// BAD
let mut errors = vec![];
for item in items {
    if let Err(e) = validate(item) {
        errors.push(e);
    }
}

// GOOD
let errors: Vec<_> = items
    .iter()
    .filter_map(|i| validate(i).err())
    .collect();
```

### ❌ Side effects inside `map`

`map` is for pure transformation. If you find yourself doing I/O or logging inside `.map(...)`, the function is no longer a transformation — split it: build the data with combinators, then iterate (`for`) only to dispatch the side effects.

```rust
// BAD — map silently performs I/O; behavior depends on iterator being consumed.
items.iter().map(|i| {
    log_audit(i);
    transform(i)
}).collect()

// GOOD — side effects are explicit, transformation stays pure.
for i in &items {
    log_audit(i);
}
let transformed: Vec<_> = items.iter().map(transform).collect();
```

### ❌ `.unwrap()` to escape an `Option`/`Result`

`unwrap` and `expect` are banned outside tests and `main()` bootstrap. They're imperative shortcuts; use `?`, `ok_or`, `unwrap_or`, or propagate up.

## Always do

- ✅ Express function bodies as a single value (return an expression, not the last `let`)
- ✅ Default to immutable `let`; treat `let mut` as something to justify
- ✅ Reach for `?` first when propagating errors; reach for `map_err` to add layer context
- ✅ Use iterator combinators (`map`, `filter`, `filter_map`, `flat_map`, `fold`, `sum`, `collect`) when building a value
- ✅ Reserve `match` for exhaustive enum dispatch with non-trivial branches
- ✅ Keep functions pure; push I/O to repositories and the HTTP layer

## Never do

- ❌ Use `match` for `Option`/`Result` when a combinator exists (`map`, `and_then`, `ok_or`, `unwrap_or`, `?`)
- ❌ Build a `Vec` with `let mut v = vec![]; for ... { v.push(...) }` — use `.collect()`
- ❌ Use `if let Some(x) = ... { y = ... }` to set a mutable variable — return the expression
- ❌ Embed side effects (I/O, logging, mutation of outside state) inside `map` / `filter` / `fold` closures
- ❌ Use `.unwrap()` / `.expect()` outside tests and `main()`

## Refactoring checklist (existing code)

1. [ ] Grep for `match` blocks with one or two real branches — most collapse into `map` / `and_then` / `filter` / `ok_or` / `unwrap_or`
2. [ ] Grep for `let mut` — for each occurrence, check whether a `fold`, `collect`, or expression-form rewrite removes the mutation
3. [ ] Grep for `for ... { ....push(...) }` patterns — replace with iterator chains ending in `.collect()`
4. [ ] Replace `if let Some(x) = opt { ... } else { ... }` returning a value with `opt.map(...).unwrap_or(...)` (or `match` only if branches are substantive)
5. [ ] Remove side effects from inside `map`/`filter` closures — split into a pure transformation pass and an explicit side-effect pass
6. [ ] Make sure no `.unwrap()` / `.expect()` was introduced outside tests / `main()`
7. [ ] `cargo clippy --all-targets --all-features -- -D warnings` clean — clippy already flags many of these (`needless_collect`, `manual_map`, `single_match`, `redundant_closure`)
