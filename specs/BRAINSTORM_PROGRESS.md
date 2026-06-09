# Navigator — Suivi du brainstorm specs

> Fichier d'état pour reprendre après une coupure / compaction de contexte.
> **À mettre à jour à la fin de chaque session de travail sur un thème.**
> Garder ce fichier COMPACT (c'est un résumé, pas une spec).

## Démarche convenue

Pour chaque thème (= 1 fichier `specs/functional/*.md`) :
1. **Brainstorm** : lister les features, détecter les edge cases, clarifier les règles métier.
2. **Stratégie d'implémentation** : identifier l'écart back Rust ↔ contrat front, et le plan.
3. **Enrichir directement** le `functional/*.md` du thème (1 seule source de vérité / thème).

### Décisions cadres
- **Sortie** : on enrichit `specs/functional/*.md` directement.
- **Contrat API de référence** : le **front React** (`navigator-front/src`, couche `src/services/`) + son **mock Fastify** (`navigator-front/server`). Le back Rust **s'aligne** dessus, mais le contrat **peut évoluer** selon nos discussions.
- **État du code (constat initial)** :
  - Front React = avancé. Mock Fastify riche (CRUD complet sur la plupart des domaines).
  - Back Rust = réel sur **user / family / magic_list** ; **bank_account** lecture seule ; **calendar / recipe / meal / shopping_list** = stubs renvoyant `[]`.

## Ordre des thèmes & statut

| # | Thème (fichier) | Spec init | Back Rust | Front React | Brainstorm |
|---|-----------------|-----------|-----------|-------------|------------|
| 1 | magic_list | bonne | réel | réel | ✅ terminé |
| 2 | transverse_partage | mince | (transverse) | (transverse) | 🔵 EN COURS |
| 3 | calendrier | mince | stub | réel | ⚪ à faire |
| 4 | comptes_bancaire | vide | lecture seule | réel | ⚪ à faire |
| 5 | recettes_repas_previsionnels | vide | stub | réel | ⚪ à faire |
| 6 | dashboard | vide | stub | réel | ⚪ à faire |
| 7 | configuration (familles + profil) | squelette | partiel | partiel | ⚪ à faire |

Légende : ⚪ à faire · 🔵 en cours · ✅ terminé

## Où on s'est arrêté (curseur)

**Thème actif : transverse_partage (#2) — brainstorm EN COURS.**
- Résultats du brainstorm partage → **`functional/transverse_partage.md`** (spec dédiée, décisions T1→T9, exemple magic_list, modèle de données, archi).
- Décision structurante : **le partage est PAR USER**. « Partager à une famille » = truc du front (expansion en N grants + regroupement d'affichage). Le back ne connaît pas les familles dans le partage ⇒ **une seule table `shares`** de grants. Modèle radicalement simplifié.
- **Brainstorm partage bouclé** : modèle fonctionnel + modèle de données (1 table) + principe erreurs/HTTP (= principe projet, codes web standards). Décisions T1→T9 figées dans `transverse_partage.md`.
- **Cadre méthodo (important)** : dans le brainstorm on ne discute PAS d'architecture logicielle (placement modules, couches) → ça relève des *rules* d'archi clean/hexagonale. Seule notion technique discutée : le **modèle de base de données**.
- ✅ `magic_list.md` amendé pour cohérence avec le modèle de partage par user (struct sans visibility/family/excluded, accès délégué au transverse, D5 marqué obsolète, routes ajustées).
- **Reprendre à** : prochain thème — #3 **calendrier** (front réel / back stub).
- ⚠️ La section « Accès & partage » de `magic_list.md` est **supersédée** par `transverse_partage.md` → à amender quand le modèle est figé.

## Journal des décisions transverses

> Décisions qui impactent plusieurs thèmes. À remplir au fil de l'eau.

- **Convention API** : JSON en **camelCase** ; enums en `SCREAMING_SNAKE`. Le back Rust s'aligne (front = référence).
- **Timestamps** : `createdAt` / `updatedAt` exposés dans toutes les vues.
- **Réponses de mutation** : renvoyer l'agrégat complet après mutation (ex: MagicList complète après mutation d'item).
- **Modèle de partage** : décisions détaillées (T1→T9) et spec complète déplacées dans **`functional/transverse_partage.md`**. Ce fichier-ci ne garde que le curseur.
