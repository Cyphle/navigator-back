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
| 5 | recettes_repas_previsionnels | vide | stub | réel | ⚪ à faire |
| 6 | dashboard | vide | stub | réel | ⚪ à faire |
| 7 | configuration (familles + profil) | squelette | partiel | partiel | ⚪ à faire |

Légende : ⚪ à faire · 🔵 en cours · ✅ terminé

## Où on s'est arrêté (curseur)

**Thème terminé : comptes_bancaire (#4) — brainstorm bouclé.**
- Résultats → **`functional/bank-account.html`** (décisions B1→B8, modèle de données 8 tables, montants versionnés, formules de calcul mensuel).
- Décisions structurantes : **B1** compte user-owned + `shares` (retrait visibility/familyId, suppression enum Visibility) ; **B2** actual = débité (debitDate≤today), remaining/forecast = pire cas fin de mois (budgets réservés à 100%) ; **B3** budget périodicité MONTHLY(mois civil)/YEARLY(année civile)/ONE_SHOT + **montant versionné** (modif = sa période + suivantes, jamais passé) ; **B4** charges = même versioning ; **B5** crédits ponctuels seulement ; **B6** CRUD complet ; **B7** rattachement au mois par `expenseDate`, split actual/forecast par `debitDate` ; **B8** delete d'un récurrent = arrêt à partir d'une date (passé conservé).
- Modèle DB : `bank_accounts`, `charges`+`charge_amounts`, `budgets`+`budget_amounts`, `budget_expenses`, `credits`, `expenses` ; accès délégué à `shares`. Charge = récurrente seulement (ponctuel = Expense). `endOfMonthForecast`=`remainingAmount`.
- **Reprendre à** : prochain thème — #5 **recettes_repas_previsionnels** ou #6 **dashboard** (les 2 points ouverts du dashboard dépendent de #6).

### Historique
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
