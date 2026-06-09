# Magic List

> Statut spec : **enrichie (brainstorm v1)**. Voir le journal de décisions en fin de fichier.

# Contexte

On utilise des listes tous les jours : liste de courses, liste d’affaires pour les vacances, liste de tâches, liste de jouets, etc.

On les utilise dans différents contextes : prévoir les repas de la semaine, préparer les vacances, le travail à finir, etc.

On les utilise pour des timings différents : pour aujourd’hui, pour les courses demain, pour les vacances dans 2 semaines, etc.

Parfois on a besoin de listes qu’on réutilise plusieurs fois, comme la liste des choses à prendre pour les vacances.

Les Magic Lists de Navigator répondent simplement à tous ces besoins : des listes **réplicables, partageables, checkables**. Elles doivent être faciles d’accès et intuitives.

# Modèle de données

```rust
pub struct MagicList {
    pub id: i32,
    pub name: String,
    pub list_type: MagicListType,
    pub owner_username: String,        // owner : seul à administrer la liste
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

// La visibilité, le partage et les droits NE sont PAS portés par magic_list :
// ils relèvent du mécanisme transverse (cf. specs/functional/transverse_partage.md).
// La table magic_list ne garde donc que owner_id ; aucune colonne family_id /
// visibility / excluded_user_ids.

pub enum MagicListType { Simple, Task, Template }

pub struct MagicListItem {
    pub id: i32,
    pub magic_list_id: i32,
    pub title: String,
    pub content: Option<String>,        // FACULTATIF (cf. décision D2)
    pub checked: bool,
    pub due_date: Option<NaiveDate>,
    pub status: Option<MagicListItemStatus>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

pub enum MagicListItemStatus { Todo, InProgress, Done }
```

**Convention JSON** : l'API expose du **camelCase** (`magicListType`, `dueDate`, `itemCount`, `createdAt`, `updatedAt`) pour s'aligner sur le contrat front. Les enums sont sérialisés en `SCREAMING_SNAKE` (`SIMPLE`, `TASK`, `TEMPLATE`, `TODO`, `IN_PROGRESS`, `DONE`).

Les champs de **partage** (`visibility` dérivée `PERSONAL`/`SHARED`, `sharedWith` = `[{ userId, access }]`, `access` effectif de l'appelant) sont **ajoutés à la vue par la couche de partage transverse** (cf. `transverse_partage.md`), pas par magic_list elle-même.

# Les types de liste et le modèle de complétion

Deux mécanismes de « fait » cohabitent — `checked` (booléen, modèle liste de courses) et `status` (TODO / IN_PROGRESS / DONE, modèle tableau de tâches). Le type de la liste décide de l'UI présentée :

| Type | Usage | `checked` | `status` | UI détail |
|------|-------|-----------|----------|-----------|
| **SIMPLE** | Notes / liste de référence à usage unique | non utilisé | non utilisé | items à plat, pas de coche |
| **TASK** | Liste actionnable (courses cochables et/ou todos à statut) | oui (coche) | optionnel | coche + menu statut, regroupement par statut |
| **TEMPLATE** | Modèle réplicable (cf. section Templates) | n/a (modèle) | n/a | édition du modèle |

> **Décision D1** : on garde 3 types. `checked` et `status` peuvent coexister sur une liste TASK. La checkbox n'est affichée que pour TASK ; le menu statut est proposé sur les items de TASK.

# Gestion d'une Magic List

## Création
En tant qu'utilisateur, je crée une magic list en spécifiant :
- un **nom** ;
- un **type** (SIMPLE / TASK / TEMPLATE).

À la création, la liste est **personnelle** (aucun partage). Le partage est une **action distincte**
gérée par le mécanisme transverse (cf. `transverse_partage.md`) : l'owner accorde des **grants** par
utilisateur ; « partager à la famille » est une commodité du front (un grant par membre).

## Modification
- Renommer la liste (owner uniquement).
- **Gérer le partage** (ajouter/retirer/modifier les grants) : action **owner-only** qui passe par
  l'endpoint de partage transverse, pas par l'édition de la liste.

## Suppression
- Supprimer une liste supprime ses items (cascade) **et ses grants de partage** (suppression explicite,
  registre polymorphe — cf. `transverse_partage.md`).
- **Supprimer un TEMPLATE ne supprime PAS les listes générées à partir de lui** (aucun lien dur / FK cascade ; cf. Templates).

# Items d'une liste

Un item contient un **titre obligatoire** et un **contenu facultatif** (décision D2). Champs optionnels supplémentaires : `dueDate`, `status`, `checked` (pour TASK).

Features :
- Ajouter un item.
- Modifier un item (titre, contenu, dueDate, status, checked).
- Supprimer un item.
- **Nettoyer les items « terminés »** : supprime en masse les items considérés terminés.

> **Règle « terminé » unifiée** : un item est terminé si `checked == true` **ou** `status == DONE`. (Aligne le bouton « Nettoyer » et le regroupement « Terminé » du front, aujourd'hui incohérents : le mock filtre sur `status===DONE` seul.)

# Templates (feature phare — Phase 2)

Un template est une liste de type TEMPLATE servant de modèle réutilisable.

- Depuis un template, **générer une instance** de type SIMPLE ou TASK, avec un **nom** propre et une **visibilité** propre.
- La génération **copie les items** du template ; sur l'instance, `checked` est remis à `false` et `status` est réinitialisé (ou retiré).
- L'instance est **indépendante** : pas de lien persistant vers le template. Modifier/supprimer le template n'affecte pas les instances déjà générées.

# Accès & partage

> Le **mécanisme** d'accès/partage est **transverse** (cf. `transverse_partage.md`) — partage par
> user, droits bitmask (`4` lecture / `6` écriture), résolution `owner → grant → rien`. Cette section
> n'en décrit que l'**application à magic_list**.

- l'**owner** a toujours plein accès à ses listes **et** est le seul à les **administrer** ;
- un non-owner accède à une liste **uniquement s'il a un grant** dessus ; son droit (`4` ou `6`) est
  résolu par la couche transverse ;
- **écriture (`6`)** sur une liste = **collaborer sur les ITEMS** : ajouter / cocher / éditer /
  supprimer des items, nettoyer les terminés ;
- **lecture seule (`4`)** = voir la liste et ses items, sans modification ;
- **owner uniquement** : renommer, gérer le partage, supprimer la liste, régénérer depuis un template.

# Visualisation

## Page d'accueil des listes (résumé)
- Vue cartes : nom, badge de visibilité, type (icône + libellé), **nombre d'items**.
- **Filtres** : par type, par visibilité, **recherche par nom**.
- Action de suppression inline (owner).

> Filtres/recherche **côté front** dans un premier temps (volumétrie faible). Possibilité d'ajouter des query params backend plus tard si besoin.

## Page détail d'une liste
- Items adaptés au type (cf. tableau des types).
- Pour TASK : **regroupement par statut** par défaut — « À faire » / « En cours » / « Terminé ».
- Tri possible par nom / statut / checked.
- `dueDate` affichée (JJ/MM/AAAA), **surlignée en retard** si dépassée et item non terminé.

# Edge cases recensés

1. **Listes perso & multi-familles** : une liste sans grant est **globale à l'owner**, visible quel que soit le contexte famille actif (le partage est par user, pas rattaché à une famille — cf. `transverse_partage.md`).
2. **Passage SHARED → PERSONAL** : = **retirer tous les grants** ; la liste disparaît des vues des autres.
3. **Owner / membre quitte la famille** : sans check d'appartenance, les grants existants **restent valides** tant que l'owner ne les révoque pas (cf. edge cases transverses).
4. **Membre sans grant** : ne voit pas la liste dans le résumé ; **404** sur accès direct (on ne révèle pas l'existence).
5. **Réordonnancement manuel** : pas de colonne `position` aujourd'hui → drag&drop non supporté en phase 1 (tri par nom/statut/checked seulement). À ajouter (colonne `position`) si besoin ultérieur.
6. **`dueDate` & fuseau** : comparaison « en retard » sur la date locale du jour.
7. **Suppression de template** : ne casse pas les instances (pas de FK).

# Stratégie d'implémentation

**Référence de contrat** : front React + mock Fastify (camelCase, réponse = liste complète après chaque mutation d'item, `createdAt`/`updatedAt` partout). Le back Rust s'aligne.

## Phase 1 — Parité CRUD & accès (back Rust ↔ front)
Le back doit rattraper les routes déjà attendues par le front. Endpoints à ajouter :

| Méthode | Route | À faire |
|---|---|---|
| `GET` | `/families/{familyId}/magic-lists/{id}` | nouvelle : détail + items (+ bloc partage de la couche transverse) |
| `PUT` | `/families/{familyId}/magic-lists/{id}` | nouvelle : **rename** (owner) — plus de visibilité/exclusions ici |
| `PUT` | `/families/{familyId}/magic-lists/{id}/shares` | nouvelle : **gérer les grants** (owner-only) — délègue au partage transverse |
| `DELETE` | `/families/{familyId}/magic-lists/{id}` | nouvelle (supprime items + grants) |
| `DELETE` | `/families/{familyId}/magic-lists/{id}/items/{itemId}` | nouvelle (réponse = liste complète) |
| `DELETE` | `/families/{familyId}/magic-lists/{id}/items/completed` | nouvelle (règle « terminé » unifiée) |

> ⚠️ Le préfixe `/families/{familyId}/` provient du contrat front initial mais **ne colle plus** au
> modèle (le partage est par user, pas scopé famille). À reconfirmer avec le front : soit on le garde
> comme simple contexte d'affichage, soit on bascule sur `/magic-lists/{id}`. Hors périmètre brainstorm.

Travaux transverses au domaine :
- Sérialiser `createdAt` / `updatedAt` dans les vues (liste, summary, items).
- Convention **camelCase** sur tous les DTO d'entrée/sortie (`#[serde(rename_all = "camelCase")]`).
- Les mutations d'item renvoient la **MagicList complète** (alignement front).
- Brancher le **partage transverse** : `check_magic_list_access` délègue à la résolution d'accès
  transverse ; ajout du bloc partage (`visibility`/`sharedWith`/`access`) dans les vues.
- Appliquer les droits (écriture `6` ⇒ items ; owner ⇒ admin de la liste — cf. section Accès & partage).

Par couche (controller → middleware → usecase → domain → repository) :
- **usecases** : `get_magic_list_by_id`, `update_magic_list` (rename), `delete_magic_list` (+ purge des grants), `delete_item_from_magic_list`, `clear_completed_items`. `check_magic_list_access` s'appuie sur la résolution d'accès transverse.
- **domain** : commande `UpdateMagicListCommand` (**name** uniquement). Erreurs : une enum par use case dans `domain/errors.rs`.
- **repository (trait + sqlx)** : `find_by_id`, `update`, `delete`, `delete_item`, `delete_completed_items`, `find_items_by_list_id`.
- **partage** : la gestion des grants et la résolution d'accès viennent du mécanisme transverse (cf. `transverse_partage.md`) — pas réimplémentées dans magic_list.
- **http** : DTO requêtes/vues camelCase, nouvelles routes dans le controller, middlewares qui dépaquettent et délèguent.
- **MiddlewareError** : variantes `#[from]` pour les nouveaux use-case errors + mapping `status_code()` (404 lecture refusée, 403 écriture/owner).
- **mocks + tests** : mettre à jour le mock repo et tester chaque couche.

## Phase 2 — Templates
- Endpoint de génération : `POST /families/{familyId}/magic-lists/{templateId}/generate` (payload : `name`, `type` cible SIMPLE|TASK). L'instance générée est **personnelle** ; le partage se fait ensuite via l'endpoint de partage (transverse).
- Usecase `generate_list_from_template` : charge le template + ses items, crée une nouvelle liste indépendante, copie les items en réinitialisant `checked`/`status`.
- Aucune contrainte FK template→instance.
- Ajouter l'UI front (le mock et le front ne l'ont pas encore → le contrat est à créer, pas à aligner).

# Journal de décisions (révisables)

- **D1** — 3 types conservés ; `checked` et `status` coexistent sur TASK ; checkbox affichée pour TASK uniquement.
- **D2** — Contenu d'item **facultatif** (titre seul obligatoire). Corrige la spec initiale.
- **D3** — Liste partagée : **écriture (`6`) = collaboratif sur les items**, **admin (rename/partage/suppression) réservé à l'owner**. *(Exprimé désormais via le bitmask transverse — cf. `transverse_partage.md`.)*
- **D4** — Templates traités en **Phase 2** (après la parité CRUD).
- ~~**D5** — `excludedMemberIds` appliqué dans le contrôle d'accès.~~ **Obsolète** : `excludedMemberIds`/`familyId`/`visibility` n'existent plus sur magic_list ; le partage est **par user** via le mécanisme transverse (`transverse_partage.md`). Exclure = ne pas créer de grant.
- Règle **« terminé »** = `checked || status==DONE` (unifie nettoyage + regroupement).
- **Filtres/recherche/tri** côté **front** en phase 1.
