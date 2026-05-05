// editing.js - Inline editing logic for categories and transactions

let currentEditingRow = null; // { element: HTMLElement, originalHtml: string }
const pastelColors = [
    "#FFB3BA", "#FFDFBA", "#FFFFBA", "#BAFFC9", "#BAE1FF", 
    "#D5AAFF", "#FFABAB", "#85E3FF", "#E2F0CB", "#FDFD96",
    "#FFC3A0", "#D4F0F0", "#CCE2CB", "#B6CFB6", "#97C1A9"
];

function cancelPreviousEdit() {
    if (currentEditingRow) {
        currentEditingRow.element.innerHTML = currentEditingRow.originalHtml;
        currentEditingRow.element.classList.remove('editing-row');
        currentEditingRow = null;
    }
}

// --- Category Editing ---

function editCategory(id) {
    const row = document.querySelector(`tr[data-category-id="${id}"]`);
    const originalHtml = row.innerHTML;
    cancelPreviousEdit();
    currentEditingRow = { element: row, originalHtml: originalHtml };
    
    const nameCell = row.children[1];
    const limitCell = row.children[2];
    const actionsCell = row.children[5];

    const currentName = nameCell.querySelector('.badge').innerText.trim();
    const currentLimit = limitCell.querySelector('.limit-text').innerText.trim();
    const currentColor = row.dataset.color;
    const currentIsIncome = row.dataset.isIncome === 'true';

    nameCell.innerHTML = `
        <div class="d-flex flex-column gap-1">
            <div class="d-flex gap-1 align-items-center">
                <input type="text" class="form-control form-control-sm" id="edit-cat-name-${id}" value="${currentName}" style="min-width: 80px;">
                <select id="edit-cat-color-${id}" class="form-select form-select-sm" style="width: 45px; padding: 2px; background-color: ${currentColor};">
                    ${pastelColors.map(c => `<option value="${c}" ${c === currentColor ? 'selected' : ''} style="background-color: ${c};"></option>`).join('')}
                </select>
            </div>
            <div class="form-check">
                <input class="form-check-input" type="checkbox" id="edit-cat-is-income-${id}" ${currentIsIncome ? 'checked' : ''}>
                <label class="small mb-0" for="edit-cat-is-income-${id}">Income</label>
            </div>
        </div>
    `;
    const colorSelect = document.getElementById(`edit-cat-color-${id}`);
    colorSelect.addEventListener('change', (e) => { e.target.style.backgroundColor = e.target.value; });

    limitCell.innerHTML = `<input type="number" step="0.01" class="form-control form-control-sm" id="edit-cat-limit-${id}" value="${currentLimit}">`;
    
    actionsCell.innerHTML = `
        <div class="d-flex gap-1">
            <button class="btn btn-sm btn-success" onclick="saveCategory(${id})">Save</button>
            <button class="btn btn-sm btn-secondary" onclick="cancelPreviousEdit()">Cancel</button>
        </div>
    `;
}

async function saveCategory(id) {
    const nameInput = document.getElementById(`edit-cat-name-${id}`);
    const limitInput = document.getElementById(`edit-cat-limit-${id}`);
    const colorInput = document.getElementById(`edit-cat-color-${id}`);
    const isIncomeInput = document.getElementById(`edit-cat-is-income-${id}`);

    if (isNaN(parseFloat(limitInput.value))) {
        limitInput.classList.add('is-invalid');
        return;
    }

    const limitResponse = await fetch('/categories/limit', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ category_id: id, month: currentMonth, limit: parseFloat(limitInput.value) })
    });
    
    const renameResponse = await fetch(`/categories/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
            name: nameInput.value, 
            color: colorInput.value,
            is_income: isIncomeInput.checked,
            is_active: true 
        })
    });

    if (limitResponse.ok && renameResponse.ok) {
        currentEditingRow = null;
        location.reload();
    } else {
        alert("Error saving category.");
    }
}

async function deleteCategory(id) {
    if(!confirm("Are you sure? This will affect all months and fail if there are transactions.")) return;
    const response = await fetch(`/categories/${id}`, { method: 'DELETE' });
    if (response.ok) location.reload();
    else alert("Error deleting category. It may have transactions.");
}

function editLimit(catId) {
    const row = document.querySelector(`tr[data-category-id="${catId}"]`);
    row.querySelector('.limit-text').classList.add('d-none');
    row.querySelector('.limit-input').classList.remove('d-none');
    row.querySelector('.limit-input').focus();
}

async function saveLimit(catId) {
    const row = document.querySelector(`tr[data-category-id="${catId}"]`);
    const val = row.querySelector('.limit-input').value;
    
    const response = await fetch('/categories/limit', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ category_id: catId, month: currentMonth, limit: parseFloat(val) })
    });

    if (response.ok) {
        location.reload();
    }
}

// --- Transaction Editing ---

function editRow(id) {
    const row = document.getElementById(`row-${id}`);
    const originalHtml = row.innerHTML;
    cancelPreviousEdit();
    currentEditingRow = { element: row, originalHtml: originalHtml };
    row.classList.add('editing-row');
    
    const date = row.dataset.date;
    const catId = row.dataset.category;
    const cardId = row.dataset.card;
    const notes = row.dataset.notes;
    const amount = row.dataset.amountDollars;

    const currentCat = categories.find(c => c.id == catId) || { id: 0, name: 'Unknown' };
    const currentCard = cards.find(c => c.id == cardId) || { id: 0, name: 'Cash' };

    row.innerHTML = `
        <td><input type="date" id="edit-date-${id}" class="form-control form-control-sm" value="${date}"></td>
        <td class="card-col">
            <div class="autocomplete-wrapper">
                <input type="text" id="edit-card-input-${id}" class="form-control form-control-sm" value="${currentCard.name}" placeholder="Card">
                <input type="hidden" id="edit-card-id-${id}" value="${cardId}">
            </div>
        </td>
        <td class="category-col">
            <div class="autocomplete-wrapper">
                <input type="text" id="edit-category-input-${id}" class="form-control form-control-sm" value="${currentCat.name}" placeholder="Cat">
                <input type="hidden" id="edit-category-id-${id}" value="${catId}">
            </div>
        </td>
        <td class="amount-col">
            <input type="number" step="0.01" id="edit-amount-${id}" class="form-control form-control-sm" value="${amount}">
        </td>
        <td class="notes-col"><input type="text" id="edit-notes-${id}" class="form-control form-control-sm" value="${notes}" placeholder="Notes"></td>
        <td style="white-space: nowrap;">
            <div class="d-flex gap-1">
                <button class="btn btn-sm btn-success" onclick="saveRow(${id})">Save</button>
                <button class="btn btn-sm btn-secondary" onclick="cancelPreviousEdit()">Cancel</button>
            </div>
        </td>
    `;

    new Autocomplete(document.getElementById(`edit-category-input-${id}`), categories.filter(c => c.is_active), (item) => {
        document.getElementById(`edit-category-id-${id}`).value = item ? item.id : "";
    });
    new Autocomplete(document.getElementById(`edit-card-input-${id}`), cards.filter(c => c.is_active), (item) => {
        document.getElementById(`edit-card-id-${id}`).value = item ? item.id : "";
    });
}

async function saveRow(id) {
    const dateInput = document.getElementById(`edit-date-${id}`);
    const catIdInput = document.getElementById(`edit-category-id-${id}`);
    const cardIdInput = document.getElementById(`edit-card-id-${id}`);
    const amountInput = document.getElementById(`edit-amount-${id}`);
    const notesInput = document.getElementById(`edit-notes-${id}`);

    if (isNaN(parseFloat(amountInput.value))) {
        amountInput.classList.add('is-invalid');
        return;
    }

    const response = await fetch(`/budget/transaction/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            category_id: parseInt(catIdInput.value),
            card_id: cardIdInput.value == "0" ? null : parseInt(cardIdInput.value),
            transaction_date: dateInput.value,
            amount_dollars: parseFloat(amountInput.value),
            notes: notesInput.value
        })
    });

    if (response.ok) {
        location.reload(); 
    }
}

// --- Card Management ---

async function toggleCard(id) {
    const card = cards.find(c => c.id == id);
    if (!card) return;

    const newState = !card.is_active;
    const response = await fetch(`/cards/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: card.name, is_active: newState })
    });

    if (response.ok) {
        location.reload();
    }
}

async function addCard() {
    const nameInput = document.getElementById('new-card-name');
    const name = nameInput.value;
    if (!name) return;

    const response = await fetch('/cards', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: name })
    });

    if (response.ok) {
        location.reload();
    }
}

// --- Deletion ---

let transactionToDelete = null;
function confirmDelete(id) { 
    transactionToDelete = id; 
    const modal = new bootstrap.Modal(document.getElementById('deleteModal'));
    modal.show(); 
}

document.addEventListener('DOMContentLoaded', () => {
    const confirmBtn = document.getElementById('confirmDeleteBtn');
    if (confirmBtn) {
        confirmBtn.addEventListener('click', async () => {
            if (transactionToDelete) {
                const response = await fetch('/budget/transaction/' + transactionToDelete, { method: 'DELETE' });
                if (response.ok) location.reload();
            }
        });
    }
});
