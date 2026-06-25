---
name: writing-spec-reports
description: Use when writing a Navigator report — brainstorm synthesis, status report, or converting an existing functional spec from French Markdown to English HTML. Produces a self-contained HTML file in surgical style (greyscale, no decoration). Triggers on keywords like report, synthesis, brainstorm summary, status, convert spec, functional spec to HTML, rapport.
allowed-tools: Read, Grep, Glob, Write, Edit
---

# Writing Navigator reports

A report captures the state or the synthesis of a brainstorm. It is written in **English**, as a **self-contained HTML file** (embedded `<style>`, no external CSS), in a **surgical** style: no fluff, no emoji, no decorative dividers, bold reserved for genuine keywords. Prefer tables and lists over prose walls. Communication must be efficient and straight to the point.

## Two jobs this skill covers

1. **Brainstorm synthesis / status report** — `specs/reports/YYYY-MM-DD-<topic>.html`.
2. **Functional spec conversion** — translate an existing `specs/functional/*.md` (French) into `specs/functional/*.html` (English), same surgical style, preserving every decision, table, data model, and edge case. The decisions journal at the end of each spec must be carried over faithfully.

## Conducting the brainstorm (non-negotiable)

The brainstorm that precedes any report is **use-case first**. Always:

- Start from **usage**: who does what, in what situation, to what end. Make the user describe the behaviour they want, ideally with a concrete scenario (real people, real dates).
- **Never** ask shortened design/technical questions — banned framings include "stored vs derived", "granularity", "periodicity", "flag vs computed", and any option list phrased in implementation terms. They short-circuit the usage discussion and confuse the user.
- Phrase questions as scenarios or plain "what do you want to happen?" prompts. One question at a time.
- The **data model comes after**, and only if it falls out naturally from the agreed use cases — present it as a consequence to validate, never as the entry point.
- Consider the `product-owner` agent to frame use cases when a domain is broad.

## Scope rules (non-negotiable)

Same as the project's brainstorm rules:

- **No software architecture** (layers, traits, injection, module placement). Only the **database model** is an allowed technical notion.
- **Errors** follow the project's common principle; reference it, do not restate the mechanism. HTTP codes are **standard web codes**.
- **API contract reference** = React front + Fastify mock: camelCase JSON, SCREAMING_SNAKE enums, mutations return the full aggregate, `createdAt`/`updatedAt` everywhere.

## Structure

Report (synthesis/status) — these `<h2>` sections:

1. **Context** — what this report is about, in 1–3 sentences.
2. **Decisions** — the decisions taken, as a list or table. State each plainly; carry stable IDs (e.g. T1, D2) when they exist.
3. **Open points** — what is unresolved, with the choice at stake. If none, say "None".
4. **Next steps** — concrete, ordered.

Converted functional spec — keep the spec's own section structure; do not flatten it into the four report sections. Mirror the source faithfully, only translating and re-styling.

## Metadata header

Under `<h1>`, a `<dl class="meta">` with at least: `Date`, `Status`. Add `Scope`/`Theme` when useful.

## HTML template (copy verbatim, fill the brackets)

```html
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>[Title]</title>
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
<h1>[Title]</h1>
<dl class="meta">
  <dt>Date</dt><dd>[YYYY-MM-DD]</dd>
  <dt>Status</dt><dd>[draft | final]</dd>
</dl>

<h2>Context</h2>
<p>[...]</p>

<h2>Decisions</h2>
<ul><li>[...]</li></ul>

<h2>Open points</h2>
<ul><li>[...]</li></ul>

<h2>Next steps</h2>
<ol><li>[...]</li></ol>
</body>
</html>
```

## Quality bar (self-check before finishing)

- Straight to the point; no padding sentences.
- Decisions are stated, not narrated. Open points name the choice at stake.
- Conversions preserve every table, data model, decision ID, and edge case from the source.
- No layer/trait/injection talk; only DB model as technical detail.
- No emoji, no inline `style=""` attributes (style lives only in the `<head>` block), minimal bold.
