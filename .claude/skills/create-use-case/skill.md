# Create a Domain Use Case

## Short description (for LLMs)
Creates a new application-layer use case function in a domain following the navigator-back pattern: async fn generic over `ActixState<DB>` + username parameter, transaction lifecycle managed here, errors mapped to `Box<dyn ApplicationError>`, and unit tests using `mock_actix_state`.

## Persona
Tu es un ingénieur backend Rust travaillant sur navigator-back. Tu connais l'architecture en couches inspirée du DDD : domain → repositories (traits) → usecases (logique métier) → http (controller/middleware). Tu es strict sur le fait de ne pas laisser fuir les préoccupations d'infrastructure dans les use cases.

## Quand utiliser ce skill
- Une nouvelle fonctionnalité nécessite une logique métier qui orchestre un ou plusieurs appels de repository
- Tu dois ajouter une fonction use case dans `src/domains/<domain>/usecases/`
- Tu câbles un nouvel endpoint et tu as besoin de la couche application en premier

## Le skill en détail

### 1. Emplacement du fichier
Créer le use case dans :
```
src/domains/<domain>/usecases/<verb>_<noun>_use_case.rs
```
L'exposer dans `src/domains/<domain>/usecases/mod.rs`.

### 2. Signature de la fonction
```rust
pub async fn <verb>_<noun>_use_case<DB: DbConnection>(
    state: web::Data<ActixState<DB>>,
    username: String,
    // ...entrées domaine supplémentaires (commands, filters, ids)...
) -> Result<OutputType, Box<dyn ApplicationError>>
where
    for<'a> <DB as DbConnection>::Tx<'a>: AsPgConn,
```
- Toujours générique sur `DB: DbConnection` pour la testabilité.
- La clause `where` est requise quand le repository a besoin d'une vraie connexion Postgres.
- Retourner `Box<dyn ApplicationError>` — jamais `sqlx::Error` ni `anyhow`.

### 3. Cycle de vie de la transaction
Le use case possède la transaction. La commencer ici, jamais dans le repository.
```rust
let mut tx = state
    .db_connection
    .begin()
    .await
    .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;
```
Sur le chemin heureux, la transaction est droppée (auto-rollback) sauf commit explicite :
```rust
tx.commit().await.map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;
```
Sur erreur, rollback explicite avant de retourner :
```rust
Err(err) => {
    tx.rollback().await
        .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;
    Err(Box::new(RepositoryError { error: err.to_string() }))
}
```

### 4. Mapping des erreurs
Toujours mapper les erreurs de repository vers `Box<dyn ApplicationError>` via `RepositoryError` :
```rust
.map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)
```

### 5. Tests dans le même fichier
Écrire un bloc `#[cfg(test)]` dans le même fichier. Couvrir au minimum trois scénarios :
- Échec de connexion DB (`MockPoolPostgresError`)
- Échec du repository (`should_error: true`)
- Chemin heureux

Utiliser des helpers factory pour éviter le boilerplate :
```rust
#[cfg(test)]
mod tests {
    use super::<verb>_<noun>_use_case;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
    use crate::testing::repositories::mock_database::{MockPoolPostgres, MockPoolPostgresError};

    fn make_state_ok() -> web::Data<MockActixState> {
        mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                // ne renseigner que ce dont ce use case a besoin
                ..MockStateConfig::default()
            },
        )
    }

    fn make_state_db_error() -> web::Data<ActixState<MockPoolPostgresError>> {
        mock_actix_state(MockPoolPostgresError, MockStateConfig::default())
    }

    fn make_state_repo_error() -> web::Data<MockActixState> {
        mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                <domain>_should_error: true,
                ..MockStateConfig::default()
            },
        )
    }

    #[actix_web::test]
    async fn should_error_on_db_connection_failure() { ... }

    #[actix_web::test]
    async fn should_error_on_repository_failure() { ... }

    #[actix_web::test]
    async fn should_return_<result>() { ... }
}
```

### 6. Checklist avant de finir
- [ ] Fichier dans `src/domains/<domain>/usecases/`
- [ ] Fonction générique sur `DB: DbConnection`
- [ ] Transaction démarrée dans le use case, pas dans le repository
- [ ] Toutes les erreurs mappées vers `Box<dyn ApplicationError>`
- [ ] Module exporté dans `usecases/mod.rs`
- [ ] Les trois scénarios de test couverts
- [ ] `cargo test` passe
