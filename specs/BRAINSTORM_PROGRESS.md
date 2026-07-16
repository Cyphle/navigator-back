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
| 2 | transverse_partage | mince | (transverse) | (transverse) | ✅ terminé |
| 3 | calendrier | mince | stub | réel | ✅ terminé |
| 4 | comptes_bancaire | vide | lecture seule | réel | ✅ terminé |
| 5 | recettes_repas_previsionnels | vide | stub | réel | ✅ terminé |
| 6 | dashboard | vide | stub | réel | ⚪ à faire |
| 7 | configuration (familles + profil) | squelette | partiel | partiel | ⚪ à faire |

Légende : ⚪ à faire · 🔵 en cours · ✅ terminé

## Où on s'est arrêté (curseur)

**Thème terminé : recettes_repas_previsionnels (#5) — brainstorm bouclé.**
- Résultats → **`functional/recipes-meals.html`** (décisions R1→R9, modèle 7 tables, pont liste de courses → Magic List).
- ⚠️ **Méthodo (retour user du 25/06)** : brainstorm = **use cases d'abord**, pas de questions techniques raccourcies (stocké vs dérivé, granularité…). Voir mémoire `brainstorm-use-case-first`.
- Décisions structurantes : **R1** recette+sélection user-owned + `shares` ; **R2** recette plate (ingredients+steps, pas de parts), steps avec image, image de présentation ; **R3** ingrédient structuré `{quantity?, unit?, name}` ; **R4** upload géré par le back, stockage abstrait ; **R5** CRUD complet recettes ; **R6** note partagée + favori par membre ; **R7** sélection de repas SANS période figée, placements `(recette→jour)`, un plat peut couvrir plusieurs jours ; **R8** « valider » → Magic List cochable, ingrédients regroupés (même unité=somme ; unités ≠ = ligne marquée « à vérifier »), lien vers recettes sources, retrait best-effort ; **R9** pas de `selectedForWeek`, dashboard montre les recettes posées sur les jours visibles.
- Modèle DB : `recipes`, `recipe_ingredients`, `recipe_steps`, `recipe_favorites`, `meal_selections`, `meal_selection_entries`, `shopping_list_links`. La liste de courses est une Magic List normale.
- **Reprendre à** : prochain thème — #6 **dashboard** (agrège agenda/calendrier + repas + comptes + magic_list) ou #7 **configuration**.

### Historique
- **#4 comptes_bancaire** ✅ → `functional/bank-account.html`. Décisions B1→B8. Compte user-owned, montants versionnés (budgets+charges), 2 dates (expenseDate→mois, debitDate→actual/forecast), CRUD complet, delete récurrent = arrêt daté.
- **#3 calendrier** ✅ → `functional/calendar.html`. Décisions C1→C7. Calendrier user-owned, all-day+endDate, récurrence simple, invité=membre, kind EVENT/TASK, agenda fenêtre j→j+7.
- **#2 transverse_partage** ✅ → `functional/sharing.html`. Partage **PAR USER**, table `shares` unique, « partager à une famille » = front. Décisions T1→T9.
- **#1 magic_list** ✅ → `functional/magic-list.html`. Amendé pour cohérence partage par user (D5 obsolète).
- **Cadre méthodo (important)** : en brainstorm on ne discute PAS d'archi logicielle (couches, modules) ; seule notion technique = le **modèle de base de données** ; erreurs = principe projet (codes web standards).
- ⚠️ La section « Accès & partage » de magic_list est supersédée par `sharing.html`.

## Journal des décisions transverses

> Décisions qui impactent plusieurs thèmes. À remplir au fil de l'eau.

- **Convention API** : JSON en **camelCase** ; enums en `SCREAMING_SNAKE`. Le back Rust s'aligne (front = référence).
- **Timestamps** : `createdAt` / `updatedAt` exposés dans toutes les vues.
- **Réponses de mutation** : renvoyer l'agrégat complet après mutation (ex: MagicList complète après mutation d'item).
- **Modèle de partage** : décisions détaillées (T1→T9) et spec complète déplacées dans **`functional/transverse_partage.md`**. Ce fichier-ci ne garde que le curseur.
