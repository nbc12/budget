use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub is_income: bool,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub color: String,
    pub is_income: bool,
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct RawCreateCategoryRequest {
    pub name: String,
    pub is_income: bool,
}

#[derive(Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: String,
    pub color: Option<String>,
    pub is_income: bool,
    pub is_active: bool,
}

impl CreateCategoryRequest {
    pub fn new(name: String, color: String, is_income: bool) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Category name cannot be empty".to_string());
        }
        
        Ok(Self {
            name: name.trim().to_string(),
            color,
            is_income,
            is_active: true,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonthlyBudget {
    pub id: i64,
    pub category_id: i64,
    pub month: String, // YYYY-MM
    pub limit_amount: i64, // Cents
}

#[derive(Debug, Serialize)]
pub struct CreateMonthlyBudgetRequest {
    pub category_id: i64,
    pub month: String,
    pub limit_amount: i64,
}

impl CreateMonthlyBudgetRequest {
    pub fn new(category_id: i64, month: String, limit_dollars: f64) -> Result<Self, String> {
        if limit_dollars < 0.0 {
            return Err("Limit cannot be negative".to_string());
        }
        
        // Basic month format validation (YYYY-MM)
        if month.len() != 7 || month.chars().nth(4) != Some('-') {
             return Err("Invalid month format. Expected YYYY-MM".to_string());
        }

        Ok(Self {
            category_id,
            month,
            limit_amount: (limit_dollars * 100.0).round() as i64,
        })
    }
}

// Combined View Model for the UI
#[derive(Debug, Serialize)]
pub struct CategoryBudgetView {
    pub category: Category,
    pub budget: Option<MonthlyBudget>, // None if no limit set for this month
    pub spent: i64,
    pub remaining: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_category_request_valid() {
        let req = CreateCategoryRequest::new("Groceries".to_string(), "#ffffff".to_string()).unwrap();
        assert_eq!(req.name, "Groceries");
        assert_eq!(req.color, "#ffffff");
    }

    #[test]
    fn test_create_category_request_empty() {
        assert!(CreateCategoryRequest::new("   ".to_string(), "#ffffff".to_string()).is_err());
    }
}