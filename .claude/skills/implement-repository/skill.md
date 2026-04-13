# Implement a Repository with SQLx

## Short description (for LLMs)
Creates a SQLx repository for a domain following the navigator-back pattern: trait in the domain layer, `Sqlx<Noun>Repository` struct in `repositories/`, entity structs with `#[derive(FromRow)]`, private `_inner` methods accepting `&mut PgConnection`, mock in `testing/repositories/`, and integration tests using `#[sqlx_testcontainers::test]`.

## Persona
Tu es un ingénieur backend Rust travaillant sur navigator-back. Tu gardes les repositories focalisés sur l'accès aux données uniquement — pas de logique métier, pas de gestion de transaction (c'est la responsabilité du use case). Tu conçois des traits de repository mockables sans base de données.

## Quand utiliser ce skill
- Un domaine a besoin de stockage persistant
- Tu ajoutes une nouvelle requête à un repository existant
- Tu dois créer le trait de repository + l'implémentation SQLx complets

## Le skill en détail

### 1. Emplacements des fichiers
```
src/domains/<domain>/domain/<domain>_repository.rs              # trait (couche domaine)
src/domains/<domain>/repositories/<domain>_entity.rs            # structs de mapping SQLx
src/domains/<domain>/repositories/<domain>_sqlx_repository.rs   # implémentation
src/testing/repositories/mock_<domain>_repository.rs            # mock pour les tests
```

### 2. Trait du repository (couche domaine)
Définir le trait dans `<domain>/domain/`, pas dans `repositories/`. Le garder libre de toute infrastructure :
```rust
use async_trait::async_trait;
use crate::config::actix::AsPgConn;

#[async_trait]
pub trait <Noun>Repository: Send + Sync {
    async fn get_<noun>_for(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
    ) -> Result<Vec<<Noun>>, sqlx::Error>;

    async fn create_<noun>(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        command: &Create<Noun>Command,
    ) -> Result<<Noun>, sqlx::Error>;
}
```
- Retourner `sqlx::Error` depuis les méthodes du repository — le use case le mappe vers `ApplicationError`.
- Accepter `&mut dyn AsPgConn` pour que le mock puisse passer n'importe quoi.

### 3. Struct entité
```rust
use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct <Noun>Entity {
    pub id: i32,
    pub name: String,
    // ... toutes les colonnes retournées par la requête
}
```
Les noms de champs doivent correspondre exactement aux alias SQL.

### 4. Implémentation SQLx
```rust
use sqlx::{Error, Postgres, PgConnection};
use async_trait::async_trait;
use crate::config::actix::AsPgConn;

pub struct Sqlx<Noun>Repository;

impl Sqlx<Noun>Repository {
    // Méthodes inner privées acceptant &mut PgConnection —
    // testables directement sans passer par le &mut dyn AsPgConn du trait.
    async fn get_<noun>_for_inner(
        &self,
        conn: &mut PgConnection,
        username: &str,
    ) -> Result<Vec<<Noun>>, Error> {
        let entities = sqlx::query_as::<Postgres, <Noun>Entity>(
            "SELECT ... FROM <table> WHERE ...",
        )
        .bind(username)
        .fetch_all(&mut *conn)
        .await?;

        Ok(entities.into_iter().map(|e| self.to_domain(e)).collect())
    }

    fn to_domain(&self, entity: <Noun>Entity) -> <Noun> {
        <Noun> {
            id: entity.id,
            name: entity.name,
            // ...
        }
    }
}

#[async_trait]
impl <Noun>Repository for Sqlx<Noun>Repository {
    async fn get_<noun>_for(&self, conn: &mut dyn AsPgConn, username: &str) -> Result<Vec<<Noun>>, Error> {
        self.get_<noun>_for_inner(conn.as_pg_conn(), username).await
    }

    async fn create_<noun>(&self, conn: &mut dyn AsPgConn, username: &str, command: &Create<Noun>Command) -> Result<<Noun>, Error> {
        self.create_<noun>_inner(conn.as_pg_conn(), username, command).await
    }
}
```

### 5. Patterns SQL
- Utiliser les placeholders `$1, $2, ...`, jamais d'interpolation de chaînes.
- Pour les insertions en masse, utiliser `UNNEST` : `INSERT INTO t (a, b) SELECT * FROM UNNEST($1::int[], $2::text[])`.
- Toujours utiliser `RETURNING id` quand on a besoin de la PK insérée.
- Pour la correspondance insensible à la casse : `LOWER(column) = LOWER($1)`.

### 6. Mock repository pour les tests
```rust
// src/testing/repositories/mock_<noun>_repository.rs
pub struct Mock<Noun>Repository {
    pub items: Vec<<Noun>>,
    pub should_error: bool,
}

#[async_trait]
impl <Noun>Repository for Mock<Noun>Repository {
    async fn get_<noun>_for(&self, _conn: &mut dyn AsPgConn, _username: &str) -> Result<Vec<<Noun>>, sqlx::Error> {
        if self.should_error { return Err(sqlx::Error::RowNotFound); }
        Ok(self.items.clone())
    }

    async fn create_<noun>(&self, _conn: &mut dyn AsPgConn, username: &str, command: &Create<Noun>Command) -> Result<<Noun>, sqlx::Error> {
        if self.should_error { return Err(sqlx::Error::RowNotFound); }
        Ok(<Noun> { id: 1, name: command.name.clone(), /* ... */ })
    }
}
```
L'ajouter dans `MockStateConfig` et `mock_actix_state` dans `src/testing/actix/mock_state.rs`.

### 7. Tests d'intégration avec testcontainers
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, Transaction};

    // Helper pour créer les données de fixture
    async fn create_<dep>(tx: &mut Transaction<'_, Postgres>, ...) -> i32 {
        let row: (i32,) = sqlx::query_as("INSERT INTO ... RETURNING id")
            .bind(...)
            .fetch_one(&mut **tx).await.unwrap();
        row.0
    }

    #[sqlx_testcontainers::test]
    async fn test_get_<noun>_for(mut conn: sqlx::PgConnection) {
        let repo = Sqlx<Noun>Repository;
        let mut tx = conn.begin().await.unwrap();

        // Arrange : insérer les données minimales requises
        let id = create_<dep>(&mut tx, ...).await;

        // Act : appeler la méthode _inner directement
        let result = repo.get_<noun>_for_inner(&mut *tx, "alice").await.unwrap();

        // Assert
        assert_eq!(result.len(), 1);
    }
}
```
- Les tests appellent les méthodes `_inner` directement, en contournant `dyn AsPgConn`.
- Chaque test obtient sa propre transaction — pas d'état partagé.
- Ne jamais committer dans les tests ; laisser la transaction drop (auto-rollback).

### 8. Checklist avant de finir
- [ ] Trait dans la couche `domain/`, pas dans `repositories/`
- [ ] Struct entité dérive `FromRow` avec les noms de champs correspondant aux alias SQL
- [ ] Méthodes `_inner` privées acceptant `&mut PgConnection`
- [ ] L'impl publique du trait délègue aux `_inner` via `conn.as_pg_conn()`
- [ ] Aucune logique métier dans le repository
- [ ] Mock dans `testing/repositories/`
- [ ] Mock câblé dans `MockStateConfig` et `mock_actix_state`
- [ ] Tests d'intégration avec `#[sqlx_testcontainers::test]` pour chaque méthode
- [ ] `cargo test` passe
