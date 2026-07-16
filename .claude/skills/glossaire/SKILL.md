---
name: glossaire
description: Use when a business/domain term is validated during a Navigator brainstorm or spec discussion and must be recorded, or when looking one up. Maintains the DDD ubiquitous language glossary at specs/glossary.md — bilingual FR↔EN entries with description, alternatives, examples. Triggers on keywords like glossaire, glossary, ubiquitous language, langage ubiquitaire, terme métier, DDD term, vocabulaire, définition de terme, "on valide le terme", "ajoute au glossaire".
allowed-tools: Read, Write, Edit
---

# Navigator glossary — ubiquitous language

The glossary is the project's **DDD ubiquitous language**: the shared vocabulary that binds business talk to code. It lives at **`specs/glossary.md`** (single source of truth). The user speaks French, the code is English, so **every term is recorded in both languages** — the FR label is the headword, the EN label is the code identifier it maps to.

## When to use

- A business/domain term is **validated** during a brainstorm or spec discussion ("on valide le terme X", "ajoute X au glossaire").
- You need to **look up** or **reuse** an agreed term instead of inventing a synonym.
- A term's meaning, alternatives, or examples **changed** and the entry must be updated.

Do **not** invent or pre-fill terms that were not explicitly validated. The glossary records agreed vocabulary, not guesses.

## File organization

The glossary is grouped by **bounded context** — a `##` section per context — because DDD ubiquitous language is context-scoped (the same word can differ across contexts). The **Transverse** section comes first and holds concepts shared by several domains (ownership, sharing, access…). Current sections: `Transverse`, `magic_list`, `calendar`, `bank_account`, `recipes & meals` (add a section when a new domain gets a term). The section header carries the context, so entries do **not** repeat it.

## Entry contract (what an entry IS)

One `###` heading per term (the **French** label), inside its context section, followed by these fields in this order:

```markdown
### Liste magique
- **Anglais / code** : Magic list (`MagicList`, table `magic_list`)
- **Description** : Liste réplicable, partageable et cochable, appartenant à un utilisateur et contenant des articles.
- **Alternatives** : liste.
- **Exemples** : liste de courses, liste de valise, liste de tâches.
```

Field rules:
- **Anglais / code** — *mandatory*. The English term as used in the codebase, with the code identifier in backticks when it maps to one (entity, table, enum value). This is the FR↔EN bridge.
- **Description** — *mandatory*. One or two plain sentences. What the term means in the domain, not how it is implemented.
- **Alternatives** — *mandatory field, `—` when none*. Synonyms, rejected wordings, or near-terms the team decided **not** to use (say which is canonical).
- **Exemples** — *mandatory*. At least one concrete example (a real-looking value, phrase, or scenario). Grounds the term.

## Procedure to add or update a term

1. **Read `specs/glossary.md`** first (create it from the template below if absent).
2. **Search for the term** (FR and EN) to avoid a duplicate. If it exists, **update in place** rather than adding a second entry.
3. Decide its **section**: a shared/cross-cutting concept goes in **Transverse**; otherwise its bounded-context section (create the `##` section if it's a new domain). A concept already in Transverse is **not** duplicated inside a domain section.
4. Write the entry following the contract above and **insert it in alphabetical order** (accent-insensitive) within its section, so lookup stays trivial.
5. Keep it compact — the glossary is a reference, not a spec. Details belong in `specs/functional/*.html`; link there only if genuinely useful.

## File template (when creating `specs/glossary.md`)

```markdown
# Navigator — Glossaire (langage ubiquitaire DDD)

> Vocabulaire métier partagé. Un terme est ajouté ici **une fois validé** en brainstorm/spec.
> Chaque terme est bilingue : libellé **français** (titre) ↔ **anglais / identifiant code**.
> Champs : Anglais/code · Description · Alternatives · Exemples.
> Organisé par **bounded context** ; les concepts communs vivent dans **Transverse**. Termes triés alpha dans chaque section.

## Transverse
### <terme français>
- **Anglais / code** : <English term> (`<code_id>`)
- **Description** : <1–2 phrases>
- **Alternatives** : <synonymes / termes rejetés, ou —>
- **Exemples** : <au moins un exemple concret>

## <bounded context, ex. magic_list>
### <terme français>
- **Anglais / code** : <English term> (`<code_id>`)
- **Description** : <1–2 phrases>
- **Alternatives** : <synonymes / termes rejetés, ou —>
- **Exemples** : <au moins un exemple concret>
```

## Quality bar (self-check before finishing)

- Term recorded in **both** languages; the code identifier is present when one exists.
- Description is domain-level, one or two sentences, no implementation/layer talk.
- Alternatives field present (`—` if none); Exemples has at least one concrete example.
- Entry sits in the **right context section**, alphabetical within it; shared concepts live in **Transverse**, not duplicated per domain.
- Only **validated** terms are added — nothing invented.
