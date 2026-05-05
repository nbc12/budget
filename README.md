# Budget App

A personal budgeting application built with Rust and modern web technologies. It provides a simple yet powerful interface for tracking income, expenses, and managing monthly budgets with interactive visualizations.

## Features

- **Monthly Budgeting**: Set and track budget limits on a per-month basis.
- **Automatic Budget Rollover**: Automatically copies budget limits from the previous month when starting a new one.
- **Transaction Tracking**: Easy entry of income and expenses with intelligent category and card (payment method) autocomplete.
- **Interactive Dashboards**: Powered by Chart.js.
    - **Budget vs Actual**: Horizontal bar chart comparing planned vs actual spending.
    - **Distribution**: Pie charts for category-level and transaction-level breakdowns of both income and expenses.
- **Category Management**: Master category list with customizable colors, income/expense toggles, and archiving (active/inactive status).
- **Payment Methods**: Manage different cards and accounts used for transactions.
- **Optional Authentication**: Simple shared-secret password protection that can be disabled for local use.
- **Inline Editing**: Smooth user experience with inline editing for categories and transactions.

## Tech Stack

- **Backend**: [Axum](https://github.com/tokio-rs/axum) (Web Framework), [SQLx](https://github.com/launchbadge/sqlx) (Database Interface), [Tokio](https://tokio.rs/) (Async Runtime).
- **Database**: SQLite.
- **Templating**: [Askama](https://github.com/djc/askama) (Type-safe compiled templates).
- **Frontend**: Vanilla JavaScript, [Bootstrap 5](https://getbootstrap.com/), [Chart.js](https://www.chartjs.org/).

## Architecture

The project follows a highly maintainable Rust web architecture:
- **Cargo Workspace**: Split into multiple crates for better compilation times and isolation.
- **Vertical Slicing**: Organized by domain (`transactions`, `categories`, `cards`) rather than technical layers.
- **Layered Design**: Each domain crate uses a standard Handler -> Service -> Repository -> Model pattern.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)

### Setup

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/nbc12/budget.git
    cd budget
    ```

2.  **Configuration**:
    The app uses command-line arguments (via `clap`) or environment variables. You can create a `.env` file:
    ```env
    DATABASE_URL="sqlite:budget.db"
    PORT="3000"
    # APP_PASSWORD="your_password" # Optional: Leave blank to disable login
    ```

3.  **Run the application**:
    ```bash
    cargo run -p app
    ```
    The app will automatically create the SQLite database and run migrations on startup.

4.  **Access the UI**:
    Open [http://localhost:3000](http://localhost:3000) in your browser.

## Project Structure

- `app/`: The main binary crate (composition root).
- `crates/categories/`: Category and budget limit management.
- `crates/transactions/`: Income and expense record tracking.
- `crates/cards/`: Payment method management.
- `crates/database/`: Shared infrastructure for connection and migration handling.
- `crates/common/`: Shared types, config, and authentication middleware.

## License

MIT
