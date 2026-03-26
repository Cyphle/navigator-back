use async_trait::async_trait;
use sqlx::{Error, Postgres, Transaction};
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

        let user_id_username: Vec<(i32, String)> = sqlx::query_as(
            "SELECT id, username, email FROM users WHERE username = ANY($1) OR email = ANY($1)",
        )
            .bind(usernames)
            .fetch_all(&mut **tx)
            .await?;

        let creator_id = &user_id_username
            .iter()
            .find(|(_, uname)| uname == username)
            .map(|(id, _)| *id)
            .ok_or(Error::RowNotFound)?;

        let family_id: (i32,) = sqlx::query_as(
            "INSERT INTO families (name, creator_id, active) VALUES ($1, $2, $3) RETURNING id",
        )
            .bind(&command.name)
            .bind(creator_id)
            .bind(true)
            .fetch_one(&mut **tx)
            .await?;

        sqlx::query(
            "INSERT INTO family_members (family_id, user_id, relation, is_admin) VALUES ($1, $2, $3, $4)",
        )
            .bind(family_id.0)
            .bind(creator_id)
            .bind(command.creator_relation.as_str())
            .bind(true)
            .execute(&mut **tx)
            .await?;

        let (user_ids, relations, is_admins): (Vec<i32>, Vec<&str>, Vec<bool>) = command.members.iter()
            .map(|member| {
                user_id_username.iter()
                    .find(|(_, uname)| uname == &member.username_or_email)
                    .map(|(id, _)| (*id, member.relation.as_str(), member.is_admin))
                    .ok_or(Error::RowNotFound)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .fold((Vec::new(), Vec::new(), Vec::new()), |mut acc, (id, rel, admin)| {
                acc.0.push(id);
                acc.1.push(rel);
                acc.2.push(admin);
                acc
            });

        sqlx::query(
            "INSERT INTO family_members (family_id, user_id, relation, is_admin)
     SELECT $1, * FROM UNNEST($2::int[], $3::text[], $4::boolean[])",
        )
            .bind(family_id.0)
            .bind(&user_ids)
            .bind(&relations)
            .bind(&is_admins)
            .execute(&mut **tx)
            .await?;

        Ok(family_id.0)
    }
}
