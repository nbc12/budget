use crate::models::{CategoryBudgetView};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct VirtualCategory {
    pub name: String,
    pub amount: i64, // Cents
    pub is_income: bool,
}

pub struct VirtualBudgetService;

impl VirtualBudgetService {
    pub fn calculate_virtual_rows(
        real_categories: &[CategoryBudgetView],
        transactions: &[(i64, i64)], // (category_id, amount)
    ) -> Vec<VirtualCategory> {
        let mut virtual_rows = Vec::new();

        // 1. Total Income
        let total_income: i64 = transactions
            .iter()
            .filter(|(_, amount)| *amount > 0)
            .map(|(_, amount)| *amount)
            .sum();

        virtual_rows.push(VirtualCategory {
            name: "Total Income".to_string(),
            amount: total_income,
            is_income: true,
        });

        // 3. Auto Split (Example: Split "Car Insurance" 50/50)
        // Find the "Car Insurance" category ID
        // In a real app, these rules would be in a config file or DB
        if let Some(car_insurance_cat) = real_categories
            .iter()
            .find(|v| v.category.name == "Car Insurance")
        {
            let insurance_spent = car_insurance_cat.spent;
            let split_amount = insurance_spent / 2;

            virtual_rows.push(VirtualCategory {
                name: "Auto (Mazda)".to_string(),
                amount: split_amount,
                is_income: false,
            });
            virtual_rows.push(VirtualCategory {
                name: "Auto (Elantra)".to_string(),
                amount: split_amount,
                is_income: false,
            });
        }

        virtual_rows
    }
}