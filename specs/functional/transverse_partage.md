# Le partage (concept transverse)

> Statut : **brainstorm en cours**. Voir le journal de décisions (T1→T9) en fin de fichier.
>
> Ceci n'est pas un thème fonctionnel mais un **mécanisme central** : tout domaine partageable
> (magic_list, calendrier, comptes, recettes, repas…) s'appuie dessus. But : **un seul mécanisme
> de partage**, un **parcours utilisateur commun**, et un **monitoring unifié**.

# Principe de base

De base, **rien n'est partagé** : tout élément est **personnel** (appartient à son créateur).
On peut ensuite **partager** un élément avec **sa famille** (par défaut la famille active).

# Les 3 notions clés

### 1. Visibilité
- `PERSONAL` : l'élément n'appartient qu'à son owner. Personne d'autre ne le voit.
- `SHARED` : l'élément est rattaché à **une famille** et devient accessible à ses membres
  selon les droits ci-dessous.

> Techniquement, **`SHARED` ⇔ il existe une row de partage** pour l'élément (cf. modèle de
> données). Pas de partage = personnel.

### 2. Propriété (owner)
Chaque élément a un **owner** (son créateur). L'owner est **au-dessus des droits** :
lui seul peut **administrer** l'élément — le **renommer**, **régler son partage**
(famille + droits par membre) et le **supprimer**. L'administration n'est **jamais déléguée**
(même un membre en écriture ne peut pas administrer).

### 3. Droits d'accès — bitmask façon Linux
Les droits portent sur le **contenu** de l'élément (les items d'une liste, les events d'un
agenda, etc.). On reprend l'encodage des permissions Linux :

| Bit | Droit | Valeur |
|-----|-------|--------|
| `r` | lecture | **4** |
| `w` | écriture | **2** |
| `x` | (réservé, non utilisé) | 1 |

L'écriture **implique** la lecture. En itération 1, seules **3 valeurs** sont valides :

| Valeur | Sens | Notation |
|--------|------|----------|
| `0` | aucun accès | `---` |
| `4` | lecture seule | `r--` |
| `6` | lecture + écriture | `rw-` |

Test des droits : `acces & 4` ⇒ peut lire · `acces & 2` ⇒ peut écrire.

# Régler le partage d'un élément

Quand un owner partage un élément `SHARED`, il définit :
- la **famille cible** (`familyId`) ;
- un **niveau famille par défaut** (`defaultAccess` ∈ `{0, 4, 6}`, **défaut = `6`**) appliqué à
  **tous les membres** de la famille ;
- des **surcharges par membre** (`memberOverrides`) : pour un `userId` donné, un droit
  spécifique (`0` / `4` / `6`) qui **remplace** le défaut famille.

Ainsi `0` en surcharge = **membre exclu** (l'ancien `excludedMemberIds` de magic_list devient
un simple override à `0`).

# Comment l'accès est résolu

Accès effectif d'un utilisateur **U** sur une ressource **R** :

1. si **U == owner(R)** → **plein accès** (`rw`) **+ administration** ;
2. sinon, si R n'a **pas de partage** (personnel) → **aucun accès** ;
3. sinon, si U **n'est pas membre courant** de la famille cible → **aucun accès** ;
4. sinon → la **surcharge** de U si elle existe, **sinon** le `defaultAccess` de la famille.

L'administration (rename / partage / suppression) reste **owner-only**, indépendamment du bitmask.

**Multi-familles.** Un utilisateur peut appartenir à plusieurs familles (une seule « active »).
L'accès à un élément partagé dépend de l'**appartenance à la famille cible du partage**, **jamais**
de la famille active (qui n'est qu'un contexte d'affichage par défaut). Un élément personnel reste
global à son owner.

# Famille étendue

Les grands-parents, oncles, tantes (cf. personas) **ne nécessitent aucun mécanisme à part** :
ce sont des **membres de la famille** avec leur `FamilyRelation`
(`GRAND_PARENT` / `UNCLE` / `AUNT` / `OTHER`). Pour « organiser les vacances avec seulement
certains », l'owner met `defaultAccess = 0` et un `override = 6` aux membres concernés (ou
l'inverse). Le partage hors-noyau n'est qu'un jeu de surcharges.

> Hors-scope itération 1 : partage inter-familles, ou vers des personnes hors de toute famille.

# Modèle de données

Un **registre central et générique** (polymorphe), dans `common/` :

```sql
CREATE TABLE shares (
    id             SERIAL PRIMARY KEY,
    resource_type  VARCHAR(40) NOT NULL,   -- 'MAGIC_LIST', 'CALENDAR_EVENT', 'BANK_ACCOUNT', ...
    resource_id    INTEGER     NOT NULL,   -- id de l'élément dans sa table métier (PAS de FK : polymorphe)
    family_id      INTEGER     NOT NULL REFERENCES families(id) ON DELETE CASCADE,
    default_access SMALLINT    NOT NULL DEFAULT 6,   -- bitmask {0,4,6}
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (resource_type, resource_id)             -- au plus 1 partage par élément
);

CREATE TABLE share_overrides (
    share_id  INTEGER  NOT NULL REFERENCES shares(id) ON DELETE CASCADE,
    user_id   INTEGER  NOT NULL REFERENCES users(id)  ON DELETE CASCADE,
    access    SMALLINT NOT NULL,             -- bitmask {0,4,6}
    PRIMARY KEY (share_id, user_id)
);
```

Conséquences :
- **Une row `shares` n'existe que si l'élément est partagé.** Personnel = aucune row.
- L'élément métier **garde son `owner_id`** ; le registre ne stocke **que** la config de partage
  (ni la propriété, ni le contenu).
- `resource_id` **n'a pas de FK** (registre polymorphe) ⇒ supprimer un élément métier doit
  **supprimer son partage explicitement** dans le usecase du domaine (pas de cascade DB).
- Les `userId` référencent `users.id` (id **global**, pas `family_members.id`).

# Architecture (intégration hexagonale)

Module transverse `domains/common/sharing/`, structuré comme un domaine :

```
domains/common/sharing/
├── domain/
│   ├── share.rs              # Share { id, resource_type, resource_id, family_id, default_access, overrides }
│   ├── resource_type.rs      # enum ResourceType { MagicList, CalendarEvent, BankAccount, Recipe, MealPlan, … }
│   ├── access.rs             # value object Access(u8) : NONE=0, READ=4, WRITE=6 ; can_read()/can_write()
│   ├── member_access.rs      # { user_id, access }
│   ├── share_repository.rs   # trait SharingRepository : find_by_resource, upsert, delete_by_resource
│   └── errors.rs             # une enum par use case
├── usecases/
│   ├── resolve_access.rs     # applique les règles ci-dessus → Access effectif
│   └── update_share.rs       # owner-only : pose / modifie / retire le partage d'une ressource
└── repositories/
    └── sqlx_share_repository.rs
```

Comment un domaine s'en sert :
- son `check_<x>_access` **délègue** à `resolve_access(ResourceType::X, resource_id, user, owner_id)`
  au lieu de réimplémenter la logique ;
- le usecase métier exploite l'`Access` retourné : pas de lecture → 404/403, pas d'écriture → 403,
  action d'admin (rename/delete/partage) → exige `user == owner` ;
- la **suppression** d'un élément appelle `delete_by_resource(...)` ;
- `resolve_access` s'appuie sur le `FamilyRepository` existant pour l'appartenance famille
  (seule dépendance `sharing → family`, **à acter dans un ADR** car déroge à l'indépendance des domaines) ;
- `ActixState<DB>` reçoit un `share_repository` injecté comme les autres.

# API

Le registre est un **détail d'implémentation serveur** ; l'API reste **orientée ressource**
(aligne le front, contrat de référence) :
- chaque ressource renvoie son **bloc partage** : `visibility`, `familyId`, `defaultAccess`,
  `memberOverrides` (`[{ userId, access }]`), et l'`access` effectif de l'appelant ;
- la **mutation** du partage passe par un endpoint de la ressource
  (ex. `PUT /families/{familyId}/magic-lists/{id}/share`, ou intégré au `PUT` de la ressource) ;
- pas d'endpoint transverse de monitoring en itération 1 (le registre central le rendra trivial
  plus tard — hors-scope).

Convention : JSON **camelCase**, valeurs d'`access` en **entiers** (`0` / `4` / `6`).

# Exemple concret — magic_list

magic_list est le premier consommateur et sert d'exemple de migration.

**Avant** (modèle propre à magic_list) : colonnes `visibility`, `family_id`,
`excluded_user_ids INTEGER[]` sur la table `magic_list`, et un `check_magic_list_access`
maison.

**Après** (modèle transverse) :
- on **retire** `visibility`, `family_id`, `excluded_user_ids` de `magic_list` (on garde `owner_id`) ;
- partage d'une liste = une row `shares(resource_type='MAGIC_LIST', resource_id=<listId>, family_id, default_access)`
  + des `share_overrides` ;
- `check_magic_list_access` délègue à `resolve_access(ResourceType::MagicList, listId, user, owner_id)` ;
- droits : tout membre avec `w` peut ajouter/cocher/éditer/supprimer des **items** et nettoyer les
  terminés ; seul l'owner renomme / règle le partage / supprime la liste (inchangé vs décision D3 de
  magic_list, désormais exprimé via le bitmask).

**Migration des données existantes :**
- pour chaque liste `SHARED` : créer la row `shares` avec `default_access = 6` (l'ancien partage
  était collaboratif) ;
- chaque `excluded_user_id` → un `share_override(user_id, access = 0)` ;
- puis **drop** des colonnes devenues inutiles.

> ⚠️ La section « Accès & partage » de `magic_list.md` est **supersédée** par ce document et sera
> amendée une fois ce modèle figé.

# Edge cases

1. **Sortie de famille / owner qui quitte.** L'accès dépend de l'appartenance **courante** : un
   membre qui quitte la famille perd l'accès aux éléments partagés. La row `shares` reste celle de
   l'owner. *(à approfondir : que devient un élément partagé si l'owner quitte la famille cible ?)*
2. **Suppression d'une famille.** `ON DELETE CASCADE` sur `shares.family_id` retire les partages ;
   les éléments redeviennent personnels (plus de row). *(à confirmer : comportement voulu ?)*
3. **Override d'un non-membre.** Un `share_override` pour un `userId` qui n'est pas (ou plus) membre
   de la famille est **inopérant** (la règle 3 de résolution coupe avant). On peut le laisser en base
   sans effet, ou le nettoyer.
4. **Bascule `SHARED → PERSONAL`.** = suppression de la row `shares` (et ses overrides en cascade).
5. **Valeurs de bitmask hors {0,4,6}.** Rejet à l'écriture (validation) en itération 1.

# Reste à traiter (brainstorm)

- Section archi : valider le module `common/sharing/` et la dépendance `sharing → family` (ADR).
- Approfondir edge cases 1 et 2 (owner quitte / suppression famille).
- Mapping d'erreurs précis (`status_code`).
- Stratégie d'implémentation détaillée + ADR.

# Journal de décisions (révisables)

- **T1 — Granularité** : partage **par élément** (généralisation de magic_list), pas d'« espaces ».
- **T2 — Droits = bitmask Linux** : `r=4`, `w=2`, `x=1` (réservé). `w` implique `r` ⇒ valeurs valides
  `{0, 4, 6}`.
- **T3 — Administration owner-only** : rename / partage / suppression réservés à l'owner ; l'écriture
  ne porte que sur le contenu.
- **T4 — Clé membre = `user_id` global** (exposé `userId`).
- **T5 — Registre central générique** dans `common/` (tables `shares` + `share_overrides`), **row
  uniquement si partagé** ; l'élément garde `owner_id`. Déroge à l'indépendance des domaines → ADR.
- **T6 — Résolution d'accès** : owner → plein + admin ; sinon row de partage ; membre courant requis ;
  override sinon `defaultAccess`. Multi-familles : famille **cible**, pas famille active.
- **T7 — API embarquée par ressource**, registre interne ; pas d'endpoint monitoring en itération 1.
- **T8 — Famille étendue = simples membres** (relation GRAND_PARENT/UNCLE/AUNT), pas de mécanisme séparé.
- **T9 — `defaultAccess` par défaut = `6`** (collaboratif) à la création d'un partage.
