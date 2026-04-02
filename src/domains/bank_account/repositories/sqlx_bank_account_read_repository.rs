use crate::config::actix::AsPgConn;
use crate::domains::bank_account::domain::bank_account::{BankAccount, Budget, Charge, Credit, Expense, TransactionType};
use crate::domains::bank_account::domain::bank_account_filters::BankAccountFilter;
use crate::domains::bank_account::domain::bank_account_repository::BankAccountReadRepository;
use crate::domains::common::big_decimal::to_big_decimal;
use crate::domains::common::periodicity::Periodicity;
use crate::domains::common::visibility::Visibility;
use async_trait::async_trait;
use chrono::NaiveDate;
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
    b_id: Option<i32>,
    b_name: Option<String>,
    b_description: Option<String>,
    b_start_date: Option<NaiveDate>,
    b_end_date: Option<NaiveDate>,
    b_initial_amount: Option<Decimal>,
    t_id: Option<i32>,
    t_type: Option<String>,
    t_description: Option<String>,
    t_amount: Option<Decimal>,
    t_start_date: Option<NaiveDate>,
    t_end_date: Option<NaiveDate>,
    t_periodicity: Option<String>,
    t_budget_id: Option<i32>,
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
                   b.id             AS b_id,
                   b.name           AS b_name,
                   b.description    AS b_description,
                   b.start_date     AS b_start_date,
                   b.end_date       AS b_end_date,
                   b.initial_amount AS b_initial_amount,
                   t.id             AS t_id,
                   t."type"         AS t_type,
                   t.description    AS t_description,
                   t.amount         AS t_amount,
                   t.start_date     AS t_start_date,
                   t.end_date       AS t_end_date,
                   t.periodicity    AS t_periodicity,
                   t.budget_id      AS t_budget_id
               FROM bank_accounts ba
               INNER JOIN users u ON ba.owner_id = u.id
               LEFT JOIN budgets b ON b.bank_account_id = ba.id
               LEFT JOIN transactions t
                   ON (t.bank_account_id = ba.id OR t.budget_id = b.id)
                   AND t.start_date >= $2
                   AND t.start_date <= $3
               WHERE u.username = $1
               ORDER BY ba.id, b.id"#,
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

            let account = accounts.entry(row.ba_id).or_insert_with(|| Self::row_to_bank_account(&row));

            if let Some(budget) = Self::row_to_budget(&row) {
                if !account.budgets.iter().any(|b| b.id == budget.id) {
                    account.budgets.push(budget);
                }
            }

            Self::add_transaction(&row, account);
        }

        Ok(account_order.into_iter().filter_map(|id| accounts.remove(&id)).collect())
    }

    fn add_transaction(row: &BankAccountEntity, entry: &mut BankAccount) {
        if let (Some(t_id), Some(t_type_str), Some(t_desc), Some(t_amount), Some(t_start), Some(t_end)) = (
            &row.t_id,
            &row.t_type,
            &row.t_description,
            &row.t_amount,
            &row.t_start_date,
            &row.t_end_date,
        ) {
            if let Ok(t_type) = TransactionType::try_from(t_type_str.as_str()) {
                match t_type {
                    TransactionType::Expense => {
                        let expense = Self::row_to_expense(&row, *t_id, t_desc, *t_amount, t_start, t_end);
                        if let Some(t_budget_id) = row.t_budget_id {
                            if let Some(budget) = entry.budgets.iter_mut().find(|b| b.id == t_budget_id) {
                                if !budget.expenses.iter().any(|e| e.id == expense.id) {
                                    budget.expenses.push(expense);
                                }
                            }
                        } else {
                            if !entry.expenses.iter().any(|e| e.id == expense.id) {
                                entry.expenses.push(expense);
                            }
                        }
                    }
                    TransactionType::Budget => {
                        let expense = Self::row_to_expense(&row, *t_id, t_desc, *t_amount, t_start, t_end);
                        if let Some(t_budget_id) = row.t_budget_id {
                            if let Some(budget) = entry.budgets.iter_mut().find(|b| b.id == t_budget_id) {
                                if !budget.expenses.iter().any(|e| e.id == expense.id) {
                                    budget.expenses.push(expense);
                                }
                            }
                        }
                    }
                    TransactionType::Credit => {
                        let credit = Self::row_to_credit(*t_id, t_desc, *t_amount, t_start);
                        if !entry.credits.iter().any(|c| c.id == credit.id) {
                            entry.credits.push(credit);
                        }
                    }
                    TransactionType::Charge => {
                        let charge = Self::row_to_charge(&row, *t_id, t_desc, *t_amount, t_start);
                        if !entry.charges.iter().any(|c| c.id == charge.id) {
                            entry.charges.push(charge);
                        }
                    }
                }
            }
        }
    }

    fn row_to_bank_account(row: &BankAccountEntity) -> BankAccount {
        BankAccount {
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
        }
    }

    fn row_to_budget(row: &BankAccountEntity) -> Option<Budget> {
        match (
            &row.b_id,
            &row.b_name,
            &row.b_description,
            &row.b_start_date,
            &row.b_initial_amount,
        ) {
            (Some(b_id), Some(b_name), Some(b_desc), Some(b_start), Some(b_init)) => Some(Budget {
                id: *b_id,
                name: b_name.clone(),
                description: b_desc.clone(),
                start_date: b_start.and_hms_opt(0, 0, 0).unwrap().and_utc(),
                end_date: row.b_end_date.map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc()),
                initial_amount: to_big_decimal(*b_init),
                expenses: vec![],
            }),
            _ => None,
        }
    }

    fn row_to_expense(
        _row: &BankAccountEntity,
        id: i32,
        description: &str,
        amount: Decimal,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> Expense {
        Expense {
            id,
            description: description.to_string(),
            amount: to_big_decimal(amount),
            expense_date: start_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            debit_date: end_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        }
    }

    fn row_to_credit(id: i32, description: &str, amount: Decimal, date: &NaiveDate) -> Credit {
        Credit {
            id,
            description: description.to_string(),
            amount: to_big_decimal(amount),
            date: date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        }
    }

    fn row_to_charge(
        row: &BankAccountEntity,
        id: i32,
        description: &str,
        amount: Decimal,
        date: &NaiveDate,
    ) -> Charge {
        Charge {
            id,
            description: description.to_string(),
            amount: to_big_decimal(amount),
            date: date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
            periodicity: match row.t_periodicity.as_deref() {
                Some("YEARLY") => Periodicity::Yearly,
                _ => Periodicity::Monthly,
            },
        }
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
    use crate::domains::common::big_decimal::bd;
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

    async fn insert_budget(tx: &mut Transaction<'_, Postgres>, bank_account_id: i32, name: &str, description: &str, initial_amount: i32, start_date: NaiveDate) -> i32 {
        let row: (i32,) = sqlx::query_as(
            "INSERT INTO budgets (bank_account_id, name, description, start_date, initial_amount) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(bank_account_id).bind(name).bind(description)
        .bind(start_date)
        .bind(Decimal::from(initial_amount))
        .fetch_one(&mut **tx).await.unwrap();
        row.0
    }

    async fn insert_budget_expense(tx: &mut Transaction<'_, Postgres>, bank_account_id: i32, budget_id: i32, description: &str, amount: i32, date: NaiveDate) {
        sqlx::query(
            r#"INSERT INTO transactions (bank_account_id, budget_id, "type", description, amount, start_date, end_date)
               VALUES ($1, $2, 'BUDGET', $3, $4, $5, $5)"#,
        )
        .bind(bank_account_id).bind(budget_id).bind(description)
        .bind(Decimal::from(amount)).bind(date)
        .execute(&mut **tx).await.unwrap();
    }

    async fn insert_charge(tx: &mut Transaction<'_, Postgres>, bank_account_id: i32, description: &str, amount: i32, date: NaiveDate) {
        sqlx::query(
            r#"INSERT INTO transactions (bank_account_id, "type", description, amount, start_date, end_date, periodicity)
               VALUES ($1, 'CHARGE', $2, $3, $4, $4, 'MONTHLY')"#,
        )
        .bind(bank_account_id).bind(description)
        .bind(Decimal::from(amount)).bind(date)
        .execute(&mut **tx).await.unwrap();
    }

    #[sqlx_testcontainers::test]
    async fn test_get_accounts_with_transactions_filtered_by_month(mut conn: sqlx::PgConnection) {
        let repo = SqlxBankAccountReadRepository;
        let mut tx = conn.begin().await.unwrap();
        let user_id = create_user(&mut tx, "alice").await;
        let account_id = create_bank_account(&mut tx, user_id, "Main", 1000).await;

        let march_15 = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let april_01 = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();

        // Transactions du mois de Mars
        insert_transaction(&mut tx, account_id, "EXPENSE", "Rent", 500, march_15).await;
        insert_transaction(&mut tx, account_id, "CREDIT", "Salary", 2000, march_15).await;
        insert_charge(&mut tx, account_id, "Netflix", 15, march_15).await;
        let budget_id = insert_budget(&mut tx, account_id, "Food", "Food budget", 300, march_15).await;
        insert_budget_expense(&mut tx, account_id, budget_id, "Groceries", 50, march_15).await;

        // Transactions hors mois (Avril) - NE DOIVENT PAS APPARAITRE
        insert_transaction(&mut tx, account_id, "EXPENSE", "April Expense", 100, april_01).await;
        insert_transaction(&mut tx, account_id, "CREDIT", "April Credit", 100, april_01).await;
        insert_charge(&mut tx, account_id, "April Charge", 10, april_01).await;
        insert_budget_expense(&mut tx, account_id, budget_id, "April Budget Expense", 20, april_01).await;

        let accounts = repo.get_bank_accounts_for_inner(&mut *tx, "alice", &BankAccountFilter { month: 3, year: 2026 }).await.unwrap();

        assert_eq!(accounts.len(), 1);

        let dt_march_15 = march_15.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let dt_account_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc();

        let expected_account = BankAccount {
            id: account_id,
            name: "Main".to_string(),
            description: "".to_string(),
            visibility: Visibility::Personal,
            starting_amount: bd(1000.0),
            start_date: dt_account_start,
            budgets: vec![Budget {
                id: budget_id,
                name: "Food".to_string(),
                description: "Food budget".to_string(),
                start_date: dt_march_15,
                end_date: None,
                initial_amount: bd(300.0),
                expenses: vec![Expense {
                    id: 4, 
                    description: "Groceries".to_string(),
                    amount: bd(50.0),
                    expense_date: dt_march_15,
                    debit_date: dt_march_15,
                }],
            }],
            charges: vec![Charge {
                id: 3,
                description: "Netflix".to_string(),
                amount: bd(15.0),
                date: dt_march_15,
                periodicity: Periodicity::Monthly,
            }],
            credits: vec![Credit {
                id: 2,
                description: "Salary".to_string(),
                amount: bd(2000.0),
                date: dt_march_15,
            }],
            expenses: vec![Expense {
                id: 1,
                description: "Rent".to_string(),
                amount: bd(500.0),
                expense_date: dt_march_15,
                debit_date: dt_march_15,
            }],
        };
        
        assert_eq!(accounts[0], expected_account);
    }

    #[sqlx_testcontainers::test]
    async fn test_get_accounts_returns_empty_list_when_no_accounts(mut conn: sqlx::PgConnection) {
        let repo = SqlxBankAccountReadRepository;
        let mut tx = conn.begin().await.unwrap();
        create_user(&mut tx, "bob").await;

        let accounts = repo.get_bank_accounts_for_inner(&mut *tx, "bob", &BankAccountFilter { month: 3, year: 2026 }).await.unwrap();

        assert!(accounts.is_empty());
    }
}
