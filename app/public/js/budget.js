// budget.js - Core logic for filtering, sorting, and navigation

function applyFilters() {
    const checkedIds = Array.from(document.querySelectorAll('.category-filter:checked')).map(cb => parseInt(cb.value));
    
    // Save to LocalStorage
    localStorage.setItem('checkedCategories', JSON.stringify(checkedIds));

    // Filter Table Rows
    document.querySelectorAll('#transaction-table-body tr').forEach(row => {
        const catId = parseInt(row.dataset.category);
        if(catId) row.style.display = checkedIds.includes(catId) ? '' : 'none';
    });

    // Refresh Charts (from charts.js)
    if (typeof refreshCharts === 'function') {
        refreshCharts(checkedIds);
    }
}

function toggleAllFilters() {
    const isChecked = document.getElementById('filter-all').checked;
    document.querySelectorAll('.category-filter').forEach(cb => cb.checked = isChecked);
    applyFilters();
}

function restoreFilters() {
    const savedFilters = localStorage.getItem('checkedCategories');
    if (savedFilters) {
        const checkedIds = JSON.parse(savedFilters);
        document.querySelectorAll('.category-filter').forEach(cb => {
            cb.checked = checkedIds.includes(parseInt(cb.value));
        });
        
        const allChecked = Array.from(document.querySelectorAll('.category-filter')).every(cb => cb.checked);
        const filterAll = document.getElementById('filter-all');
        if (filterAll) filterAll.checked = allChecked;
    }
    applyFilters();
}

function changeMonth(delta) {
    if (typeof currentMonth === 'undefined') return;
    const [year, month] = currentMonth.split('-').map(Number);
    let d = new Date(year, month - 1 + delta, 1);
    const newMonth = d.getFullYear() + '-' + String(d.getMonth() + 1).padStart(2, '0');
    window.location.href = '/budget/' + newMonth;
}

function sortTransactions() {
    const sortBy = document.getElementById('sort-select').value;
    const tbody = document.getElementById('transaction-table-body');
    if (!tbody) return;
    const rows = Array.from(tbody.querySelectorAll('tr'));

    rows.sort((a, b) => {
        switch (sortBy) {
            case 'date-desc':
                return b.dataset.date.localeCompare(a.dataset.date);
            case 'date-asc':
                return a.dataset.date.localeCompare(b.dataset.date);
            case 'amount-desc':
                return parseFloat(b.dataset.amountDollars) - parseFloat(a.dataset.amountDollars);
            case 'category':
                const catA = a.querySelector('.category-col .badge')?.innerText.toLowerCase() || "";
                const catB = b.querySelector('.category-col .badge')?.innerText.toLowerCase() || "";
                return catA.localeCompare(catB);
            case 'card':
                const cardA = a.querySelector('.card-col')?.innerText.toLowerCase() || "";
                const cardB = b.querySelector('.card-col')?.innerText.toLowerCase() || "";
                return cardA.localeCompare(cardB);
            default:
                return 0;
        }
    });

    rows.forEach(row => tbody.appendChild(row));
}

// Global initialization logic that depends on other files
document.addEventListener('DOMContentLoaded', () => {
    if (typeof initCharts === 'function') initCharts();
    restoreFilters();
});
