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
| 6 | dashboard | écrite | stub | réel | ✅ terminé |
| 7 | configuration (familles + profil) | écrite | partiel | partiel | ✅ terminé |

Légende : ⚪ à faire · 🔵 en cours · ✅ terminé

## Où on s'est arrêté (curseur)

**🎉 Brainstorm COMPLET — les 7 thèmes sont terminés.**
- Dernier thème bouclé : **configuration (#7)** → **`functional/configuration.html`** (décisions CF1→CF11).
- Gouvernance famille : **créateur = audit seul** (aucun pouvoir permanent) ; **rôle admin = source de tous les pouvoirs**, « tous révocables » avec **invariant ≥ 1 admin** ; inviter/retirer = admins, quitter = tout le monde, dernier admin doit promouvoir avant de partir.
- Invitations : email/username d'un **user existant**, **in-app** (accepter/décliner), pas de doublon, pas d'expiration. Multi-famille + création illimitée.
- Fin de vie : **soft delete** → réactivation par **n'importe quel admin** dans X jours → sinon **hard delete**. Profil minimal (nom affiché + avatar ; email/username read-only Keycloak). **Frontière : Keycloak = authentification seule, autorisation = base Navigator.**
- ⚠️ **Points à réconcilier** (notés dans la spec) : périmètre du hard-delete (ressources user-scoped vs family-scoped, cf. `sharing.html`), valeur de X, suppression de compte vs invariant admin.
- **Prochaine étape** : le brainstorm est fini — passer à la **rédaction des tickets** (par vertical) ou à l'implémentation. Plus de thème à brainstormer.

### Historique
- **#7 configuration** ✅ → `functional/configuration.html`. Décisions CF1→CF11, 3 tables (`families`, `family_members`, `family_invitations`) + colonnes profil sur `users`. Créateur audit ; rôle admin collectif révocable (invariant ≥ 1) ; invitation in-app par email/username d'un user existant ; soft delete + réactivation admin + hard delete après X jours ; Keycloak = authN seul.
- **#5 recettes_repas_previsionnels** ✅ → `functional/recipes-meals.html`. Décisions R1→R9, modèle 7 tables. Recette plate ; sélection de repas SANS période figée (placements recette→jour) ; « valider » → Magic List cochable (agrégation ingrédients). ⚠️ Méthodo user du 25/06 : use cases d'abord (mémoire `brainstorm-use-case-first`).
- **#4 comptes_bancaire** ✅ → `functional/bank-account.html`. Décisions B1→B9. Compte user-owned ; changer un montant = **clôture + recréation** (date de fin au dernier jour du mois précédent, PAS de versionnement) ; **dépense de budget = Expense avec `budgetId`** (pas d'entité séparée) ; 2 dates (expenseDate→mois, debitDate→actual/forecast) ; dates inclusives ; CRUD complet.
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
