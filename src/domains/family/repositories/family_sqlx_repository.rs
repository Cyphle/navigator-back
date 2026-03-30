use std::collections::HashMap;
use async_trait::async_trait;
use sqlx::{Error, Postgres, Transaction};
use log::error;
use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
use crate::domains::family::domain::family::{Family, FamilyMember};
use crate::domains::family::domain::family_relation::FamilyRelation;
use crate::domains::family::repositories::family_entity::FamilyEntity;
use crate::domains::family::domain::family_repository::FamilyRepository;

pub struct SqlxFamilyRepository;

#[async_trait]
impl<'a> FamilyRepository<Transaction<'a, Postgres>> for SqlxFamilyRepository {
    async fn get_families_for(
        &self,
        tx: &mut Transaction<'a, Postgres>,
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
            .fetch_all(&mut **tx)
            .await?;

        let mut families_map: HashMap<i32, Vec<FamilyEntity>> = HashMap::new();
        for entity in families_entities {
            families_map.entry(entity.id).or_default().push(entity);
        }

        let families = families_map
            .into_values()
            .map(|entities| self.to_family(entities))
            .collect();

        Ok(families)
    }

    async fn get_family_by_name(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        username: &str,
        name: &str,
    ) -> Result<Family, Error> {
        let families_entities = sqlx::query_as::<sqlx::Postgres, FamilyEntity>(
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
            .fetch_all(&mut **tx)
            .await?;

        if families_entities.is_empty() {
            return Err(Error::RowNotFound);
        }

        Ok(self.to_family(families_entities))
    }

    async fn create_family(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        username: &str,
        command: &CreateFamilyCommand
    ) -> Result<Family, Error> {
        let mut usernames: Vec<String> = command.members.iter().map(|m| m.username_or_email.clone()).collect();
        usernames.push(username.to_string());

        let user_ids = self.get_user_ids(tx, usernames).await?;
        let creator_id = self.get_creator_id(username, &user_ids)?;
        let family_id = self.insert_family_in_database(tx, &command.name, creator_id).await?;
        self.insert_creator_in_database(tx, family_id, creator_id, command.creator_relation.as_str()).await?;
        let (user_ids, relations, is_admins) = self.insert_unknown_family_members(tx, command, &user_ids).await?;
        self.insert_known_family_members(tx, family_id, &user_ids, &relations, &is_admins).await?;

        let families_entities = sqlx::query_as::<sqlx::Postgres, FamilyEntity>(
            "SELECT f.id, f.name, f.creator_id, f.active, u.id as user_id, u.username, fm.relation, fm.is_admin FROM families f \
            INNER JOIN family_members fm    ON f.id = fm.family_id \
            INNER JOIN users u              ON fm.user_id = u.id \
            WHERE f.id = $1",
        )
            .bind(family_id)
            .fetch_all(&mut **tx)
            .await?;

        Ok(self.to_family(families_entities))
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

    fn to_family(&self, families: Vec<FamilyEntity>) -> Family {
        let creator_username = families
            .iter()
            .find(|family| family.creator_id == family.user_id)
            .map(|family| family.username.clone())
            .unwrap();

        let members = families
            .iter()
            .map(|family| FamilyMember {
                username: family.username.clone(),
                relation: FamilyRelation::from_str(&family.relation),
                is_admin: family.is_admin
            })
            .collect::<Vec<FamilyMember>>();

        Family {
            id: families[0].id,
            name: families[0].name.clone(),
            creator_username,
            members,
            active: families[0].active
        }
    }
}
