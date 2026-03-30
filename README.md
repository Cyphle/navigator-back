# Navigator

Navigate through your life swiftly.

# Features
## Gestion de la famille
- Récupérer la liste des membres de la famille
- Créer des comptes pour des membres de la famille

## Gestion de todo list
- Récupérer la liste de ses todo list
- Créer une todo list
- Lire le contenu d'une todo list
- Ajouter des items
- Modifier des items
- Supprimer des items
- Partager une todo liste avec toute une famille ou quelques membres de la famille

## Agenda
-> ou alors lier à google agenda
- Ajouter des agendas
- Partager des agendas
- Ajouter un événement
- Ajouter un événement récurrent
- Modifier des événements
- Modifier une occurence d'événement récurrent
- Supprimer un événement
- Supprimer une occurence d'un événement réccurent

## Repas
- Ajouter une recette
- Partager une recette avec la famille
- Sélectionner des recettes à faire
- Générer la liste des ingrédients

# Configuration
Le projet utilise une configuration hiérarchique basée sur des fichiers YAML et des variables d'environnement.

## Fichiers de configuration
Les fichiers sont chargés dans cet ordre :
1. `config/default.yaml` : configuration par défaut.
2. `config/local.yaml` (optionnel) : configuration locale (ignorée par git).

## Variables d'environnement
Vous pouvez surcharger n'importe quelle valeur de configuration en utilisant des variables d'environnement préfixées par `NAVIGATOR_`.

Le séparateur pour les structures imbriquées est le simple underscore `_`.

### Exemples :
- Pour modifier le port de l'application (`app.port`) : `NAVIGATOR_APP_PORT=8080`
- Pour modifier l'hôte de la base de données (`database.host`) : `NAVIGATOR_DATABASE_HOST=postgres`
- Pour modifier le niveau de log (`logging.level`) : `NAVIGATOR_LOGGING_LEVEL=info`
