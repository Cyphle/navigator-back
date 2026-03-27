use async_trait::async_trait;
use sqlx::{Error, Postgres, Transaction};
use log::error;
use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
use crate::domains::family::repositories::family_entity::FamilyEntity;
use crate::domains::family::repositories::family_repository::FamilyRepository;

pub struct SqlxFamilyRepository;

#[async_trait]
impl<'a> FamilyRepository<Transaction<'a, Postgres>> for SqlxFamilyRepository {
    async fn get_families_for(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        username: &str,
    ) -> Result<Vec<FamilyEntity>, Error> {
        let families = sqlx::query_as::<sqlx::Postgres, FamilyEntity>(
            "SELECT f.id, f.name FROM families f \
            INNER JOIN family_members fm ON f.id = fm.family_id \
            INNER JOIN users u ON fm.user_id = u.id \
            WHERE u.username = $1",
        )
            .bind(username)
            .fetch_all(&mut **tx)
            .await?;

        Ok(families)
    }

    async fn get_family_by_name(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        username: &str,
        name: &str,
    ) -> Result<FamilyEntity, Error> {
        let family = sqlx::query_as::<sqlx::Postgres, FamilyEntity>(
            "SELECT f.id, f.name FROM families f \
            INNER JOIN family_members fm ON f.id = fm.family_id \
            INNER JOIN users u ON fm.user_id = u.id \
            WHERE u.username = $1 \
            AND LOWER(f.name) = LOWER($2)",
        )
            .bind(username)
            .bind(name)
            .fetch_one(&mut **tx)
            .await?;

        Ok(family)
    }

    async fn create_family(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        username: &str,
        command: &CreateFamilyCommand
    ) -> Result<i32, Error> {
        let mut usernames: Vec<String> = command.members.iter().map(|m| m.username_or_email.clone()).collect();
        usernames.push(username.to_string());

        let user_ids = self.get_user_ids(tx, usernames).await?;
        let creator_id = self.get_creator_id(username, &user_ids)?;
        let family_id = self.insert_family_in_database(tx, &command.name, creator_id).await?;
        self.insert_creator_in_database(tx, family_id, creator_id, command.creator_relation.as_str()).await?;
        let (user_ids, relations, is_admins) = self.insert_unknown_family_members(tx, command, &user_ids).await?;
        self.insert_known_family_members(tx, family_id, &user_ids, &relations, &is_admins).await?;

        Ok(family_id)
    }
}

impl SqlxFamilyRepository {
    async fn get_user_ids<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        usernames: Vec<String>,
    ) -> Result<Vec<(i32, Option<String>, Option<String>)>, Error> {
        let user_id_username: Vec<(i32, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT id, username, email FROM users WHERE username = ANY($1) OR email = ANY($1)",
        )
            .bind(usernames)
            .fetch_all(&mut **tx)
            .await?;
        Ok(user_id_username)
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

    async fn insert_family_in_database<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        name: &str,
        creator_id: i32,
    ) -> Result<i32, Error> {
        let family_id: (i32,) = sqlx::query_as(
            "INSERT INTO families (name, creator_id, active) VALUES ($1, $2, $3) RETURNING id",
        )
            .bind(name)
            .bind(creator_id)
            .bind(true)
            .fetch_one(&mut **tx)
            .await?;
        Ok(family_id.0)
    }

    async fn insert_creator_in_database<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
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
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn insert_unknown_family_members<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        command: &CreateFamilyCommand,
        user_id_username: &[(i32, Option<String>, Option<String>)],
    ) -> Result<(Vec<i32>, Vec<String>, Vec<bool>), Error> {
        let (mut user_ids, mut relations, mut is_admins): (Vec<i32>, Vec<String>, Vec<bool>) = (Vec::new(), Vec::new(), Vec::new());

        for member in &command.members {
            let existing_user = user_id_username.iter()
                .find(|(_, uname, email)| {
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
                    .fetch_one(&mut **tx)
                    .await?;
                new_user_id.0
            };

            user_ids.push(user_id);
            relations.push(member.relation.as_str().to_string());
            is_admins.push(member.is_admin);
        }
        Ok((user_ids, relations, is_admins))
    }

    async fn insert_known_family_members<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
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
                .execute(&mut **tx)
                .await?;
        }
        Ok(())
    }
}
