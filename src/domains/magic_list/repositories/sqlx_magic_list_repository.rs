use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::common::visibility::Visibility;
use crate::domains::magic_list::domain::create_magic_list_command::CreateMagicListCommand;
use crate::domains::magic_list::domain::create_magic_list_item_command::CreateMagicListItemCommand;
use crate::domains::magic_list::domain::magic_list::MagicList;
use crate::domains::magic_list::domain::magic_list_repository::MagicListRepository;
use async_trait::async_trait;
use sqlx::{PgConnection, Pool, Postgres};

#[derive(sqlx::FromRow)]
struct MagicListRow {
    id: i32,
    owner_username: String,
    visibility: String,
    family_id: Option<i32>,
}

pub struct SqlxMagicListRepository {
    pub pool: Pool<Postgres>,
}

impl SqlxMagicListRepository {
    async fn find_by_id_inner(&self, conn: &mut PgConnection, magic_list_id: i32) -> Result<MagicList, Box<dyn ApplicationError>> {
        let row: MagicListRow = sqlx::query_as(
            "SELECT ml.id, u.username as owner_username, ml.visibility, ml.family_id \
             FROM magic_list ml \
             JOIN users u ON ml.owner_id = u.id \
             WHERE ml.id = $1",
        )
        .bind(magic_list_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;

        Ok(MagicList {
            id: row.id,
            owner_username: row.owner_username,
            visibility: match row.visibility.as_str() {
                "SHARED" => Visibility::Shared,
                _ => Visibility::Personal,
            },
            family_id: row.family_id,
        })
    }

    async fn is_family_member_inner(&self, conn: &mut PgConnection, username: &str, family_id: i32) -> Result<bool, Box<dyn ApplicationError>> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(\
                SELECT 1 FROM family_members fm \
                JOIN users u ON fm.user_id = u.id \
                WHERE u.username = $1 AND fm.family_id = $2\
            )",
        )
        .bind(username)
        .bind(family_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;

        Ok(result.0)
    }

    async fn add_item_inner(&self, conn: &mut PgConnection, magic_list_id: i32, command: CreateMagicListItemCommand) -> Result<(), Box<dyn ApplicationError>> {
        sqlx::query(
            "INSERT INTO magic_list_item (magic_list_id, title, content, checked, due_date, status) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(magic_list_id)
        .bind(&command.title)
        .bind(&command.content)
        .bind(command.checked.unwrap_or(false))
        .bind(command.due_date)
        .bind(command.status.as_ref().map(|s| s.to_string()))
        .execute(&mut *conn)
        .await
        .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;

        Ok(())
    }
}

#[async_trait]
impl MagicListRepository for SqlxMagicListRepository {
    async fn create(&self, username: &String, command: CreateMagicListCommand) -> Result<(), Box<dyn ApplicationError>> {
        let owner_id: (i32,) = sqlx::query_as(
            "SELECT id FROM users WHERE username = $1",
        )
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;

        sqlx::query(
            "INSERT INTO magic_list (name, type, visibility, owner_id, family_id, excluded_user_ids) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(command.name)
        .bind(command.magic_list_type.to_string())
        .bind(command.visibility.to_string())
        .bind(owner_id.0)
        .bind(command.family_id)
        .bind(command.excluded_member_ids)
        .execute(&self.pool)
        .await
        .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;

        Ok(())
    }

    async fn find_by_id(&self, magic_list_id: i32) -> Result<MagicList, Box<dyn ApplicationError>> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;
        self.find_by_id_inner(&mut *conn, magic_list_id).await
    }

    async fn is_family_member(&self, username: &str, family_id: i32) -> Result<bool, Box<dyn ApplicationError>> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;
        self.is_family_member_inner(&mut *conn, username, family_id).await
    }

    async fn add_item(&self, magic_list_id: i32, command: CreateMagicListItemCommand) -> Result<(), Box<dyn ApplicationError>> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>)?;
        self.add_item_inner(&mut *conn, magic_list_id, command).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, Transaction};
    use chrono::NaiveDate;
    use crate::domains::magic_list::domain::magic_list_item_status::MagicListItemStatus;

    async fn create_user(tx: &mut Transaction<'_, Postgres>, username: &str) -> i32 {
        let row: (i32,) = sqlx::query_as(
            "INSERT INTO users (username, email, first_name, last_name) VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(username).bind(format!("{}@test.com", username)).bind("First").bind("Last")
        .fetch_one(&mut **tx).await.unwrap();
        row.0
    }

    async fn create_magic_list(tx: &mut Transaction<'_, Postgres>, owner_id: i32, visibility: &str, family_id: Option<i32>) -> i32 {
        let row: (i32,) = sqlx::query_as(
            "INSERT INTO magic_list (name, type, visibility, owner_id, family_id) VALUES ($1, $2, $3, $4, $5) RETURNING id"
        )
        .bind("Test list").bind("SIMPLE").bind(visibility).bind(owner_id).bind(family_id)
        .fetch_one(&mut **tx).await.unwrap();
        row.0
    }

    async fn create_family_with_member(tx: &mut Transaction<'_, Postgres>, creator_id: i32, member_id: i32) -> i32 {
        let row: (i32,) = sqlx::query_as(
            "INSERT INTO families (name, creator_id, active) VALUES ($1, $2, true) RETURNING id"
        )
        .bind("Test family").bind(creator_id)
        .fetch_one(&mut **tx).await.unwrap();
        let family_id = row.0;
        sqlx::query("INSERT INTO family_members (family_id, user_id, relation, is_admin) VALUES ($1, $2, 'PARENT', true)")
            .bind(family_id).bind(creator_id).execute(&mut **tx).await.unwrap();
        sqlx::query("INSERT INTO family_members (family_id, user_id, relation, is_admin) VALUES ($1, $2, 'CHILD', false)")
            .bind(family_id).bind(member_id).execute(&mut **tx).await.unwrap();
        family_id
    }

    #[sqlx_testcontainers::test]
    async fn test_find_by_id(mut conn: sqlx::PgConnection) {
        let repo = SqlxMagicListRepository { pool: Pool::connect_lazy("postgres://unused").unwrap() };
        let mut tx = conn.begin().await.unwrap();
        let user_id = create_user(&mut tx, "alice").await;
        let ml_id = create_magic_list(&mut tx, user_id, "SHARED", None).await;

        let result = repo.find_by_id_inner(&mut *tx, ml_id).await.unwrap();

        assert_eq!(result.owner_username, "alice");
        assert_eq!(result.visibility, Visibility::Shared);
    }

    #[sqlx_testcontainers::test]
    async fn test_is_family_member_returns_true(mut conn: sqlx::PgConnection) {
        let repo = SqlxMagicListRepository { pool: Pool::connect_lazy("postgres://unused").unwrap() };
        let mut tx = conn.begin().await.unwrap();
        let alice_id = create_user(&mut tx, "alice").await;
        let bob_id = create_user(&mut tx, "bob").await;
        let family_id = create_family_with_member(&mut tx, alice_id, bob_id).await;

        let result = repo.is_family_member_inner(&mut *tx, "bob", family_id).await.unwrap();
        assert!(result);
    }

    #[sqlx_testcontainers::test]
    async fn test_is_family_member_returns_false(mut conn: sqlx::PgConnection) {
        let repo = SqlxMagicListRepository { pool: Pool::connect_lazy("postgres://unused").unwrap() };
        let mut tx = conn.begin().await.unwrap();
        let alice_id = create_user(&mut tx, "alice").await;
        create_user(&mut tx, "charlie").await;
        let bob_id = create_user(&mut tx, "bob").await;
        let family_id = create_family_with_member(&mut tx, alice_id, bob_id).await;

        let result = repo.is_family_member_inner(&mut *tx, "charlie", family_id).await.unwrap();
        assert!(!result);
    }

    #[sqlx_testcontainers::test]
    async fn test_add_item(mut conn: sqlx::PgConnection) {
        let repo = SqlxMagicListRepository { pool: Pool::connect_lazy("postgres://unused").unwrap() };
        let mut tx = conn.begin().await.unwrap();
        let user_id = create_user(&mut tx, "alice").await;
        let ml_id = create_magic_list(&mut tx, user_id, "PERSONAL", None).await;

        let command = CreateMagicListItemCommand {
            title: "Buy milk".to_string(),
            content: Some("2L whole milk".to_string()),
            checked: Some(false),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()),
            status: Some(MagicListItemStatus::Todo),
        };
        let result = repo.add_item_inner(&mut *tx, ml_id, command).await;
        assert!(result.is_ok());

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM magic_list_item WHERE magic_list_id = $1")
            .bind(ml_id).fetch_one(&mut *tx).await.unwrap();
        assert_eq!(count.0, 1);
    }
}
