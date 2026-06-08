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
    pub owner_username: String,
    pub visibility: Visibility,
    pub family_id: Option<i32>,        // requis si Shared, None si Personal
    pub excluded_member_ids: Vec<i32>, // membres de la famille privés d'accès
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

pub enum Visibility { Shared, Personal }

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

**Convention JSON** : l'API expose du **camelCase** (`magicListType`, `dueDate`, `familyId`, `excludedMemberIds`, `itemCount`, `createdAt`, `updatedAt`) pour s'aligner sur le contrat front. Les enums sont sérialisés en `SCREAMING_SNAKE` (`SIMPLE`, `TASK`, `TEMPLATE`, `SHARED`, `PERSONAL`, `TODO`, `IN_PROGRESS`, `DONE`).

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
- un **type** (SIMPLE / TASK / TEMPLATE) ;
- la **visibilité** : partagée avec la famille (`SHARED`) ou personnelle (`PERSONAL`) ;
- si partagée : les **membres exclus** (par défaut toute la famille active y a accès).

Règles :
- `SHARED` ⇒ `familyId` requis. `PERSONAL` ⇒ `familyId` non envoyé / `None`.
- `excludedMemberIds` n'a de sens que si `SHARED`.

## Modification
- Renommer la liste.
- Changer la visibilité (`PERSONAL` ⇄ `SHARED`).
- Ajouter / retirer des membres exclus.

## Suppression
- Supprimer une liste supprime ses items (cascade).
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

Règle d'accès (lecture) :
- l'**owner** a toujours accès à ses listes (perso comme partagées) ;
- un non-owner a accès **uniquement si** la liste est `SHARED`, possède un `familyId`, que l'utilisateur est **membre de cette famille** **et qu'il n'est pas dans `excludedMemberIds`**.

> **Décision D5** : `excludedMemberIds` doit être **appliqué** dans le check d'accès (aujourd'hui stocké mais ignoré).

Droits d'écriture sur une liste partagée (décision D3 — **collaboratif, admin owner**) :
- tout membre ayant accès peut **ajouter / cocher / éditer / supprimer des ITEMS** et nettoyer les terminés ;
- seul l'**owner** peut **renommer, changer visibilité/exclusions, supprimer la LISTE**, ou la régénérer depuis un template.

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

1. **Listes perso & multi-familles** : une liste `PERSONAL` (`familyId = null`) appartient à l'utilisateur, pas à une famille. Décision : elle est **globale à l'utilisateur** et visible quel que soit le contexte famille actif. (À reconfirmer si l'UX impose un rattachement.)
2. **Passage SHARED → PERSONAL** : on conserve `familyId`/`excludedMemberIds` en base mais ils deviennent inopérants ; la liste sort de la vue famille des autres membres.
3. **Owner quitte la famille** : la liste partagée reste celle de l'owner ; définir si elle reste visible aux autres (proposition : non — l'accès dépend de l'appartenance courante).
4. **Membre exclu** : ne voit pas la liste dans le résumé, 403 sur accès direct.
5. **Réordonnancement manuel** : pas de colonne `position` aujourd'hui → drag&drop non supporté en phase 1 (tri par nom/statut/checked seulement). À ajouter (colonne `position`) si besoin ultérieur.
6. **`dueDate` & fuseau** : comparaison « en retard » sur la date locale du jour.
7. **Suppression de template** : ne casse pas les instances (pas de FK).

# Stratégie d'implémentation

**Référence de contrat** : front React + mock Fastify (camelCase, réponse = liste complète après chaque mutation d'item, `createdAt`/`updatedAt` partout). Le back Rust s'aligne.

## Phase 1 — Parité CRUD & accès (back Rust ↔ front)
Le back doit rattraper les routes déjà attendues par le front. Endpoints à ajouter :

| Méthode | Route | À faire |
|---|---|---|
| `GET` | `/families/{familyId}/magic-lists/{id}` | nouvelle : détail + items |
| `PUT` | `/families/{familyId}/magic-lists/{id}` | nouvelle : rename + visibilité + exclusions |
| `DELETE` | `/families/{familyId}/magic-lists/{id}` | nouvelle |
| `DELETE` | `/families/{familyId}/magic-lists/{id}/items/{itemId}` | nouvelle (réponse = liste complète) |
| `DELETE` | `/families/{familyId}/magic-lists/{id}/items/completed` | nouvelle (règle « terminé » unifiée) |

Travaux transverses au domaine :
- Sérialiser `createdAt` / `updatedAt` dans les vues (liste, summary, items).
- Convention **camelCase** sur tous les DTO d'entrée/sortie (`#[serde(rename_all = "camelCase")]`).
- Les mutations d'item renvoient la **MagicList complète** (alignement front).
- Brancher `excludedMemberIds` dans `check_magic_list_access`.
- Appliquer les droits D3 (collaboratif items / admin owner liste).

Par couche (selon l'archi du projet — controller → middleware → usecase → domain → repository) :
- **usecases** : `get_magic_list_by_id`, `update_magic_list`, `delete_magic_list`, `delete_item_from_magic_list`, `clear_completed_items`. Réutiliser `check_magic_list_access` (étendu aux exclusions et aux droits owner-vs-membre).
- **domain** : commande `UpdateMagicListCommand` (name, visibility, excluded_member_ids). Erreurs : une enum par use case dans `domain/errors.rs`.
- **repository (trait + sqlx)** : `find_by_id`, `update`, `delete`, `delete_item`, `delete_completed_items`, `find_items_by_list_id`.
- **http** : DTO requêtes/vues camelCase, nouvelles routes dans le controller, middlewares qui dépaquettent et délèguent.
- **MiddlewareError** : variantes `#[from]` pour les nouveaux use-case errors + mapping `status_code()` (403 accès, 404 introuvable).
- **mocks + tests** : mettre à jour le mock repo et tester chaque couche.

## Phase 2 — Templates
- Endpoint de génération : `POST /families/{familyId}/magic-lists/{templateId}/generate` (payload : `name`, `type` cible SIMPLE|TASK, `visibility`, `excludedMemberIds?`).
- Usecase `generate_list_from_template` : charge le template + ses items, crée une nouvelle liste indépendante, copie les items en réinitialisant `checked`/`status`.
- Aucune contrainte FK template→instance.
- Ajouter l'UI front (le mock et le front ne l'ont pas encore → le contrat est à créer, pas à aligner).

# Journal de décisions (révisables)

- **D1** — 3 types conservés ; `checked` et `status` coexistent sur TASK ; checkbox affichée pour TASK uniquement.
- **D2** — Contenu d'item **facultatif** (titre seul obligatoire). Corrige la spec initiale.
- **D3** — Liste partagée **collaborative sur les items**, **admin (rename/visibilité/suppression) réservé à l'owner**.
- **D4** — Templates traités en **Phase 2** (après la parité CRUD).
- **D5** — `excludedMemberIds` **appliqué** dans le contrôle d'accès.
- Règle **« terminé »** = `checked || status==DONE` (unifie nettoyage + regroupement).
- **Filtres/recherche/tri** côté **front** en phase 1.
