use crate::config::actix::AsPgConn;
use crate::domains::bank_account::domain::bank_account::{BankAccount, Charge, Credit, Expense, TransactionType};
use crate::domains::bank_account::domain::bank_account_filters::BankAccountFilter;
use crate::domains::bank_account::domain::bank_account_repository::BankAccountReadRepository;
use crate::domains::common::big_decimal::{to_big_decimal, BigDecimal, RoundingMode};
use crate::domains::common::periodicity::Periodicity;
use crate::domains::common::visibility::Visibility;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sqlx::{Error, FromRow, PgConnection, Postgres};
use std::collections::HashMap;

#[derive(Debug, FromRow)]
struct BankAccountEntity {
    ba_id: i32,
    ba_name: String,
    ba_description: Option<String>,
    ba_starting_amount: Decimal,
    ba_start_date: NaiveDate,
    t_id: Option<i32>,
    t_type: Option<String>,
    t_description: Option<String>,
    t_amount: Option<Decimal>,
    t_start_date: Option<NaiveDate>,
    t_end_date: Option<NaiveDate>,
    t_periodicity: Option<String>,
}

pub struct SqlxBankAccountReadRepository;

impl SqlxBankAccountReadRepository {
    async fn get_bank_accounts_for_inner(
        &self,
        conn: &mut PgConnection,
        username: &str,
        filter: &BankAccountFilter,
    ) -> Result<Vec<BankAccount>, Error> {
        let (first_day, last_day) = Self::month_bounds(filter)?;

        let rows = sqlx::query_as::<Postgres, BankAccountEntity>(
            r#"SELECT
                   ba.id            AS ba_id,
                   ba.name          AS ba_name,
                   ba.description   AS ba_description,
                   ba.starting_amount AS ba_starting_amount,
                   ba.start_date    AS ba_start_date,
                   t.id             AS t_id,
                   t."type"         AS t_type,
                   t.description    AS t_description,
                   t.amount         AS t_amount,
                   t.start_date     AS t_start_date,
                   t.end_date       AS t_end_date,
                   t.periodicity    AS t_periodicity
               FROM bank_accounts ba
               INNER JOIN users u ON ba.owner_id = u.id
               LEFT JOIN transactions t
                   ON t.bank_account_id = ba.id
                   AND t.start_date >= $2
                   AND t.start_date <= $3
               WHERE u.username = $1
               ORDER BY ba.id"#,
        )
        .bind(username)
        .bind(first_day)
        .bind(last_day)
        .fetch_all(&mut *conn)
        .await?;

        let mut accounts: HashMap<i32, BankAccount> = HashMap::new();
        let mut account_order: Vec<i32> = Vec::new();

        for row in rows {
            if account_order.last() != Some(&row.ba_id) {
                account_order.push(row.ba_id);
            }

            let entry = accounts.entry(row.ba_id).or_insert_with(|| BankAccount {
                id: row.ba_id,
                name: row.ba_name.clone(),
                description: row.ba_description.clone().unwrap_or_default(),
                visibility: Visibility::Personal,
                starting_amount: to_big_decimal(row.ba_starting_amount),
                start_date: row.ba_start_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
                budgets: vec![],
                charges: vec![],
                credits: vec![],
                expenses: vec![],
            });

            match (&row.t_id, &row.t_type, &row.t_description, &row.t_amount, &row.t_start_date, &row.t_end_date) {
                (Some(t_id), Some(t_type), Some(t_desc), Some(t_amount), Some(t_start), Some(t_end)) => {
                    match TransactionType::try_from(t_type.as_str()) {
                        Ok(TransactionType::Expense) => entry.expenses.push(Expense {
                            id: *t_id,
                            description: t_desc.clone(),
                            amount: to_big_decimal(*t_amount),
                            expense_date: t_start.and_hms_opt(0, 0, 0).unwrap().and_utc(),
                            debit_date: t_end.and_hms_opt(0, 0, 0).unwrap().and_utc(),
                        }),
                        Ok(TransactionType::Credit) => entry.credits.push(Credit {
                            id: *t_id,
                            description: t_desc.clone(),
                            amount: to_big_decimal(*t_amount),
                            date: t_start.and_hms_opt(0, 0, 0).unwrap().and_utc(),
                        }),
                        Ok(TransactionType::Charge) => entry.charges.push(Charge {
                            id: *t_id,
                            description: t_desc.clone(),
                            amount: to_big_decimal(*t_amount),
                            date: t_start.and_hms_opt(0, 0, 0).unwrap().and_utc(),
                            periodicity: match row.t_periodicity.as_deref() {
                                Some("YEARLY") => Periodicity::Yearly,
                                _ => Periodicity::Monthly,
                            },
                        }),
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        Ok(account_order.into_iter().filter_map(|id| accounts.remove(&id)).collect())
    }

    fn month_bounds(filter: &BankAccountFilter) -> Result<(NaiveDate, NaiveDate), Error> {
        let first_day = NaiveDate::from_ymd_opt(filter.year, filter.month as u32, 1)
            .ok_or_else(|| {
                let e: Box<dyn std::error::Error + Send + Sync> =
                    format!("invalid date: year={}, month={}", filter.year, filter.month).into();
                Error::Decode(e)
            })?;
        let (next_year, next_month) = if filter.month == 12 {
            (filter.year + 1, 1u32)
        } else {
            (filter.year, filter.month as u32 + 1)
        };
        let last_day = NaiveDate::from_ymd_opt(next_year, next_month, 1)
            .and_then(|d| d.pred_opt())
            .ok_or_else(|| {
                let e: Box<dyn std::error::Error + Send + Sync> = "date overflow".to_string().into();
                Error::Decode(e)
            })?;
        Ok((first_day, last_day))
    }
}

#[async_trait]
impl BankAccountReadRepository for SqlxBankAccountReadRepository {
    async fn get_bank_accounts_for(
        &self,
        conn: &mut dyn AsPgConn,
        username: &str,
        filter: &BankAccountFilter,
    ) -> Result<Vec<BankAccount>, Error> {
        self.get_bank_accounts_for_inner(conn.as_pg_conn(), username, filter).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, Transaction};

    async fn create_user(tx: &mut Transaction<'_, Postgres>, username: &str) -> i32 {
        let row: (i32,) = sqlx::query_as(
            "INSERT INTO users (username, email, first_name, last_name) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(username)
        .bind(format!("{}@test.com", username))
        .bind("First").bind("Last")
        .fetch_one(&mut **tx).await.unwrap();
        row.0
    }

    async fn create_bank_account(tx: &mut Transaction<'_, Postgres>, owner_id: i32, name: &str, starting_amount: i32) -> i32 {
        let row: (i32,) = sqlx::query_as(
            "INSERT INTO bank_accounts (owner_id, name, starting_amount, start_date) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(owner_id).bind(name)
        .bind(Decimal::from(starting_amount))
        .bind(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
        .fetch_one(&mut **tx).await.unwrap();
        row.0
    }

    async fn insert_transaction(tx: &mut Transaction<'_, Postgres>, bank_account_id: i32, tx_type: &str, description: &str, amount: i32, date: NaiveDate) {
        sqlx::query(
            r#"INSERT INTO transactions (bank_account_id, "type", description, amount, start_date, end_date)
               VALUES ($1, $2, $3, $4, $5, $5)"#,
        )
        .bind(bank_account_id).bind(tx_type).bind(description)
        .bind(Decimal::from(amount)).bind(date)
        .execute(&mut **tx).await.unwrap();
    }

    #[sqlx_testcontainers::test]
    async fn test_returns_account_with_expenses_and_credits_for_month(mut conn: sqlx::PgConnection) {
        let repo = SqlxBankAccountReadRepository;
        let mut tx = conn.begin().await.unwrap();
        let user_id = create_user(&mut tx, "alice").await;
        let account_id = create_bank_account(&mut tx, user_id, "Main", 1000).await;
        let march = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();

        insert_transaction(&mut tx, account_id, "EXPENSE", "Rent",   500, march).await;
        insert_transaction(&mut tx, account_id, "EXPENSE", "Food",   200, march).await;
        insert_transaction(&mut tx, account_id, "CREDIT",  "Salary", 300, march).await;
        // Out of month — must not appear
        insert_transaction(&mut tx, account_id, "EXPENSE", "April",  100, NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()).await;

        let accounts = repo.get_bank_accounts_for_inner(&mut *tx, "alice", &BankAccountFilter { month: 3, year: 2026 }).await.unwrap();

        assert_eq!(accounts.len(), 1);
        let a = &accounts[0];
        assert_eq!(a.expenses.len(), 2);
        assert_eq!(a.credits.len(), 1);
        assert_eq!(a.starting_amount.to_f64(), 1000.0);

        let mut expense_amounts: Vec<f64> = a.expenses.iter().map(|e| e.amount.to_f64()).collect();
        expense_amounts.sort_by(|x, y| x.partial_cmp(y).unwrap());
        assert_eq!(expense_amounts, vec![200.0, 500.0]);
        assert_eq!(a.credits[0].amount.to_f64(), 300.0);
    }

    #[sqlx_testcontainers::test]
    async fn test_account_with_no_transactions_has_empty_collections(mut conn: sqlx::PgConnection) {
        let repo = SqlxBankAccountReadRepository;
        let mut tx = conn.begin().await.unwrap();
        let user_id = create_user(&mut tx, "alice").await;
        create_bank_account(&mut tx, user_id, "Empty", 500).await;

        let accounts = repo.get_bank_accounts_for_inner(&mut *tx, "alice", &BankAccountFilter { month: 3, year: 2026 }).await.unwrap();

        assert_eq!(accounts.len(), 1);
        assert!(accounts[0].expenses.is_empty());
        assert!(accounts[0].credits.is_empty());
        assert_eq!(accounts[0].starting_amount.to_f64(), 500.0);
    }

    #[sqlx_testcontainers::test]
    async fn test_returns_one_bank_account_per_account(mut conn: sqlx::PgConnection) {
        let repo = SqlxBankAccountReadRepository;
        let mut tx = conn.begin().await.unwrap();
        let user_id = create_user(&mut tx, "alice").await;
        create_bank_account(&mut tx, user_id, "Account A", 100).await;
        create_bank_account(&mut tx, user_id, "Account B", 200).await;

        let accounts = repo.get_bank_accounts_for_inner(&mut *tx, "alice", &BankAccountFilter { month: 3, year: 2026 }).await.unwrap();

        assert_eq!(accounts.len(), 2);
    }

    #[sqlx_testcontainers::test]
    async fn test_excludes_other_users_accounts(mut conn: sqlx::PgConnection) {
        let repo = SqlxBankAccountReadRepository;
        let mut tx = conn.begin().await.unwrap();
        let alice_id = create_user(&mut tx, "alice").await;
        let bob_id   = create_user(&mut tx, "bob").await;
        create_bank_account(&mut tx, alice_id, "Alice", 100).await;
        create_bank_account(&mut tx, bob_id,   "Bob",   200).await;

        let accounts = repo.get_bank_accounts_for_inner(&mut *tx, "alice", &BankAccountFilter { month: 3, year: 2026 }).await.unwrap();

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "Alice");
    }

    #[sqlx_testcontainers::test]
    async fn test_includes_first_and_last_day_of_month(mut conn: sqlx::PgConnection) {
        let repo = SqlxBankAccountReadRepository;
        let mut tx = conn.begin().await.unwrap();
        let user_id = create_user(&mut tx, "alice").await;
        let account_id = create_bank_account(&mut tx, user_id, "Main", 0).await;

        insert_transaction(&mut tx, account_id, "EXPENSE", "First", 10, NaiveDate::from_ymd_opt(2026, 3,  1).unwrap()).await;
        insert_transaction(&mut tx, account_id, "EXPENSE", "Last",  20, NaiveDate::from_ymd_opt(2026, 3, 31).unwrap()).await;
        insert_transaction(&mut tx, account_id, "EXPENSE", "Before", 5, NaiveDate::from_ymd_opt(2026, 2, 28).unwrap()).await;
        insert_transaction(&mut tx, account_id, "EXPENSE", "After",  5, NaiveDate::from_ymd_opt(2026, 4,  1).unwrap()).await;

        let accounts = repo.get_bank_accounts_for_inner(&mut *tx, "alice", &BankAccountFilter { month: 3, year: 2026 }).await.unwrap();

        assert_eq!(accounts[0].expenses.len(), 2);
    }
}
