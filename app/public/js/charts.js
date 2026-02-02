// charts.js - Logic for Budget App visualizations

let barChart, expensePieChart, incomeTransPieChart, expenseTransPieChart;

function updateChartHeight(categoryCount) {
    const BAR_THICKNESS = 10;
    const HEIGHT_PER_CATEGORY = BAR_THICKNESS * 6;
    const chartPadding = 100;
    const dynamicHeight = Math.max(300, (categoryCount * HEIGHT_PER_CATEGORY) + chartPadding);
    const container = document.getElementById('barChartContainer');
    if (container) container.style.height = dynamicHeight + 'px';
}

function initCharts() {
    if (typeof currentBudgetRows === 'undefined') return;
    
    const activeRows = currentBudgetRows.filter(r => r.is_active);
    const labels = activeRows.map(r => r.name);
    const limits = activeRows.map(r => r.limit);
    const spent = activeRows.map(r => r.spent);
    const colors = activeRows.map(r => r.color);

    updateChartHeight(labels.length);

    // Bar Chart
    const barCtx = document.getElementById('budgetBarChart');
    if (barCtx) {
        barChart = new Chart(barCtx, {
            type: 'bar',
            data: {
                labels: labels,
                datasets: [
                    {
                        label: 'Budget',
                        data: limits,
                        backgroundColor: '#e9ecef',
                        borderColor: '#adb5bd',
                        borderWidth: 1,
                        barThickness: 10
                    },
                    {
                        label: 'Actual',
                        data: spent,
                        backgroundColor: colors,
                        borderColor: colors.map(c => c),
                        borderWidth: 1,
                        barThickness: 10
                    }
                ]
            },
            options: {
                indexAxis: 'y',
                responsive: true,
                maintainAspectRatio: false,
                plugins: { title: { display: true, text: 'Budget vs Actual' } },
                scales: {
                    x: { beginAtZero: true, ticks: { maxTicksLimit: 5 } },
                    y: { stacked: false, grid: { display: false } }
                }
            }
        });
    }

    // Category Expense Pie
    const expenseCtx = document.getElementById('expensePieChart');
    if (expenseCtx) {
        const expenseData = currentBudgetRows.filter(r => !r.is_income && r.spent > 0);
        expensePieChart = new Chart(expenseCtx, {
            type: 'pie',
            data: {
                labels: expenseData.map(r => r.name),
                datasets: [{
                    data: expenseData.map(r => r.spent),
                    backgroundColor: expenseData.map(r => r.color),
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    title: { display: true, text: 'Expenses by Category' },
                    legend: { position: 'bottom', labels: { boxWidth: 10, font: { size: 9 } } }
                }
            }
        });
    }

    // Transaction Income Pie
    const incomeTransCtx = document.getElementById('incomeTransPieChart');
    if (incomeTransCtx) {
        const incomeTrans = allTransactions.filter(t => t.is_income);
        incomeTransPieChart = new Chart(incomeTransCtx, {
            type: 'pie',
            data: {
                labels: incomeTrans.map(t => `${t.category_name}: ${t.notes || 'Income'}`),
                datasets: [{
                    data: incomeTrans.map(t => t.amount),
                    backgroundColor: incomeTrans.map(t => t.category_color),
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    title: { display: true, text: 'Individual Income Transactions' },
                    legend: { display: false }
                }
            }
        });
    }

    // Transaction Expense Pie
    const expenseTransCtx = document.getElementById('expenseTransPieChart');
    if (expenseTransCtx) {
        const expenseTrans = allTransactions.filter(t => !t.is_income);
        expenseTransPieChart = new Chart(expenseTransCtx, {
            type: 'pie',
            data: {
                labels: expenseTrans.map(t => `${t.category_name}: ${t.notes || 'Expense'}`),
                datasets: [{
                    data: expenseTrans.map(t => t.amount),
                    backgroundColor: expenseTrans.map(t => t.category_color),
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    title: { display: true, text: 'Individual Expense Transactions' },
                    legend: { display: false }
                }
            }
        });
    }
}

function refreshCharts(checkedIds) {
    if (!(barChart && expensePieChart && incomeTransPieChart && expenseTransPieChart)) return;

    const filteredRows = currentBudgetRows.filter(r => checkedIds.includes(r.id));
    const filteredTrans = allTransactions.filter(t => checkedIds.includes(t.category_id));
    
    updateChartHeight(filteredRows.length);

    // Update Bar
    barChart.data.labels = filteredRows.map(r => r.name);
    barChart.data.datasets[0].data = filteredRows.map(r => r.limit);
    barChart.data.datasets[1].data = filteredRows.map(r => r.spent);
    barChart.data.datasets[1].backgroundColor = filteredRows.map(r => r.color);
    barChart.update();

    // Update Expense Category Pie
    const expenseData = filteredRows.filter(r => !r.is_income && r.spent > 0);
    expensePieChart.data.labels = expenseData.map(r => r.name);
    expensePieChart.data.datasets[0].data = expenseData.map(r => r.spent);
    expensePieChart.data.datasets[0].backgroundColor = expenseData.map(r => r.color);
    expensePieChart.update();

    // Update Trans Income Pie
    const incTrans = filteredTrans.filter(t => t.is_income);
    incomeTransPieChart.data.labels = incTrans.map(t => `${t.category_name}: ${t.notes || 'Income'}`);
    incomeTransPieChart.data.datasets[0].data = incTrans.map(t => t.amount);
    incomeTransPieChart.data.datasets[0].backgroundColor = incTrans.map(t => t.category_color);
    incomeTransPieChart.update();

    // Update Trans Expense Pie
    const expTrans = filteredTrans.filter(t => !t.is_income);
    expenseTransPieChart.data.labels = expTrans.map(t => `${t.category_name}: ${t.notes || 'Expense'}`);
    expenseTransPieChart.data.datasets[0].data = expTrans.map(t => t.amount);
    expenseTransPieChart.data.datasets[0].backgroundColor = expTrans.map(t => t.category_color);
    expenseTransPieChart.update();
}
