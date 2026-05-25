# Contexte

On utilise des listes tous les jours : liste de courses, liste d’affaires pour les vacances, liste de tâches, liste de jouets, etc

On les utilise dans différents contexte : prévoir les repas de la semaine, préparer les vacances, le travail à finir, etc

On les utilise pour des timing différents : pour aujourd’hui, pour les courses demain, pour les vacances dans 2 semaines, etc

Parfois on a besoin de listes qu’on utilise plusieurs fois comme la liste des choses à prendre pour les vacances.

Les Magic Lists de Navigator sont là pour répondre simplement à tous ces besoins : avoir des listes réplicables, partageables, checkables. Elles doivent être faciles d’accès et intuitives.

# Structure de données

Une Magic List d’un point de vue métier est structurée ainsi :

```rust
pub struct MagicList {
		pub id: i32,
    pub name: String,
    pub list_type: MagicListType,
    pub owner_username: String,
    pub visibility: Visibility,
    pub family_id: Option<i32>,
}

pub enum Visibility {
    Shared,
    Personal,
}

pub struct MagicListItem {
    pub id: i32,
    pub magic_list_id: i32,
    pub title: String,
    pub content: Option<String>,
    pub checked: bool,
    pub due_date: Option<NaiveDate>,
    pub status: Option<MagicListItemStatus>,
}
```

# La gestion de Magic List

## Création

En tant qu’utilisateur, je souhaite pouvoir créer une magic list. Lorsque je créé une magic list, je veux pouvoir spécifier :

- un nom
- un type : à savoir si c’est une liste simple à usage unique, une liste à checkbox avec des éléments que je peux valider/invalider comme une liste de course, ou alors un template de liste par exemple une liste de todo à faire avant chaque vacances
- si je partage cette liste avec la famille ou non
- quels sont les membres de la famille qui n’y ont pas accès. Par défaut si je partage à la famille, toute la famille y a accès

## Update

Je veux pouvoir modifier une magic list ou la supprimer.

## Delete

Si je supprime un template, cela ne doit pas supprimer les listes créées à partir de celui-ci.

## Autre features

Depuis une liste qui est un template, je veux pouvoir générer une liste simple ou une checkbox liste en lui donnant un nom particulier.

Je veux pouvoir modifier la visibilité au sein de la famille et ajouter ou retirer des membres exclus.

# La création d’item dans une liste

Un item d’une magic list est assez basique. Il contient un titre et un contenu obligatoire.

On peut rajouter quelques informations facultatives comme une due date, un status ou un check dans le cas d’une liste de type checkbox.
Je veux pouvoir faire ce qui est standard avec des éléments de liste à savoir modifier ou supprimer.

# Visualisation

Quand j’arrive sur la page de mes listes, je veux toutes les voir mais de manière résumé avec leur nom, leur type, leur visibilité et le nombre d’éléments dedans.

Sur la page d’accueil des listes, je veux des filtres simple sur le type, la visibilité et un champ de recherche par nom.

Quand je clique sur une liste, je veux voir l’ensemble des items. En fonction du type, je veux une visualisation adaptée. S’il y a des status, je veux par défaut les items soient regroupés par status. Je veux pouvoir modifier l’ordre en triant par nom ou par status ou par checkbox si la liste le permet.