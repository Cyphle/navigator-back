use std::collections::HashMap;
use async_trait::async_trait;
use sqlx::{Error, Postgres, Transaction};
use log::error;
use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
use crate::domains::family::domain::family::{Family, FamilyMember};
use crate::domains::family::domain::family_relation::FamilyRelation;
use crate::domains::family::repositories::family_entity::FamilyEntity;
use crate::config::actix::AsPgConn;
use crate::domains::family::domain::family_repository::{DynFamilyRepository, FamilyRepository};

use sqlx::PgConnection;

pub struct SqlxFamilyRepository;

impl SqlxFamilyRepository {
    async fn get_families_for_inner(
        &self,
        conn: &mut PgConnection,
        username: &str,
    ) -> Result<Vec<Family>, Error> {
        let families_entities = sqlx::query_as::<Postgres, FamilyEntity>(
            "SELECT f.id, f.name, f.creator_id, f.active, u.id as user_id, u.username, fm.relation, fm.is_admin FROM families f \
            INNER JOIN family_members fm    ON f.id = fm.family_id \
            INNER JOIN users u              ON fm.user_id = u.id \
            WHERE f.id IN ( \
                SELECT f2.id FROM families f2 \
                INNER JOIN users u2 ON f2.creator_id = u2.id \
                WHERE u2.username = $1 \
            )",
        )
            .bind(username)
            .fetch_all(&mut *conn)
            .await?;

        let mut families_map: HashMap<i32, Vec<FamilyEntity>> = HashMap::new();
        for entity in families_entities {
            families_map.entry(entity.id).or_default().push(entity);
        }

        Ok(families_map.into_values().map(|e| self.to_family(e)).collect())
    }

    async fn get_family_by_name_inner(
        &self,
        conn: &mut PgConnection,
        username: &str,
        name: &str,
    ) -> Result<Family, Error> {
        let families_entities = sqlx::query_as::<Postgres, FamilyEntity>(
            "SELECT f.id, f.name, f.creator_id, f.active, u.id as user_id, u.username, fm.relation, fm.is_admin FROM families f \
            INNER JOIN family_members fm    ON f.id = fm.family_id \
            INNER JOIN users u              ON fm.user_id = u.id \
            WHERE f.id IN ( \
                SELECT f2.id FROM families f2 \
                INNER JOIN users u2 ON f2.creator_id = u2.id \
                WHERE u2.username = $1 \
                AND LOWER(f2.name) = LOWER($2) \
            )",
        )
            .bind(username)
            .bind(name)
            .fetch_all(&mut *conn)
            .await?;

        if families_entities.is_empty() {
            return Err(Error::RowNotFound);
        }

        Ok(self.to_family(families_entities))
    }

    async fn create_family_inner(
        &self,
        conn: &mut PgConnection,
        username: &str,
        command: &CreateFamilyCommand,
    ) -> Result<Family, Error> {
        let mut usernames: Vec<String> = command.members.iter().map(|m| m.username_or_email.clone()).collect();
        usernames.push(username.to_string());

        let user_ids = self.get_user_ids(conn, usernames).await?;
        let creator_id = self.get_creator_id(username, &user_ids)?;
        let family_id = self.insert_family_in_database(conn, &command.name, creator_id).await?;
        self.insert_creator_in_database(conn, family_id, creator_id, command.creator_relation.as_str()).await?;
        let (user_ids, relations, is_admins) = self.insert_unknown_family_members(conn, command, &user_ids).await?;
        self.insert_known_family_members(conn, family_id, &user_ids, &relations, &is_admins).await?;

        let families_entities = sqlx::query_as::<Postgres, FamilyEntity>(
            "SELECT f.id, f.name, f.creator_id, f.active, u.id as user_id, u.username, fm.relation, fm.is_admin FROM families f \
            INNER JOIN family_members fm    ON f.id = fm.family_id \
            INNER JOIN users u              ON fm.user_id = u.id \
            WHERE f.id = $1",
        )
            .bind(family_id)
            .fetch_all(&mut *conn)
            .await?;

        Ok(self.to_family(families_entities))
    }

    async fn get_user_ids(
        &self,
        conn: &mut PgConnection,
        usernames: Vec<String>,
    ) -> Result<Vec<(i32, Option<String>, Option<String>)>, Error> {
        Ok(sqlx::query_as(
            "SELECT id, username, email FROM users WHERE username = ANY($1) OR email = ANY($1)",
        )
            .bind(usernames)
            .fetch_all(&mut *conn)
            .await?)
    }

    fn get_creator_id(
        &self,
        username: &str,
        user_id_username: &[(i32, Option<String>, Option<String>)],
    ) -> Result<i32, Error> {
        user_id_username
            .iter()
            .find(|(_, uname, email)| {
                let matches_uname = uname.as_ref().map_or(false, |u| u.to_lowercase() == username.to_lowercase());
                let matches_email = email.as_ref().map_or(false, |e| e.to_lowercase() == username.to_lowercase());
                matches_uname || matches_email
            })
            .map(|(id, _, _)| *id)
            .ok_or_else(|| {
                error!("Creator not found in users: {}", username);
                Error::RowNotFound
            })
    }

    async fn insert_family_in_database(
        &self,
        conn: &mut PgConnection,
        name: &str,
        creator_id: i32,
    ) -> Result<i32, Error> {
        let family_id: (i32,) = sqlx::query_as(
            "INSERT INTO families (name, creator_id, active) VALUES ($1, $2, $3) RETURNING id",
        )
            .bind(name)
            .bind(creator_id)
            .bind(true)
            .fetch_one(&mut *conn)
            .await?;
        Ok(family_id.0)
    }

    async fn insert_creator_in_database(
        &self,
        conn: &mut PgConnection,
        family_id: i32,
        creator_id: i32,
        relation: &str,
    ) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO family_members (family_id, user_id, relation, is_admin) VALUES ($1, $2, $3, $4)",
        )
            .bind(family_id)
            .bind(creator_id)
            .bind(relation)
            .bind(true)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    async fn insert_unknown_family_members(
        &self,
        conn: &mut PgConnection,
        command: &CreateFamilyCommand,
        user_id_username: &[(i32, Option<String>, Option<String>)],
    ) -> Result<(Vec<i32>, Vec<String>, Vec<bool>), Error> {
        let (mut user_ids, mut relations, mut is_admins) = (Vec::new(), Vec::new(), Vec::new());

        for member in &command.members {
            let existing_user = user_id_username.iter().find(|(_, uname, email)| {
                let matches_uname = uname.as_ref().map_or(false, |u| u.to_lowercase() == member.username_or_email.to_lowercase());
                let matches_email = email.as_ref().map_or(false, |e| e.to_lowercase() == member.username_or_email.to_lowercase());
                matches_uname || matches_email
            });

            let user_id = if let Some((id, _, _)) = existing_user {
                *id
            } else {
                let new_user_id: (i32,) = sqlx::query_as(
                    "INSERT INTO users (username, email, first_name, last_name) VALUES ($1, $2, $3, $4) RETURNING id",
                )
                    .bind(&member.username_or_email)
                    .bind(&member.username_or_email)
                    .bind("Coincoin")
                    .bind("Mon canard")
                    .fetch_one(&mut *conn)
                    .await?;
                new_user_id.0
            };

            user_ids.push(user_id);
            relations.push(member.relation.as_str().to_string());
            is_admins.push(member.is_admin);
        }
        Ok((user_ids, relations, is_admins))
    }

    async fn insert_known_family_members(
        &self,
        conn: &mut PgConnection,
        family_id: i32,
        user_ids: &[i32],
        relations: &[String],
        is_admins: &[bool],
    ) -> Result<(), Error> {
        if !user_ids.is_empty() {
            sqlx::query(
                "INSERT INTO family_members (family_id, user_id, relation, is_admin)
                 SELECT $1, * FROM UNNEST($2::int[], $3::text[], $4::boolean[])",
            )
                .bind(family_id)
                .bind(user_ids)
                .bind(relations)
                .bind(is_admins)
                .execute(&mut *conn)
                .await?;
        }
        Ok(())
    }

    fn to_family(&self, families: Vec<FamilyEntity>) -> Family {
        let creator_username = families
            .iter()
            .find(|f| f.creator_id == f.user_id)
            .map(|f| f.username.clone())
            .unwrap();

        let members = families.iter().map(|f| FamilyMember {
            username: f.username.clone(),
            relation: FamilyRelation::from_str(&f.relation),
            is_admin: f.is_admin,
        }).collect();

        Family {
            id: families[0].id,
            name: families[0].name.clone(),
            creator_username,
            members,
            active: families[0].active,
        }
    }
}

#[async_trait]
impl<'a> FamilyRepository<Transaction<'a, Postgres>> for SqlxFamilyRepository {
    async fn get_families_for(&self, tx: &mut Transaction<'a, Postgres>, username: &str) -> Result<Vec<Family>, Error> {
        self.get_families_for_inner(&mut **tx, username).await
    }
    async fn get_family_by_name(&self, tx: &mut Transaction<'a, Postgres>, username: &str, name: &str) -> Result<Family, Error> {
        self.get_family_by_name_inner(&mut **tx, username, name).await
    }
    async fn create_family(&self, tx: &mut Transaction<'a, Postgres>, username: &str, command: &CreateFamilyCommand) -> Result<Family, Error> {
        self.create_family_inner(&mut **tx, username, command).await
    }
}

#[async_trait]
impl DynFamilyRepository for SqlxFamilyRepository {
    async fn get_families_for(&self, conn: &mut dyn AsPgConn, username: &str) -> Result<Vec<Family>, Error> {
        self.get_families_for_inner(conn.as_pg_conn(), username).await
    }
    async fn get_family_by_name(&self, conn: &mut dyn AsPgConn, username: &str, name: &str) -> Result<Family, Error> {
        self.get_family_by_name_inner(conn.as_pg_conn(), username, name).await
    }
    async fn create_family(&self, conn: &mut dyn AsPgConn, username: &str, command: &CreateFamilyCommand) -> Result<Family, Error> {
        self.create_family_inner(conn.as_pg_conn(), username, command).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Connection;
    use crate::domains::family::domain::create_family_command::CreateFamilyMemberCommand;

    async fn create_user(tx: &mut Transaction<'_, Postgres>, username: &str, email: &str) -> i32 {
        let row: (i32,) = sqlx::query_as("INSERT INTO users (username, email, first_name, last_name) VALUES ($1, $2, $3, $4) RETURNING id")
            .bind(username).bind(email).bind("First").bind("Last")
            .fetch_one(&mut **tx).await.unwrap();
        row.0
    }

    #[sqlx_testcontainers::test]
    async fn test_create_family(mut conn: sqlx::PgConnection) {
        let repo = SqlxFamilyRepository;
        let mut tx = conn.begin().await.unwrap();
        create_user(&mut tx, "alice", "alice@example.com").await;
        let command = CreateFamilyCommand {
            name: "The Alices".to_string(),
            creator_relation: FamilyRelation::Parent,
            members: vec![CreateFamilyMemberCommand {
                username_or_email: "bob@example.com".to_string(),
                relation: FamilyRelation::Child,
                is_admin: false,
            }],
        };
        let family = repo.create_family_inner(&mut *tx, "alice", &command).await.unwrap();
        assert_eq!(family.name, "The Alices");
        assert_eq!(family.creator_username, "alice");
        assert_eq!(family.members.len(), 2);
    }

    #[sqlx_testcontainers::test]
    async fn test_get_families_for(mut conn: sqlx::PgConnection) {
        let repo = SqlxFamilyRepository;
        let mut tx = conn.begin().await.unwrap();
        let alice_id = create_user(&mut tx, "alice", "alice@example.com").await;
        let family_id: (i32,) = sqlx::query_as("INSERT INTO families (name, creator_id) VALUES ($1, $2) RETURNING id")
            .bind("Alice Family").bind(alice_id).fetch_one(&mut *tx).await.unwrap();
        sqlx::query("INSERT INTO family_members (family_id, user_id, relation, is_admin) VALUES ($1, $2, $3, $4)")
            .bind(family_id.0).bind(alice_id).bind("PARENT").bind(true)
            .execute(&mut *tx).await.unwrap();
        let families = repo.get_families_for_inner(&mut *tx, "alice").await.unwrap();
        assert_eq!(families.len(), 1);
        assert_eq!(families[0].name, "Alice Family");
    }

    #[sqlx_testcontainers::test]
    async fn test_get_family_by_name(mut conn: sqlx::PgConnection) {
        let repo = SqlxFamilyRepository;
        let mut tx = conn.begin().await.unwrap();
        create_user(&mut tx, "alice", "alice@example.com").await;
        let command = CreateFamilyCommand {
            name: "Alice Club".to_string(),
            creator_relation: FamilyRelation::Other,
            members: vec![],
        };
        repo.create_family_inner(&mut *tx, "alice", &command).await.unwrap();
        let family = repo.get_family_by_name_inner(&mut *tx, "alice", "alice club").await.unwrap();
        assert_eq!(family.name, "Alice Club");
    }
}