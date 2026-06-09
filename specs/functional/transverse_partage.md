# Le partage (concept transverse)

> Statut : **brainstorm en cours**. Voir le journal de décisions (T1→T9) en fin de fichier.
>
> Ceci n'est pas un thème fonctionnel mais un **mécanisme central** : tout domaine partageable
> (magic_list, calendrier, comptes, recettes, repas…) s'appuie dessus. But : **un seul mécanisme
> de partage**, un **parcours utilisateur commun**, et un **monitoring unifié**.

# Principe de base

De base, **rien n'est partagé** : tout élément est **personnel** (appartient à son créateur).
On peut ensuite **partager** un élément avec **d'autres utilisateurs**.

**Le partage se fait par utilisateur.** Le back ne raisonne qu'en **grants individuels**
`(ressource → utilisateur → droit)`. Il **ne connaît pas la notion de « partager à une famille »** :
c'est une **commodité du front** (cf. plus bas).

# Les 3 notions clés

### 1. Visibilité (dérivée)
- `PERSONAL` : aucun grant → seul l'owner y a accès.
- `SHARED` : au moins un grant existe.

La visibilité n'est **pas** une colonne : elle se **déduit** de l'existence de grants.

### 2. Propriété (owner)
Chaque élément a un **owner** (son créateur). L'owner est **au-dessus des droits** :
lui seul peut **administrer** l'élément — le **renommer**, **gérer ses partages** (ajouter/retirer
des grants) et le **supprimer**. L'administration n'est **jamais déléguée** (même un utilisateur en
écriture ne peut pas administrer).

### 3. Droits d'accès — bitmask façon Linux
Les droits portent sur le **contenu** de l'élément (les items d'une liste, les events d'un
agenda, etc.). On reprend l'encodage des permissions Linux :

| Bit | Droit | Valeur |
|-----|-------|--------|
| `r` | lecture | **4** |
| `w` | écriture | **2** |
| `x` | (réservé, non utilisé) | 1 |

L'écriture **implique** la lecture. En itération 1, un grant porte une de ces **2 valeurs** :

| Valeur | Sens | Notation |
|--------|------|----------|
| `4` | lecture seule | `r--` |
| `6` | lecture + écriture | `rw-` |

Pas de grant = **aucun accès** (équivalent `0` / `---`). Test des droits : `acces & 4` ⇒ peut lire ·
`acces & 2` ⇒ peut écrire.

# Partager un élément

L'owner partage un élément en créant des **grants** : pour chaque `userId`, un droit (`4` ou `6`).
- Ajouter un grant = donner l'accès à un utilisateur.
- Modifier un grant = changer son droit (`4` ⇄ `6`).
- Retirer un grant = couper l'accès (l'élément redevient personnel s'il n'en reste aucun).

# Comment l'accès est résolu

Accès effectif d'un utilisateur **U** sur une ressource **R** :

1. si **U == owner(R)** → **plein accès** (`rw`) **+ administration** ;
2. sinon, s'il existe un **grant** pour `(R, U)` → le droit de ce grant (`4` ou `6`) ;
3. sinon → **aucun accès**.

Pas de vérification d'appartenance à une famille : le grant individuel **est** la source de vérité.
L'administration (rename / partages / suppression) reste **owner-only**, indépendamment du bitmask.

# « Partager à la famille » = commodité du front

Côté back, **aucune notion de famille dans le partage**. Côté front :
- « Partager à ma famille » = le front **itère les membres** de la famille et crée **un grant par
  user** (droit choisi, ex. `6`). Exclure quelqu'un = **ne pas créer son grant**.
- L'**affichage** peut **regrouper** les grants par famille (« partagé avec la famille Dupont »),
  c'est purement présentation.

**Conséquences assumées :**
- **Snapshot, pas dynamique** : un membre qui rejoint la famille *après* le partage n'a **pas**
  d'accès automatique. Le front peut reproposer un « re-partager à la famille ».
- **Quitter la famille ne retire pas l'accès** : sans check d'appartenance, un grant reste valide
  tant que l'owner ne le révoque pas. (Le front peut proposer une action de nettoyage.)

> Ce choix **simplifie radicalement le back** : pas de `family_id`, pas de niveau par défaut, pas
> d'overrides, pas de dépendance `sharing → family`.

# Modèle de données

Un **registre central et générique** (polymorphe), dans `common/`, **une seule table** :

```sql
CREATE TABLE shares (
    id             SERIAL PRIMARY KEY,
    resource_type  VARCHAR(40) NOT NULL,   -- 'MAGIC_LIST', 'CALENDAR_EVENT', 'BANK_ACCOUNT', ...
    resource_id    INTEGER     NOT NULL,   -- id de l'élément dans sa table métier (PAS de FK : polymorphe)
    user_id        INTEGER     NOT NULL REFERENCES users(id) ON DELETE CASCADE,  -- bénéficiaire du grant
    access         SMALLINT    NOT NULL,   -- bitmask {4, 6}
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (resource_type, resource_id, user_id)   -- au plus 1 grant par (ressource, user)
);
```

Conséquences :
- **Un grant n'existe que si l'élément est partagé à cet utilisateur.** Personnel = aucune row.
- L'élément métier **garde son `owner_id`** ; le registre ne stocke **que** les grants
  (ni la propriété, ni le contenu).
- `resource_id` **n'a pas de FK** (registre polymorphe) ⇒ supprimer un élément métier doit
  **supprimer ses grants explicitement** (pas de cascade DB possible sur une clé polymorphe).
- Les `user_id` référencent `users.id` (id **global**).

> L'**architecture logicielle** (où vit ce module, découpage en couches, traits, injection) n'est
> **pas** traitée ici : elle suit les **règles d'architecture clean/hexagonale** du projet.

# Erreurs & codes HTTP

Le partage suit le **principe d'erreur commun à tout le projet** (cf. CLAUDE.md), sans rien
réinventer : chaque couche peut lever une erreur, **encapsulée et re-mappée à chaque couche
traversée** (hexagonal), jusqu'au contrôleur, en conservant la **chaîne de causes** via `#[source]`.

- une erreur SQL → `RepositoryError` (`NotFound` / `Conflict` / `Technical(#[source] …)`) ;
- wrappée par l'erreur **du use case** (variantes portant le contexte : `resource_id`, `user_id`) ;
- convertie en `MiddlewareError` (`#[from]`) puis en réponse HTTP.

Codes HTTP selon les **standards web** :

| Situation | Code |
|-----------|------|
| Non authentifié | **401** |
| Accès en lecture refusé (ni owner, ni grant) | **404** (on ne révèle pas l'existence) |
| Action refusée alors qu'on a la lecture (écriture sans `w`, ou admin non-owner) | **403** |
| Conflit (ex. grant déjà présent) | **409** |
| Erreur technique | **500** (corps neutralisé, chaîne loggée) |

# API

Le registre est un **détail d'implémentation serveur** ; l'API reste **orientée ressource**
(aligne le front, contrat de référence) :
- chaque ressource renvoie ses partages : `sharedWith` = `[{ userId, access }]`, l'`access`
  effectif de l'appelant, et une **`visibility` dérivée** `PERSONAL`/`SHARED` (calculée back :
  `SHARED` ⇔ `sharedWith` non vide) pour coller au badge de visibilité déjà affiché par le front ;
- la **mutation** des partages passe par un endpoint de la ressource
  (ex. `PUT /magic-lists/{id}/shares` avec la liste de grants voulue), exécuté **par l'owner** ;
- « partager à la famille » s'exprime côté front comme un `sharedWith` peuplé des `userId` des
  membres ; le back ne voit que des grants individuels ;
- pas d'endpoint transverse de monitoring en itération 1 (le registre central le rendra trivial
  plus tard — hors-scope).

Convention : JSON **camelCase**, valeurs d'`access` en **entiers** (`4` / `6`).

# Exemple concret — magic_list

magic_list est le premier consommateur et sert d'exemple de migration.

**Avant** (modèle propre à magic_list) : colonnes `visibility`, `family_id`,
`excluded_user_ids INTEGER[]` sur la table `magic_list`, et un `check_magic_list_access` maison
basé sur l'appartenance famille.

**Après** (modèle transverse par user) :
- on **retire** `visibility`, `family_id`, `excluded_user_ids` de `magic_list` (on garde `owner_id`) ;
- partager une liste = des rows `shares(resource_type='MAGIC_LIST', resource_id=<listId>, user_id, access)` ;
- `check_magic_list_access` délègue à `resolve_access(ResourceType::MagicList, listId, user, owner_id)` ;
- droits : tout user avec `w` peut ajouter/cocher/éditer/supprimer des **items** et nettoyer les
  terminés ; seul l'owner renomme / gère les partages / supprime la liste.

**Migration des données existantes :**
- pour chaque liste `SHARED` : créer un grant `access = 6` (l'ancien partage était collaboratif)
  **pour chaque membre de la famille cible** sauf ceux présents dans `excluded_user_ids` ;
- puis **drop** des colonnes `visibility`, `family_id`, `excluded_user_ids`.

> La section « Accès & partage » de `magic_list.md` a été **amendée** pour suivre ce modèle (par user,
> bitmask, owner-only admin).

# Edge cases

1. **Membre rejoint la famille après coup** : pas d'accès automatique (snapshot). Front propose
   éventuellement un « re-partager ».
2. **Membre quitte la famille** : son grant reste valide tant que l'owner ne le révoque pas
   (pas de check d'appartenance). Front peut proposer un nettoyage.
3. **Suppression d'un utilisateur** : `ON DELETE CASCADE` sur `shares.user_id` retire ses grants.
4. **Bascule `SHARED → PERSONAL`** : = retirer tous les grants.
5. **Valeurs de bitmask hors {4,6}** : rejet à l'écriture (validation) en itération 1.
6. **Grant sur soi-même / sur l'owner** : ignoré / interdit (l'owner a déjà plein accès).

# Reste à traiter (brainstorm)

- ✅ `magic_list.md` amendé (1er consommateur) pour suivre ce modèle.
- (Implémentation/architecture : hors brainstorm — suivra les règles d'archi du projet.)

# Journal de décisions (révisables)

- **T1 — Granularité** : partage **par élément** (généralisation de magic_list), pas d'« espaces ».
- **T2 — Droits = bitmask Linux** : `r=4`, `w=2`, `x=1` (réservé). `w` implique `r` ⇒ un grant vaut
  `4` ou `6` ; pas de grant = aucun accès.
- **T3 — Administration owner-only** : rename / partages / suppression réservés à l'owner ;
  l'écriture ne porte que sur le contenu.
- **T4 — Clé bénéficiaire = `user_id` global** (exposé `userId`).
- **T5 — Registre central générique** dans `common/`, **une seule table `shares`** de grants
  `(resource_type, resource_id, user_id, access)`, **row uniquement si partagé** ; l'élément garde
  `owner_id`.
- **T6 — Résolution d'accès** : owner → plein + admin ; sinon grant `(R, user)` ; sinon aucun accès.
  **Pas de check d'appartenance famille.**
- **T7 — API embarquée par ressource** (`sharedWith` + `access` effectif + `visibility` dérivée
  `PERSONAL`/`SHARED`), registre interne ; pas d'endpoint monitoring en itération 1.
- **T8 — Le partage est PAR USER.** « Partager à une famille » = **commodité du front** (expansion
  en N grants individuels) + regroupement à l'affichage. Le back ne connaît pas les familles dans le
  partage. Conséquences assumées : snapshot (pas d'héritage dynamique) ; quitter la famille ne retire
  pas l'accès.
- **T9 — Erreurs & HTTP** : suit le **principe projet** (encapsulation + re-mapping à chaque couche,
  chaîne `#[source]`, remontée au contrôleur). Codes **standards web** : 401 / 404 (lecture refusée,
  non révélée) / 403 (écriture sans `w` ou admin non-owner) / 409 / 500.
