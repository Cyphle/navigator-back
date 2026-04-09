
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TransactionType {
    Expense,
    Charge,
    Credit,
    Budget,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Expense => write!(f, "EXPENSE"),
            TransactionType::Charge  => write!(f, "CHARGE"),
            TransactionType::Credit  => write!(f, "CREDIT"),
            TransactionType::Budget  => write!(f, "BUDGET"),
        }
    }
}

impl TryFrom<&str> for TransactionType {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "EXPENSE" => Ok(TransactionType::Expense),
            "CHARGE" => Ok(TransactionType::Charge),
            "CREDIT" => Ok(TransactionType::Credit),
            "BUDGET" => Ok(TransactionType::Budget),
            _ => Err(()),
        }
    }
}