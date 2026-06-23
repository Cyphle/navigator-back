---
name: writing-spec-tickets
description: Use when writing or updating a Navigator spec ticket. Produces one self-contained HTML file per ticket under specs/tickets/<vertical>/, surgical style (greyscale, no decoration), priority-first filename, four mandatory sections (Business context, Objective, Expected outcome, Acceptance & validation). Triggers on keywords like ticket, backlog, decompose, vertical slice, spec ticket, acceptance criteria, write tickets.
allowed-tools: Read, Grep, Glob, Write, Edit
---

# Writing Navigator spec tickets

A ticket is a precise, testable unit of work derived from a functional spec. It is written in **English**, as a **self-contained HTML file** (embedded `<style>`, no external CSS), in a **surgical** style: no fluff, no emoji, no decorative dividers, bold reserved for genuine keywords. Prefer tables and lists over prose walls.

## Scope rules (non-negotiable)

These mirror the project's brainstorm rules — respect them or the ticket is wrong:

- **No software architecture.** Do not mention layers, traits, injection, controller/middleware/usecase/repository placement, or module boundaries. That belongs to the project's clean/hexagonal rules, not to a ticket.
- **The only technical notion allowed is the database model** (tables, columns, constraints, deltas).
- **Errors:** never reinvent per ticket. Refer to the project's common error principle (each layer wraps and re-maps via `#[source]`, up to the controller) and use **standard web HTTP codes** (401 / 403 / 404 / 409 / 500, etc.). State the code, not the mechanism.
- **API contract reference** is the React front + Fastify mock: JSON in **camelCase**, enums in **SCREAMING_SNAKE**, mutations return the full aggregate, `createdAt`/`updatedAt` everywhere. Align tickets to it; note divergences explicitly.

## File location and naming

- Path: `specs/tickets/<vertical>/<NNN>-<slug>.html`
- `<vertical>` ∈ `magic-list`, `sharing`, `calendar`, `bank-account`, `recipes-meals`, `dashboard`, `configuration`.
- `<NNN>` = **priority number, first in the filename**, so alphabetical order = priority order. Use gap-friendly steps (`010`, `020`, `030`, …) to insert between two later.
- `<slug>` = short kebab-case action, e.g. `010-rename-magic-list.html`.
- The ticket **ID** displayed in the document is `<vertical>/<NNN>` (e.g. `magic-list/020`).

## Mandatory structure

Exactly these four `<h2>` sections, in order:

1. **Business context** — why this exists. The need, the persona(s) concerned, the situation it solves. 2–5 sentences max. No solution talk.
2. **Objective** — the single outcome this ticket delivers, in one or two sentences. If you need "and", consider splitting the ticket.
3. **Expected outcome** — what is delivered, concretely:
   - behaviors / endpoints (method + route, request and response shape in camelCase),
   - database model deltas if any (new/changed columns, constraints),
   - **Out of scope** — an explicit short list of what this ticket does NOT cover.
4. **Acceptance & validation** — an **ordered list of testable checks**. Each check is observable (Given/When/Then or a concrete assertion on the API contract / HTTP code / edge case). No "it works". Cover the nominal path, the relevant edge cases, and the error codes.

## Metadata header

Right under `<h1>`, a `<dl class="meta">` with: `Vertical`, `Priority`, `Status` (draft | ready | done), `Depends on` (other ticket IDs or "none").

## HTML template (copy verbatim, fill the brackets)

```html
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>[vertical]/[NNN] — [Title]</title>
<style>
  :root { --ink:#1a1a1a; --muted:#666; --line:#d0d0d0; --code-bg:#f4f4f4; }
  body { color:var(--ink); background:#fff; font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,Helvetica,Arial,sans-serif; line-height:1.5; max-width:50rem; margin:2rem auto; padding:0 1.25rem; }
  h1 { font-size:1.5rem; margin:0 0 .25rem; }
  h2 { font-size:1.15rem; margin:2rem 0 .5rem; border-bottom:1px solid var(--line); padding-bottom:.2rem; }
  h3 { font-size:1rem; margin:1.25rem 0 .4rem; }
  p,ul,ol,table,pre { margin:.5rem 0; }
  ul,ol { padding-left:1.4rem; } li { margin:.15rem 0; }
  strong { font-weight:600; }
  dl.meta { display:grid; grid-template-columns:max-content 1fr; gap:.1rem .75rem; margin:.5rem 0 1.5rem; font-size:.9rem; color:var(--muted); }
  dl.meta dt { font-weight:600; } dl.meta dd { margin:0; }
  table { border-collapse:collapse; width:100%; font-size:.95rem; }
  th,td { border:1px solid var(--line); padding:.35rem .6rem; text-align:left; vertical-align:top; }
  th { font-weight:600; background:var(--code-bg); }
  code { font-family:"SF Mono",ui-monospace,Menlo,Consolas,monospace; background:var(--code-bg); padding:.05rem .3rem; border-radius:2px; font-size:.9em; }
  pre { background:var(--code-bg); padding:.75rem 1rem; overflow-x:auto; border:1px solid var(--line); border-radius:2px; } pre code { background:none; padding:0; }
</style>
</head>
<body>
<h1>[vertical]/[NNN] · [Title]</h1>
<dl class="meta">
  <dt>Vertical</dt><dd>[vertical]</dd>
  <dt>Priority</dt><dd>[NNN]</dd>
  <dt>Status</dt><dd>draft</dd>
  <dt>Depends on</dt><dd>[ids or none]</dd>
</dl>

<h2>Business context</h2>
<p>[why this exists, who needs it]</p>

<h2>Objective</h2>
<p>[the single outcome]</p>

<h2>Expected outcome</h2>
<p>[behaviors]</p>
<table>
  <tr><th>Method</th><th>Route</th><th>Behavior</th></tr>
  <tr><td>[GET]</td><td>[/...]</td><td>[...]</td></tr>
</table>
<h3>Out of scope</h3>
<ul><li>[explicitly excluded]</li></ul>

<h2>Acceptance &amp; validation</h2>
<ol>
  <li>[testable check — nominal]</li>
  <li>[testable check — edge case]</li>
  <li>[testable check — error code]</li>
</ol>
</body>
</html>
```

## Quality bar (self-check before finishing)

- One ticket = one objective. Split if it needs "and".
- Every acceptance item is observable and testable; none says "works correctly".
- Out-of-scope is present and explicit.
- No layer/trait/injection talk; only DB model as technical detail.
- HTTP codes are standard; errors point to the project principle.
- Filename starts with the priority number; ID matches the header.
- No emoji, no inline `style=""` attributes (style lives only in the `<head>` block), minimal bold.
