use serde::Deserialize;
use crate::domains::bank_account::domain::bank_account_filters::BankAccountFilter;

#[derive(Deserialize)]
pub struct RequestFilter {
    pub date: String
}

impl RequestFilter {
    pub fn to_bank_account_filter(&self) -> BankAccountFilter {
        let parts: Vec<&str> = self.date.split('-').collect();
        let year: i32 = parts[0].parse().unwrap();
        let month: u8 = parts[1].parse().unwrap();
        BankAccountFilter {
            month,
            year,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_map_request_filter_to_bank_account_filter() {
        let request = RequestFilter {
            date: "2026-03".to_string(),
        };

        let filter = request.to_bank_account_filter();

        assert_eq!(filter.year, 2026);
        assert_eq!(filter.month, 3);
    }
}