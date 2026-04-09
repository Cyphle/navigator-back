use crate::domains::bank_account::domain::budget::Budget;
use crate::domains::bank_account::domain::charge::Charge;
use crate::domains::bank_account::domain::credit::Credit;
use crate::domains::bank_account::domain::expense::Expense;
use crate::domains::common::big_decimal::BigDecimal;
use crate::domains::common::visibility::Visibility;
use chrono::DateTime;

#[derive(Debug, PartialEq, Clone)]
pub struct BankAccount {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub visibility: Visibility,
    pub starting_amount: BigDecimal,
    pub start_date: DateTime<chrono::Utc>,
    pub budgets: Vec<Budget>,
    pub charges: Vec<Charge>,
    pub credits: Vec<Credit>,
    pub expenses: Vec<Expense>,
}

// Calculate the remaining estimated amount at the end of the month if all budgets are completely used
/*
- Il faut le montant du compte au début de mois et tous les montant initiaux des budgets
- Plus les charges du mois
- Plus les crédits du mois
- Plus les dépenses du mois

- Mais pour avoir le montant au début de mois, il faut la balance qui n'est pas stockée donc il faut tout recalculer
-> c'est ok dans un premier temps mais ça ne l'est plus au bout d'un moment. On peut estimer qu'un compte particulier
ça n'est pas des millions de lignes par mois (sauf les gros bourges)
-> Donc il faut récupérer TOUTES les transactions.
 */

// Calculate the current remaining amount





// TODO TO continue
// #[cfg(test)]
// mod tests {
//     use actix_web::cookie::time::Month;
//     use crate::domains::common::big_decimal::BigDecimal;
//     use crate::domains::common::periodicity::Periodicity;
//     use crate::testing::domain::mock_bank_account::{a_bank_account, a_budget, a_charge, a_credit, an_expense, utc_date};
//
//     #[test]
//     fn should_sum_of_bank_account_transactions_amounts() {
//         let start_date = utc_date(2026, Month::April, 7);
//         let bank_account = a_bank_account(
//             BigDecimal::from(0.0),
//             start_date,
//             vec![
//                 a_budget(
//                     start_date,
//                     BigDecimal::from(100.0),
//                     vec![
//                         an_expense(
//                             BigDecimal::from(10.0),
//                             utc_date(2026, Month::April, 7),
//                             utc_date(2026, Month::April, 7),
//                         )
//                     ]
//                 )
//             ],
//             vec![
//                 a_charge(
//                     BigDecimal::from(20.0),
//                     utc_date(2026, Month::April, 7),
//                     Periodicity::Monthly,
//                 )
//             ],
//             vec![
//                 a_credit(
//                     BigDecimal::from(30.0),
//                     utc_date(2026, Month::April, 7),
//                 )
//             ],
//             vec![
//                 an_expense(
//                     BigDecimal::from(40.0),
//                     utc_date(2026, Month::April, 7),
//                     utc_date(2026, Month::April, 7),
//                 )
//             ],
//         );
//
//         let total = bank_account.total_transactions();
//
//         assert_eq!(total, BigDecimal::from(100.0));
//     }
//
//
//     // #[test]
//     // fn should_calculate_bank_account_start_amount_of_given_month() {
//     //     let month = chrono::NaiveDate::from_ymd_opt(2023, Month::April.number_from_month(), 1);
//     //     let bank_account = a_bank_account(
//     //
//     //     );
//     // }
// }