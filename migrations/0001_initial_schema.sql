-- Consolidated Initial Schema

-- 1. CATEGORIES TABLE (Master List)
CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL DEFAULT '#f8f9fa',
    is_income BOOLEAN NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT 1
);

-- 2. CARDS TABLE (Payment Methods)
CREATE TABLE cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1
);

-- 3. MONTHLY BUDGETS TABLE (Per-Month Limits)
CREATE TABLE monthly_budgets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    month TEXT NOT NULL, -- Format: YYYY-MM
    limit_amount INTEGER NOT NULL DEFAULT 0, -- Stored in Cents
    UNIQUE(category_id, month),
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

-- 4. TRANSACTIONS TABLE
CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    card_id INTEGER,
    transaction_date TEXT NOT NULL, -- Format: YYYY-MM-DD
    amount INTEGER NOT NULL, -- Cents, Positive=Income, Negative=Expense
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE RESTRICT,
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE SET NULL
);

-- 5. INDEXES
CREATE INDEX idx_transactions_date ON transactions(transaction_date);
CREATE INDEX idx_transactions_category ON transactions(category_id);
CREATE INDEX idx_transactions_card ON transactions(card_id);
CREATE INDEX idx_monthly_budgets_month ON monthly_budgets(month);

-- 6. INITIAL SEED DATA
INSERT INTO cards (name, is_active) VALUES ('AFCU Debit', 1);
INSERT INTO cards (name, is_active) VALUES ('AFCU Credit', 1);
INSERT INTO cards (name, is_active) VALUES ('Discover Credit', 1);

-- Master Categories
INSERT INTO categories (name, color, is_income) VALUES ('Salary', '#BAFFC9', 1);
INSERT INTO categories (name, color, is_income) VALUES ('Rent', '#BAE1FF', 0);
INSERT INTO categories (name, color, is_income) VALUES ('Groceries', '#FFB3BA', 0);
INSERT INTO categories (name, color, is_income) VALUES ('Fast Food', '#FFDFBA', 0);
INSERT INTO categories (name, color, is_income) VALUES ('Mazda auto', '#FFDAC1', 0);
INSERT INTO categories (name, color, is_income) VALUES ('Elantra auto', '#FCB7AF', 0);
INSERT INTO categories (name, color, is_income) VALUES ('Phone', '#FFFFBA', 0);
INSERT INTO categories (name, color, is_income) VALUES ('Health', '#D4F0F0', 0);
INSERT INTO categories (name, color, is_income) VALUES ('Other', '#f8f9fa', 0);
INSERT INTO categories (name, color, is_income) VALUES ('Subscriptions', '#D5AAFF', 0);
INSERT INTO categories (name, color, is_income) VALUES ('Tithing', '#E7FFAC', 0);

-- Initial Monthly Budgets (Cents)
INSERT INTO monthly_budgets (category_id, month, limit_amount) SELECT id, strftime('%Y-%m', 'now'), 344000 FROM categories WHERE name = 'Salary';
INSERT INTO monthly_budgets (category_id, month, limit_amount) SELECT id, strftime('%Y-%m', 'now'), 130000 FROM categories WHERE name = 'Rent';
INSERT INTO monthly_budgets (category_id, month, limit_amount) SELECT id, strftime('%Y-%m', 'now'), 40000 FROM categories WHERE name = 'Groceries';
INSERT INTO monthly_budgets (category_id, month, limit_amount) SELECT id, strftime('%Y-%m', 'now'), 5000 FROM categories WHERE name = 'Fast Food';
INSERT INTO monthly_budgets (category_id, month, limit_amount) SELECT id, strftime('%Y-%m', 'now'), 40000 FROM categories WHERE name = 'Mazda auto';
INSERT INTO monthly_budgets (category_id, month, limit_amount) SELECT id, strftime('%Y-%m', 'now'), 40000 FROM categories WHERE name = 'Elantra auto';
INSERT INTO monthly_budgets (category_id, month, limit_amount) SELECT id, strftime('%Y-%m', 'now'), 3000 FROM categories WHERE name = 'Phone';
INSERT INTO monthly_budgets (category_id, month, limit_amount) SELECT id, strftime('%Y-%m', 'now'), 0 FROM categories WHERE name = 'Health';
INSERT INTO monthly_budgets (category_id, month, limit_amount) SELECT id, strftime('%Y-%m', 'now'), 20000 FROM categories WHERE name = 'Other';
INSERT INTO monthly_budgets (category_id, month, limit_amount) SELECT id, strftime('%Y-%m', 'now'), 700 FROM categories WHERE name = 'Subscriptions';
INSERT INTO monthly_budgets (category_id, month, limit_amount) SELECT id, strftime('%Y-%m', 'now'), 34400 FROM categories WHERE name = 'Tithing';
