# Navigator — Glossaire (langage ubiquitaire DDD)

> Vocabulaire métier partagé. Un terme est ajouté ici **une fois validé** en brainstorm/spec.
> Chaque terme est bilingue : libellé **français** (titre) ↔ **anglais / identifiant code**.
> Champs : Anglais/code · Description · Alternatives · Exemples.
> Organisé par **bounded context** (esprit DDD : un même mot peut différer selon le contexte).
> Les concepts communs à plusieurs domaines vivent dans **Transverse**. Termes triés alpha dans chaque section.
> **Dates** : toutes les dates sont **inclusives** — le jour indiqué est compris (ex. un budget du 2 au 18 mai couvre le 2 **et** le 18).

---

## Transverse (partage & propriété)

### Administration
- **Anglais / code** : Administration (owner-only)
- **Description** : Actions réservées au seul propriétaire d'un élément : le renommer, gérer son partage (ajouter/retirer des octrois), le supprimer. Jamais déléguée, indépendante du droit d'écriture.
- **Alternatives** : N/A
- **Exemples** : sur une liste, seul le propriétaire peut la renommer ou la supprimer.

### Droit d'accès
- **Anglais / code** : Access right / bitmask (`access`)
- **Description** : Droit d'un non-propriétaire sur le **contenu** d'un élément, encodé en bitmask façon Linux : lecture `r`=4, écriture `w`=6 (l'écriture implique la lecture) ; l'absence d'octroi = aucun accès. L'API expose l'accès **effectif** de l'appelant sur chaque ressource.
- **Alternatives** : lecture (4) / écriture (6), accès effectif, bitmask.
- **Exemples** : `6` = collaborer sur les items ; `4` = lecture seule ; aucun octroi ⇒ 404.

### Octroi
- **Anglais / code** : Grant (ligne de `shares` / `sharedWith`)
- **Description** : Autorisation individuelle `(ressource → utilisateur → droit)` donnée par le propriétaire ; une ligne n'existe que si l'élément est partagé avec cet utilisateur (au plus un octroi par couple). Le destinataire est le **bénéficiaire** (`userId`).
- **Alternatives** : grant, bénéficiaire ; s'octroyer à soi-même / au propriétaire est ignoré.
- **Exemples** : retirer le dernier octroi rend l'élément de nouveau personnel.

### Partage
- **Anglais / code** : Sharing (`shares`)
- **Description** : Mécanisme transverse unique rendant un élément — personnel par défaut — accessible à d'autres utilisateurs, **par utilisateur**. Registre polymorphe unique (table `shares`). C'est un **instantané** : il ne suit pas dynamiquement l'appartenance à la famille.
- **Alternatives** : sharing ; instantané (snapshot, pas dynamique) ; granularité par élément, pas d'« espaces ».
- **Exemples** : partager une liste = créer un octroi par membre visé ; rejoindre la famille après coup ne donne pas accès automatiquement.

### Partage avec la famille
- **Anglais / code** : Share with the family (commodité front)
- **Description** : « Partager avec ma famille » n'existe pas côté back : le front itère les membres de la famille et crée un octroi individuel par utilisateur. Exclure quelqu'un = ne pas créer son octroi.
- **Alternatives** : le back ne connaît ni `family_id`, ni niveau par défaut, ni dépendance sharing → family.
- **Exemples** : l'écran affiche « partagé avec la famille Dupont » alors que le back ne voit que des octrois individuels.

### Propriétaire
- **Anglais / code** : Owner (`owner_id` / `owner_username`)
- **Description** : Créateur d'un élément, porté par la table métier. Toujours au-dessus des droits : accès complet (`rw`) + administration, et seul à pouvoir administrer. La propriété n'est pas déléguée.
- **Alternatives** : owner, ownership.
- **Exemples** : le propriétaire d'un compte, d'un calendrier, d'une recette ou d'une liste.

### Visibilité
- **Anglais / code** : Visibility (dérivée : `PERSONAL` | `SHARED`)
- **Description** : Notion **dérivée** (jamais stockée en colonne) : `PERSONAL` (aucun octroi, seul le propriétaire y accède) ou `SHARED` (au moins un octroi). Alimente le badge affiché par le front.
- **Alternatives** : Personnel / Partagé.
- **Exemples** : dès le premier octroi, un élément passe `PERSONAL → SHARED` ; retirer tous les octrois le repasse `PERSONAL`.

---

## magic_list

### Article de liste magique
- **Anglais / code** : Magic list item (`MagicListItem`)
- **Description** : Ligne d'une liste magique : **titre** obligatoire, **contenu** optionnel, plus `dueDate`, statut et coché optionnels ; supprimée en cascade avec sa liste.
- **Alternatives** : item, article.
- **Exemples** : « Lait » sur une liste de courses ; un article avec une date d'échéance sur une liste de vacances.

### Article terminé (règle « completed »)
- **Anglais / code** : Completed item (dérivé : `checked == true || status == DONE`)
- **Description** : Règle **dérivée** unifiant l'action « vider les terminés » et le regroupement « Terminés » : un article compte comme terminé s'il est coché **ou** au statut DONE.
- **Alternatives** : completed ; « vider les articles terminés » (Clear completed).
- **Exemples** : un article coché et un article DONE sont tous deux « terminés ».

### Coché
- **Anglais / code** : Checked (`checked`)
- **Description** : Drapeau de complétion façon liste de courses ; la case n'est proposée que sur les listes TASK et est remise à `false` sur une instance générée.
- **Alternatives** : case à cocher (vs modèle statut / tableau de tâches).
- **Exemples** : cocher « Pain » met `checked = true`.

### En retard
- **Anglais / code** : Overdue (dérivé)
- **Description** : État d'affichage : un article **non terminé** dont la `dueDate` est dépassée (par rapport à la date locale du jour) est mis en évidence.
- **Alternatives** : past due, échéance dépassée.
- **Exemples** : un article dû hier et non terminé s'affiche en surbrillance.

### Instance
- **Anglais / code** : Instance (générée via `.../generate`)
- **Description** : Liste indépendante produite en **générant** depuis un modèle ; aucun lien ni FK vers le modèle (le supprimer n'affecte pas l'instance), personnelle à sa création.
- **Alternatives** : liste générée.
- **Exemples** : « Vacances juillet » générée depuis le modèle « Valise vacances » subsiste même après suppression du modèle.

### Liste magique
- **Anglais / code** : Magic list (`MagicList`, table `magic_list`)
- **Description** : Entité cœur de Navigator : liste **réplicable, partageable et cochable**, appartenant à un utilisateur et contenant des articles. La table ne garde que le propriétaire ; visibilité, partage et droits vivent dans le mécanisme de partage.
- **Alternatives** : liste.
- **Exemples** : liste de courses, liste de valise, liste de tâches, liste de jouets.

### Modèle
- **Anglais / code** : Template (`MagicListType::Template` / `TEMPLATE`)
- **Description** : Liste de type TEMPLATE servant de **modèle réutilisable** d'où l'on génère des instances SIMPLE ou TASK ; le supprimer ne supprime pas les listes générées (pas de FK cascade).
- **Alternatives** : « Template » gardé en anglais (nom de feature).
- **Exemples** : « Valise vacances » réutilisée à chaque départ.

### Statut d'article
- **Anglais / code** : Item status (`MagicListItemStatus` : `TODO` | `IN_PROGRESS` | `DONE`)
- **Description** : Champ optionnel façon **tableau de tâches** sur les articles d'une liste TASK ; sert à regrouper « À faire / En cours / Terminé ». Peut coexister avec `checked`.
- **Alternatives** : task-board (vs modèle coché / liste de courses).
- **Exemples** : un article passé en `IN_PROGRESS` apparaît dans le groupe « En cours ».

### Type de liste magique
- **Anglais / code** : Magic list type (`MagicListType` : `SIMPLE` | `TASK` | `TEMPLATE`)
- **Description** : Value object fixant le modèle de complétion et l'UI : SIMPLE (notes à plat, sans case), TASK (case + statut), TEMPLATE (modèle réutilisable).
- **Alternatives** : —
- **Exemples** : choisir TASK pour une liste de courses cochable.

---

## calendar

### Agenda
- **Anglais / code** : Agenda / summary (`.../calendars/summary`)
- **Description** : Vue agrégée **à plat**, sur une fenêtre de temps, des événements de tous les calendriers visibles (possédés + partagés) plus ceux où l'utilisateur est invité ; les récurrences y sont déployées en occurrences.
- **Alternatives** : résumé d'agenda, agenda view.
- **Exemples** : sans `from`/`to`, le back renvoie aujourd'hui → aujourd'hui + 7 jours.

### Calendrier
- **Anglais / code** : Calendar (`Calendar`)
- **Description** : Conteneur d'événements appartenant à un utilisateur, identifié par un **nom** et une **couleur**, administré par son seul propriétaire ; l'accès au-delà est délégué au partage. Un utilisateur peut posséder plusieurs calendriers.
- **Alternatives** : — (les anciens `type: SHARED|PERSONAL` et `familyId`/`visibility` ont été retirés, C1).
- **Exemples** : un calendrier personnel créé avec un nom et une couleur.

### Élément de calendrier
- **Anglais / code** : Calendar event (`CalendarEvent`)
- **Description** : Élément temporel d'un calendrier, soit un **événement** (un moment) soit une **tâche** (à faire pour une date), selon son `kind` ; porte titre, dates, heure/durée optionnelles, récurrence, invités, rappel.
- **Alternatives** : item, événement/tâche.
- **Exemples** : un rendez-vous, un anniversaire ou une corvée récurrente stockés en une ligne `CalendarEvent`.

### Fait
- **Anglais / code** : Done (`done`)
- **Description** : Drapeau de complétion pertinent uniquement quand `kind == TASK` (défaut `false`) ; cocher une tâche ne la supprime pas. Sur un EVENT il reste `false` et est ignoré.
- **Alternatives** : terminé, coché.
- **Exemples** : cocher une tâche la garde dans le calendrier au lieu de la supprimer.

### Fenêtre de l'agenda
- **Anglais / code** : Agenda window (`from` / `to`)
- **Description** : Plage `[from, to]` sur laquelle l'agenda est calculé et les récurrences déployées ; par défaut aujourd'hui → aujourd'hui + 7 jours.
- **Alternatives** : fenêtre par défaut, `[from, to]`.
- **Exemples** : `?from=&to=` ⇒ fenêtre par défaut d'une semaine.

### Invité
- **Anglais / code** : Invitee (`calendar_event_invites`, `member_username`)
- **Description** : Membre de la famille (par username) convié à un événement ; l'invitation fait apparaître l'événement dans son agenda **sans** lui donner accès au calendrier.
- **Alternatives** : participant, attendee.
- **Exemples** : « inviter toute la famille » crée un invité par membre.

### Nature de l'élément
- **Anglais / code** : Kind (`EventKind` : `EVENT` | `TASK`)
- **Description** : Value object distinguant un **événement** (un moment) d'une **tâche** (à faire pour une date, avec drapeau `done`, recouvrant un peu magic_list).
- **Alternatives** : kind, type.
- **Exemples** : `EVENT` pour un rendez-vous à 10:00, `TASK` pour une corvée à échéance.

### Occurrence
- **Anglais / code** : Occurrence (déployée depuis `RecurrenceType`)
- **Description** : Instance unique d'un élément récurrent dans la fenêtre d'agenda ; l'élément est stocké une fois + sa règle, et éditer/supprimer affecte **toute la série**, jamais une occurrence isolée.
- **Alternatives** : instance de série.
- **Exemples** : chaque occurrence d'un événement annuel tombant dans `[from, to]`.

### Rappel
- **Anglais / code** : Reminder (`reminderMinutesBefore`)
- **Description** : Nombre de minutes avant l'élément où un rappel se déclencherait ; pour l'instant **seulement stocké/exposé**, sans déclenchement côté back (C6).
- **Alternatives** : —
- **Exemples** : `reminderMinutesBefore` posé mais ne déclenchant encore aucune notification.

### Récurrence
- **Anglais / code** : Recurrence (`RecurrenceType` : `NONE` | `DAILY` | `WEEKLY` | `MONTHLY` | `YEARLY`)
- **Description** : Règle de répétition **simple et infinie**, sans date de fin ni exception par occurrence ; le back déploie les occurrences dans la fenêtre.
- **Alternatives** : règle récurrente.
- **Exemples** : YEARLY pour les anniversaires, WEEKLY pour le sport.

---

## bank_account

### Budget
- **Anglais / code** : Budget (`budgets`, `budget_amounts`)
- **Description** : Enveloppe de dépense **récurrente** avec sa propre périodicité, à laquelle des dépenses peuvent être **rattachées** ; son reste du mois est le `currentAmount`. Changer son montant = clôturer le budget et en créer un nouveau (voir Date de fin).
- **Alternatives** : enveloppe de dépense.
- **Exemples** : un budget restaurant de 1000 à partir du 01/01/2026, relevé à 2000.

### Charge
- **Anglais / code** : Charge (`charges`, `charge_amounts`)
- **Description** : Sortie **récurrente** (MONTHLY ou YEARLY), stockée avec ses bornes `[startDate, endDate?]` et projetée une occurrence par période. Changer son montant = clôturer la charge et en créer une nouvelle (voir Date de fin).
- **Alternatives** : sortie récurrente (une sortie ponctuelle est une Dépense, jamais une Charge).
- **Exemples** : une charge mensuelle couvrant tout le mois civil, même créée en cours de mois.

### Compte bancaire
- **Anglais / code** : Bank account (`bank_accounts`)
- **Description** : Compte appartenant à un utilisateur, portant un solde initial, une date de début et quatre mouvements (charges, crédits, dépenses, budgets) ; toute mutation renvoie l'agrégat complet.
- **Alternatives** : —
- **Exemples** : créer un compte avec un nom, un `startingAmount` et un `startDate`.

### Crédit
- **Anglais / code** : Credit (`credits`)
- **Description** : Entrée **ponctuelle** portant un montant et une `creditDate`. Toujours ponctuelle (un salaire n'est jamais identique d'un mois à l'autre, donc non modélisé comme récurrent, B5).
- **Alternatives** : entrée ponctuelle.
- **Exemples** : un salaire versé à une `creditDate`.

### Date de fin
- **Anglais / code** : End date (`end_date`)
- **Description** : Borne de **fin** d'une charge/budget récurrent (jour inclus). Deux usages : (1) **supprimer** une charge/budget = poser sa date de fin, les mois passés étant conservés (B8) ; (2) **changer son montant** = la clôturer au **dernier jour du mois précédent** et démarrer une nouvelle charge/budget le mois courant. On ne modifie jamais le passé.
- **Alternatives** : date d'arrêt.
- **Exemples** : passer un budget de 1000 à 2000 en juin ⇒ clôturer l'ancien au 31/05 et créer un nouveau budget de 2000 démarrant le 01/06.

### Date de débit
- **Anglais / code** : Debit date (`debit_date`)
- **Description** : Moment où une sortie **apparaît sur le compte** ; pilote le partage réel/prévisionnel : `debitDate ≤ aujourd'hui` = déjà débité (B2).
- **Alternatives** : —
- **Exemples** : `debitDate=02/04` ⇒ compte dans le solde réel seulement à partir du 02/04.

### Date de dépense
- **Anglais / code** : Expense date (`expense_date`)
- **Description** : Moment où une dépense **a été faite** ; détermine son mois d'appartenance et le mois de budget consommé (B7).
- **Alternatives** : —
- **Exemples** : `expenseDate=31/03` ⇒ dépense de mars.

### Dépense
- **Anglais / code** : Expense (`expenses`)
- **Description** : Sortie **ponctuelle** portant une `expenseDate` (pilote le mois) et une `debitDate` (pilote réel/prévisionnel). Elle peut être **rattachée à un budget** — elle en consomme alors le mois — ou rester hors budget. Il n'y a pas de « dépense de budget » séparée.
- **Alternatives** : sortie ponctuelle (distincte d'une Charge récurrente) ; une dépense rattachée à un budget reste une dépense.
- **Exemples** : `expenseDate=31/03, debitDate=02/04` ⇒ appartient à mars mais impacte le solde le 02/04 ; rattachée au budget « Restaurant », elle en consomme le mois de mars.

### Montant courant du budget
- **Anglais / code** : Current amount (`currentAmount`)
- **Description** : Reste d'un budget pour un mois M : montant du budget pour M moins les **dépenses rattachées** au budget et affectées à M (par `expenseDate`) ; peut être négatif.
- **Alternatives** : reste de l'enveloppe.
- **Exemples** : un budget de 1000 avec 300 de dépenses dans le mois laisse `currentAmount = 700`.

### Périodicité
- **Anglais / code** : Periodicity (`ChargePeriodicity`, `BudgetPeriodicity`)
- **Description** : Value object donnant la cadence : `MONTHLY` (mois civil entier), `YEARLY` (année civile), `ONE_SHOT` (budget seulement — enveloppe finie unique).
- **Alternatives** : —
- **Exemples** : un budget MONTHLY couvre tout le mois civil, même créé en cours de mois (jamais 08/03 → 08/04).

### Prévision de fin de mois
- **Anglais / code** : Remaining amount (`remainingAmount`)
- **Description** : Solde de fin de mois si tous les budgets ont été consommés. C'est `= montant début de mois - somme(montant initiaux des budgets) - somme(charges) - somme(dépenses)`
- **Alternatives** : End-of-month forecast (`endOfMonthForecast`)
- **Exemples** : pour un mois futur, `remainingAmount` est la projection, `actualAmount` juste le solde d'ouverture.

### Récurrent / ponctuel
- **Anglais / code** : Recurring
- **Description** : Distinction **cœur** du domaine : charges et budgets sont récurrents (alignés sur le calendrier, projetés une occurrence par période) ; crédits, dépenses et dépenses de budget sont ponctuels (un seul montant, pas de récurrence).
- **Alternatives** : —
- **Exemples** : une sortie ponctuelle est une Dépense, jamais une Charge.

### Solde initial
- **Anglais / code** : Starting amount (`starting_amount`, `start_date`)
- **Description** : Solde du compte à sa **date de début**, base de calcul de tous les soldes (réel et prévisionnel).
- **Alternatives** : solde de départ.
- **Exemples** : un compte créé avec un `startingAmount` et un `startDate`.

### Solde courant
- **Anglais / code** : Actual amount (`actualAmount`)
- **Description** : Solde réel : `startingAmount` + Σ(crédits − charges − dépenses − dépenses de budget) dont la `debitDate ≤ aujourd'hui` ; seuls les mouvements déjà débités comptent (B2). Pour un mois futur, égale le solde d'ouverture.
- **Alternatives** : solde d'ouverture (mois futur = rien de débité).
- **Exemples** : une dépense de `debitDate 02/04` n'entre dans le réel qu'à partir du 02/04.

---

## recipes & meals

### Liste de courses
- **Anglais / code** : Shopping list — une **Magic List** normale
- **Description** : Liste cochable générée depuis une sélection validée ; c'est une Magic List ordinaire (son propre domaine), librement ajustable (ajout/modif/suppression d'items).
- **Alternatives** : Magic List (nom produit anglais) ; non modélisée côté front (R8).
- **Exemples** : la liste bâtie à partir des recettes placées sur les jours de la semaine.

### Livre de recettes
- **Anglais / code** : Cookbook / catalogue
- **Description** : Collection personnelle/partagée de recettes parcourue avec recherche, filtre par catégorie, note minimale et tri (paginé).
- **Alternatives** : carnet de recettes, catalogue.
- **Exemples** : parcourir les recettes `PLAT` avec une note minimale.

### Note
- **Anglais / code** : Rating (`rating`)
- **Description** : Valeur **unique et partagée** par recette : toute personne y ayant accès la voit et peut la modifier.
- **Alternatives** : notation.
- **Exemples** : une recette notée puis re-notée par n'importe quel membre ayant accès.

### Recette
- **Anglais / code** : Recipe (`Recipe`, table `recipes`)
- **Description** : Entité du livre de recettes : nom, catégorie, image de présentation, ingrédients structurés, étapes ordonnées, note partagée ; CRUD complet.
- **Alternatives** : —
- **Exemples** : une recette `PLAT` avec ses ingrédients et ses étapes.

---

## family

### Administrateur de famille
- **Anglais / code** : Family admin (`is_admin` sur `family_members`, exposé `role: ADMIN`)
- **Description** : Rôle **collectif et révocable** dans une famille, source de tous les pouvoirs de gouvernance : inviter/retirer des membres, octroyer/révoquer l'admin, renommer, archiver, réactiver. Invariant : au moins **un** admin tant que la famille a des membres.
- **Alternatives** : admin, rôle admin ; à distinguer de l'**Administration** (Transverse), qui vise un élément partagé et reste réservée à son propriétaire.
- **Exemples** : n'importe quel admin peut promouvoir un membre ou retirer l'admin d'un autre (créateur inclus), tant qu'il reste un admin.

### Créateur
- **Anglais / code** : Creator (`created_by` sur `families`)
- **Description** : Utilisateur qui a créé la famille, conservé comme **attribut d'audit immuable** ; il n'a **aucun pouvoir permanent** (s'il perd l'admin, il redevient un membre ordinaire). Une famille n'a pas de propriétaire permanent.
- **Alternatives** : à ne pas confondre avec le **Propriétaire** (Transverse), qui garde ses droits à vie.
- **Exemples** : un créateur rétrogradé reste historisé comme créateur, mais ne peut plus rien configurer.

### Famille
- **Anglais / code** : Family (`Family`, table `families`)
- **Description** : Groupe d'utilisateurs (les membres) support de l'organisation Navigator ; **gouvernée par le rôle admin**, pas par un propriétaire. Un utilisateur peut appartenir à plusieurs familles et en créer autant qu'il veut.
- **Alternatives** : —
- **Exemples** : sa propre famille et la famille de son conjoint, entre lesquelles il bascule.

### Invitation
- **Anglais / code** : Family invitation (`family_invitations`)
- **Description** : Proposition faite par un admin à un **utilisateur Navigator existant** (désigné par email ou username) de rejoindre la famille ; **in-app**, en attente jusqu'à acceptation (l'invité devient membre) ou refus. Au plus une invitation en attente par (famille, utilisateur), sans expiration.
- **Alternatives** : — (pas de lien/code partageable, pas d'onboarding d'un non-inscrit).
- **Exemples** : inviter quelqu'un par email ; un admin peut annuler une invitation encore en attente.

### Membre
- **Anglais / code** : Family member (`family_members`)
- **Description** : Utilisateur appartenant à une famille, avec un rôle (membre simple ou administrateur). Tout membre peut **quitter** la famille de lui-même ; seuls les admins invitent ou retirent des membres.
- **Alternatives** : —
- **Exemples** : un membre simple ne peut pas inviter ; le dernier admin ne peut partir sans promouvoir un autre membre.
