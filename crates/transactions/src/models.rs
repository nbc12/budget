use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: i64,
    pub category_id: i64,
    pub card_id: Option<i64>,
    pub transaction_date: String, // 'YYYY-MM-DD'
    pub amount: i64,             // Cents
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateTransactionRequest {
    category_id: i64,
    card_id: Option<i64>,
    transaction_date: String,
    amount: i64,
    notes: Option<String>,
}

#[derive(Deserialize)]
pub struct RawCreateTransactionRequest {
    pub category_id: i64,
    pub card_id: Option<String>,
    pub transaction_date: String,
    pub amount_dollars: f64,
    pub notes: Option<String>,
}

impl CreateTransactionRequest {
    pub fn new(
        category_id: i64,
        card_id: Option<i64>,
        transaction_date: String,
        amount_dollars: f64,
        is_income: bool,
        notes: Option<String>,
    ) -> Result<Self, String> {
        if NaiveDate::parse_from_str(&transaction_date, "%Y-%m-%d").is_err() {
            return Err("Invalid date format, expected YYYY-MM-DD".to_string());
        }

        let mut amount = (amount_dollars.abs() * 100.0).round() as i64;
        if !is_income {
            amount = -amount;
        }

        Ok(Self {
            category_id,
            card_id,
            transaction_date,
            amount,
            notes,
        })
    }

    pub fn category_id(&self) -> i64 {
        self.category_id
    }

    pub fn card_id(&self) -> Option<i64> {
        self.card_id
    }

    pub fn transaction_date(&self) -> &str {
        &self.transaction_date
    }

    pub fn amount(&self) -> i64 {
        self.amount
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

#[derive(Debug, Serialize)]
pub struct MonthlySummary {
    pub month: String,
    pub total_income: i64,
    pub total_expenses: i64,
    pub net: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transaction_request_expense() {
        let req = CreateTransactionRequest::new(1, Some(1), "2023-10-27".into(), 45.50, false, None).unwrap();
        assert_eq!(req.amount(), -4550);
    }

    #[test]
    fn test_create_transaction_request_income() {
        let req = CreateTransactionRequest::new(1, Some(1), "2023-10-27".into(), 100.00, true, None).unwrap();
        assert_eq!(req.amount(), 10000);
    }
}
