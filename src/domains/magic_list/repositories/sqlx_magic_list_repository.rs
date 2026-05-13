use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::common::visibility::Visibility;
use crate::domains::magic_list::domain::create_magic_list_command::CreateMagicListCommand;
use crate::domains::magic_list::domain::create_magic_list_item_command::CreateMagicListItemCommand;
use crate::domains::magic_list::domain::magic_list::MagicList;
use crate::domains::magic_list::domain::magic_list_repository::MagicListRepository;
use crate::domains::magic_list::domain::magic_list_summary::MagicListSummary;
use crate::domains::magic_list::domain::magic_list_type::MagicListType;
use crate::domains::magic_list::domain::update_magic_list_item_command::UpdateMagicListItemCommand;
use async_trait::async_trait;
use sqlx::{PgConnection, Pool, Postgres};

#[derive(sqlx::FromRow)]
struct MagicListRow {
    id: i32,
    name: String,
    list_type: String,
    owner_username: String,
    visibility: String,
    family_id: Option<i32>,
}

#[derive(sqlx::FromRow)]
struct MagicListSummaryRow {
    id: i32,
    name: String,
    visibility: String,
    magic_list_type: String,
    family_id: Option<i32>,
    item_count: i64,
}

pub struct SqlxMagicListRepository {
    pub pool: Pool<Postgres>,
}

impl SqlxMagicListRepository {
    async fn find_by_id_inner(&self, conn: &mut PgConnection, magic_list_id: i32) -> Result<MagicList, RepositoryError> {
        let row: MagicListRow = sqlx::query_as(
            "SELECT ml.id, ml.name, ml.type as list_type, u.username as owner_username, ml.visibility, ml.family_id \
             FROM magic_list ml \
             JOIN users u ON ml.owner_id = u.id \
             WHERE ml.id = $1",
        )
        .bind(magic_list_id)
        .fetch_one(&mut *conn)
        .await?;

        Ok(MagicList {
            id: row.id,
            name: row.name,
            list_type: MagicListType::from_str(&row.list_type),
            owner_username: row.owner_username,
            visibility: match row.visibility.as_str() {
                "SHARED" => Visibility::Shared,
                _ => Visibility::Personal,
            },
            family_id: row.family_id,
        })
    }

    async fn get_summary_for_user_and_family_inner(&self, conn: &mut PgConnection, username: &str, family_id: i32) -> Result<Vec<MagicListSummary>, RepositoryError> {
        let rows: Vec<MagicListSummaryRow> = sqlx::query_as(
            "SELECT ml.id, ml.name, ml.visibility, ml.type as magic_list_type, ml.family_id, \
                    COUNT(mli.id) as item_count \
             FROM magic_list ml \
             JOIN users u ON ml.owner_id = u.id \
             LEFT JOIN magic_list_item mli ON mli.magic_list_id = ml.id \
             WHERE ml.family_id = $2 \
               AND ( \
                   (ml.visibility = 'PERSONAL' AND u.username = $1) \
                   OR (ml.visibility = 'SHARED') \
               ) \
             GROUP BY ml.id, ml.name, ml.visibility, ml.type, ml.family_id",
        )
        .bind(username)
        .bind(family_id)
        .fetch_all(&mut *conn)
        .await?;

        Ok(rows.into_iter().map(|row| MagicListSummary {
            id: row.id,
            name: row.name,
            visibility: match row.visibility.as_str() {
                "SHARED" => Visibility::Shared,
                _ => Visibility::Personal,
            },
            magic_list_type: MagicListType::from_str(&row.magic_list_type),
            family_id: row.family_id,
            item_count: row.item_count,
        }).collect())
    }

    async fn add_item_inner(&self, conn: &mut PgConnection, magic_list_id: i32, command: CreateMagicListItemCommand) -> Result<(), RepositoryError> {
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
        .await?;

        Ok(())
    }

    async fn update_item_inner(&self, conn: &mut PgConnection, magic_list_id: i32, item_id: i32, command: UpdateMagicListItemCommand) -> Result<(), RepositoryError> {
        sqlx::query(
            "UPDATE magic_list_item \
             SET title = COALESCE($1, title), \
                 content = COALESCE($2, content), \
                 checked = COALESCE($3, checked), \
                 due_date = COALESCE($4, due_date), \
                 status = COALESCE($5, status), \
                 updated_at = NOW() \
             WHERE id = $6 AND magic_list_id = $7"
        )
        .bind(&command.title)
        .bind(&command.content)
        .bind(command.checked)
        .bind(command.due_date)
        .bind(command.status.as_ref().map(|s| s.to_string()))
        .bind(item_id)
        .bind(magic_list_id)
        .execute(&mut *conn)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl MagicListRepository for SqlxMagicListRepository {
    async fn create(&self, username: &str, command: CreateMagicListCommand) -> Result<(), RepositoryError> {
        let owner_id: (i32,) = sqlx::query_as(
            "SELECT id FROM users WHERE username = $1",
        )
            .bind(username)
            .fetch_one(&self.pool)
            .await?;

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
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, magic_list_id: i32) -> Result<MagicList, RepositoryError> {
        let mut conn = self.pool.acquire().await?;
        self.find_by_id_inner(&mut *conn, magic_list_id).await
    }

    async fn get_summary_for_user_and_family(&self, username: &str, family_id: i32) -> Result<Vec<MagicListSummary>, RepositoryError> {
        let mut conn = self.pool.acquire().await?;
        self.get_summary_for_user_and_family_inner(&mut *conn, username, family_id).await
    }

    async fn add_item(&self, magic_list_id: i32, command: CreateMagicListItemCommand) -> Result<(), RepositoryError> {
        let mut conn = self.pool.acquire().await?;
        self.add_item_inner(&mut *conn, magic_list_id, command).await
    }

    async fn update_item(&self, magic_list_id: i32, item_id: i32, command: UpdateMagicListItemCommand) -> Result<(), RepositoryError> {
        let mut conn = self.pool.acquire().await?;
        self.update_item_inner(&mut *conn, magic_list_id, item_id, command).await
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

    async fn add_item_to_list(tx: &mut Transaction<'_, Postgres>, magic_list_id: i32, title: &str) {
        sqlx::query("INSERT INTO magic_list_item (magic_list_id, title) VALUES ($1, $2)")
            .bind(magic_list_id).bind(title)
            .execute(&mut **tx).await.unwrap();
    }

    #[sqlx_testcontainers::test]
    async fn test_get_summary_returns_personal_and_shared_lists(mut conn: sqlx::PgConnection) {
        let repo = SqlxMagicListRepository { pool: Pool::connect_lazy("postgres://unused").unwrap() };
        let mut tx = conn.begin().await.unwrap();
        let alice_id = create_user(&mut tx, "alice").await;
        let bob_id = create_user(&mut tx, "bob").await;
        let family_id = create_family_with_member(&mut tx, alice_id, bob_id).await;

        let shared_id = create_magic_list(&mut tx, alice_id, "SHARED", Some(family_id)).await;
        add_item_to_list(&mut tx, shared_id, "Item 1").await;
        add_item_to_list(&mut tx, shared_id, "Item 2").await;

        let personal_id = create_magic_list(&mut tx, bob_id, "PERSONAL", Some(family_id)).await;
        add_item_to_list(&mut tx, personal_id, "Item 3").await;

        let result = repo.get_summary_for_user_and_family_inner(&mut *tx, "bob", family_id).await.unwrap();

        assert_eq!(result.len(), 2);
        let shared = result.iter().find(|s| s.id == shared_id).unwrap();
        assert_eq!(shared.visibility, Visibility::Shared);
        assert_eq!(shared.item_count, 2);
        let personal = result.iter().find(|s| s.id == personal_id).unwrap();
        assert_eq!(personal.visibility, Visibility::Personal);
        assert_eq!(personal.item_count, 1);
    }

    #[sqlx_testcontainers::test]
    async fn test_get_summary_excludes_other_users_personal_lists(mut conn: sqlx::PgConnection) {
        let repo = SqlxMagicListRepository { pool: Pool::connect_lazy("postgres://unused").unwrap() };
        let mut tx = conn.begin().await.unwrap();
        let alice_id = create_user(&mut tx, "alice").await;
        let bob_id = create_user(&mut tx, "bob").await;
        let family_id = create_family_with_member(&mut tx, alice_id, bob_id).await;

        create_magic_list(&mut tx, alice_id, "PERSONAL", Some(family_id)).await;

        let result = repo.get_summary_for_user_and_family_inner(&mut *tx, "bob", family_id).await.unwrap();

        assert!(result.is_empty());
    }

    #[sqlx_testcontainers::test]
    async fn test_get_summary_returns_empty_for_no_lists(mut conn: sqlx::PgConnection) {
        let repo = SqlxMagicListRepository { pool: Pool::connect_lazy("postgres://unused").unwrap() };
        let mut tx = conn.begin().await.unwrap();
        create_user(&mut tx, "alice").await;

        let result = repo.get_summary_for_user_and_family_inner(&mut *tx, "alice", 999).await.unwrap();

        assert!(result.is_empty());
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

    #[sqlx_testcontainers::test]
    async fn test_update_item(mut conn: sqlx::PgConnection) {
        let repo = SqlxMagicListRepository { pool: Pool::connect_lazy("postgres://unused").unwrap() };
        let mut tx = conn.begin().await.unwrap();
        let user_id = create_user(&mut tx, "alice").await;
        let ml_id = create_magic_list(&mut tx, user_id, "PERSONAL", None).await;

        let add_command = CreateMagicListItemCommand {
            title: "Original".to_string(),
            content: Some("Original content".to_string()),
            checked: Some(false),
            due_date: None,
            status: Some(MagicListItemStatus::Todo),
        };
        repo.add_item_inner(&mut *tx, ml_id, add_command).await.unwrap();

        let item_id: (i32,) = sqlx::query_as("SELECT id FROM magic_list_item WHERE magic_list_id = $1")
            .bind(ml_id).fetch_one(&mut *tx).await.unwrap();

        let update_command = UpdateMagicListItemCommand {
            title: Some("Updated".to_string()),
            content: None,
            checked: Some(true),
            due_date: None,
            status: Some(MagicListItemStatus::Done),
        };
        repo.update_item_inner(&mut *tx, ml_id, item_id.0, update_command).await.unwrap();

        let row: (String, String, bool, String) = sqlx::query_as(
            "SELECT title, content, checked, status FROM magic_list_item WHERE id = $1"
        ).bind(item_id.0).fetch_one(&mut *tx).await.unwrap();

        assert_eq!(row.0, "Updated");
        assert_eq!(row.1, "Original content");
        assert!(row.2);
        assert_eq!(row.3, "DONE");
    }
}
