<script lang="ts">
  import { onMount } from 'svelte';
  import TransactionRow from './lib/TransactionRow.svelte';
  import TransactionForm from './lib/TransactionForm.svelte';
  import Charts from './lib/Charts.svelte';
  import ManageCardsModal from './lib/ManageCardsModal.svelte';
  
  let currentMonth = new Date().toISOString().slice(0, 7);
  let data: any = null;
  let allCategories: any[] = [];
  let allCards: any[] = [];
  let loading = true;

  // Filtering
  const FILTERS_STORAGE_KEY = 'checkedCategories';
  let checkedCategoryIds: number[] = [];
  let filterAll = true;
  let filtersInitialized = false;

  function loadSavedFilters(): number[] | null {
    try {
      const raw = localStorage.getItem(FILTERS_STORAGE_KEY);
      return raw ? JSON.parse(raw) : null;
    } catch (e) {
      return null;
    }
  }

  function saveFilters(ids: number[]) {
    try {
      localStorage.setItem(FILTERS_STORAGE_KEY, JSON.stringify(ids));
    } catch (e) {
      // localStorage unavailable (private browsing, etc.) - ignore
    }
  }

  // Sorting
  let sortBy = 'date-desc';

  // Manage Cards modal
  let cardsModalOpen = false;

  // Category inline edit (Budget table)
  const PASTEL_COLORS = [
    "#FFB3BA", "#FFDFBA", "#FFFFBA", "#BAFFC9", "#BAE1FF",
    "#E2F0CB", "#FDFD96", "#FFC3A0", "#FFD1DC", "#D4F0F0",
    "#CCE2CB", "#B6CFB6", "#97C1A9", "#FCB7AF", "#FFDAC1",
    "#E7FFAC", "#FFABAB", "#D5AAFF", "#85E3FF", "#B9F6CA"
  ];
  let editingCategoryId: number | null = null;
  let editCatName = '';
  let editCatColor = '';
  let editCatIsIncome = false;
  let editCatLimit = '';

  function startEditCategory(row: any) {
    editingCategoryId = row.category_id;
    editCatName = row.category_name;
    editCatColor = row.category_color;
    editCatIsIncome = row.is_income;
    editCatLimit = row.limit_dollars;
  }

  function cancelEditCategory() {
    editingCategoryId = null;
  }

  async function saveCategory(id: number) {
    const limit = parseFloat(editCatLimit);
    if (isNaN(limit)) return;

    try {
      const [limitRes, renameRes] = await Promise.all([
        fetch('/categories/limit', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ category_id: id, month: currentMonth, limit })
        }),
        fetch(`/categories/${id}`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            name: editCatName,
            color: editCatColor,
            is_income: editCatIsIncome,
            is_active: true
          })
        })
      ]);

      if (limitRes.ok && renameRes.ok) {
        editingCategoryId = null;
        fetchData();
      } else {
        alert('Error saving category.');
      }
    } catch (e) {
      console.error('Save category failed', e);
    }
  }

  async function deleteCategory(id: number) {
    if (!confirm('Are you sure? This will affect all months and fail if there are transactions.')) return;
    try {
      const res = await fetch(`/categories/${id}`, { method: 'DELETE' });
      if (res.ok) {
        fetchData();
      } else {
        const body = await res.json().catch(() => null);
        alert(body?.error || 'Error deleting category. It may have transactions.');
      }
    } catch (e) {
      console.error('Delete category failed', e);
    }
  }

  async function fetchData() {
    loading = true;
    try {
      const [dataRes, catRes, cardRes] = await Promise.all([
        fetch(`/budget/api/${currentMonth}`),
        fetch('/categories/api'),
        fetch('/cards/all')
      ]);
      
      data = await dataRes.json();
      allCategories = await catRes.json();
      allCards = await cardRes.json();

      // Initialize filters if first load: restore from localStorage if
      // present (matching the old app's behavior), else default to all.
      if (!filtersInitialized) {
        filtersInitialized = true;
        const allIds = data.budget_rows.filter((r: any) => r.is_active).map((r: any) => r.category_id);
        const saved = loadSavedFilters();
        checkedCategoryIds = saved ? allIds.filter((id: number) => saved.includes(id)) : allIds;
      }
    } catch (e) {
      console.error("Fetch failed", e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchData();
  });

  function changeMonth(delta: number) {
    const [y, m] = currentMonth.split('-').map(Number);
    const d = new Date(y, m - 1 + delta, 1);
    currentMonth = d.getFullYear() + '-' + String(d.getMonth() + 1).padStart(2, '0');
    fetchData();
  }

  // Old app never displayed deactivated categories on the dashboard (budget
  // table, filters, or charts) - only the dedicated Manage Categories page
  // shows inactive ones. Keep that behavior here.
  $: activeBudgetRows = data ? data.budget_rows.filter((r: any) => r.is_active) : [];

  function toggleAllFilters() {
    if (filterAll) {
      checkedCategoryIds = activeBudgetRows.map((r: any) => r.category_id);
    } else {
      checkedCategoryIds = [];
    }
  }

  $: filteredTransactions = data ? data.transactions.filter((t: any) => checkedCategoryIds.includes(t.category_id)) : [];
  $: filteredBudgetRows = activeBudgetRows.filter((r: any) => checkedCategoryIds.includes(r.category_id));
  $: sortedTransactions = sortTransactions(filteredTransactions, sortBy);
  // t.amount_dollars from the API is always a positive display value (see
  // TransactionRow), with sign carried separately in is_income - so it must
  // be applied here too when summing.
  $: transactionsSum = filteredTransactions.reduce((sum: number, t: any) => {
    const amount = parseFloat(t.amount_dollars);
    return sum + (t.is_income ? amount : -amount);
  }, 0);

  function sortTransactions(transactions: any[], sortBy: string) {
    const sorted = [...transactions];
    sorted.sort((a, b) => {
      switch (sortBy) {
        case 'date-desc':
          return b.transaction_date.localeCompare(a.transaction_date);
        case 'date-asc':
          return a.transaction_date.localeCompare(b.transaction_date);
        case 'amount-desc':
          return parseFloat(b.amount_dollars) - parseFloat(a.amount_dollars);
        case 'category':
          return a.category_name.toLowerCase().localeCompare(b.category_name.toLowerCase());
        case 'card':
          return a.card_name.toLowerCase().localeCompare(b.card_name.toLowerCase());
        default:
          return 0;
      }
    });
    return sorted;
  }
  $: {
    if (data) {
        filterAll = checkedCategoryIds.length === activeBudgetRows.length;
    }
  }
  $: if (filtersInitialized) {
    saveFilters(checkedCategoryIds);
  }

  async function handleAddCategory(e: Event) {
    const form = e.target as HTMLFormElement;
    const formData = new FormData(form);
    const name = formData.get('name') as string;
    const limit = parseFloat(formData.get('monthly_limit') as string);
    const is_income = formData.get('is_income') === 'on';

    try {
        const res = await fetch('/categories', {
            method: 'POST',
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            body: new URLSearchParams({
                name,
                monthly_limit: limit.toString(),
                is_income: is_income ? 'on' : ''
            })
        });
        if (res.ok) {
            form.reset();
            fetchData();
        }
    } catch (e) {
        console.error("Add category failed", e);
    }
  }

  async function updateLimit(catId: number, limit: string) {
    try {
        await fetch('/categories/limit', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ category_id: catId, month: currentMonth, limit: parseFloat(limit) })
        });
        fetchData();
    } catch (e) {
        console.error("Limit update failed", e);
    }
  }
</script>

<main class="container py-4 mx-auto" style="max-width: 1100px;">
  <header class="d-flex justify-content-between align-items-center mb-4">
    <button class="btn btn-outline-secondary btn-sm" on:click={() => changeMonth(-1)}>← Prev</button>
    <div class="text-center">
      <h1 class="h2 mb-0">{data ? data.month_display : currentMonth}</h1>
      <div class="d-flex justify-content-center gap-2 small">
         <button class="btn btn-link btn-sm p-0 text-muted" on:click={() => cardsModalOpen = true}>Manage Cards</button>
         <span class="text-muted">|</span>
         <a href="/categories" class="text-muted text-decoration-none">Manage Categories</a>
      </div>
    </div>
    <button class="btn btn-outline-secondary btn-sm" on:click={() => changeMonth(1)}>Next →</button>
  </header>

  {#if loading && !data}
    <div class="text-center py-5">
      <div class="spinner-border text-primary" role="status">
        <span class="visually-hidden">Loading...</span>
      </div>
    </div>
  {:else if data}
    <!-- Financial Overview -->
    <div class="row g-2 mb-4">
      <div class="col-4">
        <div class="stats-card">
          <div class="text-muted small text-uppercase">Income</div>
          <div class="h4 mb-0 text-success">${data.overview.total_income}</div>
        </div>
      </div>
      <div class="col-4">
        <div class="stats-card">
          <div class="text-muted small text-uppercase">Expenses</div>
          <div class="h4 mb-0 text-danger">${data.overview.total_expenses}</div>
        </div>
      </div>
      <div class="col-4">
        <div class="stats-card">
          <div class="text-muted small text-uppercase">Net</div>
          <div class="h4 mb-0 {data.overview.net_is_positive ? 'text-success' : 'text-danger'}">
            ${data.overview.net_balance}
          </div>
        </div>
      </div>
    </div>

    <!-- Charts -->
    <Charts budgetRows={filteredBudgetRows} transactions={filteredTransactions} />

    <!-- Budget Table -->
    <section class="mb-5">
      <div class="d-flex justify-content-between align-items-center mb-3">
        <h2 class="h4 mb-0">Budget Summary</h2>
      </div>
      <div class="table-responsive bg-white rounded shadow-sm">
        <table class="table table-hover mb-0">
          <thead class="table-light">
            <tr>
              <th style="width: 40px;">
                <input type="checkbox" bind:checked={filterAll} on:change={toggleAllFilters}>
              </th>
              <th>Category</th>
              <th>Budget</th>
              <th>Actual</th>
              <th>Remaining</th>
              <th style="width: 1%; white-space: nowrap;">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each activeBudgetRows as row}
              {#if editingCategoryId === row.category_id}
                <tr>
                  <td></td>
                  <td>
                    <div class="d-flex flex-column gap-1">
                      <div class="d-flex gap-1 align-items-center">
                        <input type="text" class="form-control form-control-sm" style="min-width: 80px;" bind:value={editCatName}>
                        <select class="form-select form-select-sm" style="width: 45px; padding: 2px; background-color: {editCatColor};" bind:value={editCatColor}>
                          {#each PASTEL_COLORS as c}
                            <option value={c} style="background-color: {c};"></option>
                          {/each}
                        </select>
                      </div>
                      <div class="form-check">
                        <input class="form-check-input" type="checkbox" id="edit-cat-inc-{row.category_id}" bind:checked={editCatIsIncome}>
                        <label class="small mb-0" for="edit-cat-inc-{row.category_id}">Income</label>
                      </div>
                    </div>
                  </td>
                  <td>
                    <input type="number" step="0.01" class="form-control form-control-sm" bind:value={editCatLimit}>
                  </td>
                  <td>${row.spent_dollars}</td>
                  <td>${row.remaining_dollars}</td>
                  <td style="white-space: nowrap;">
                    <div class="d-flex gap-1">
                      <button class="btn btn-sm btn-success" on:click={() => saveCategory(row.category_id)}>Save</button>
                      <button class="btn btn-sm btn-secondary" on:click={cancelEditCategory}>Cancel</button>
                    </div>
                  </td>
                </tr>
              {:else}
                <tr>
                  <td>
                    <input type="checkbox" value={row.category_id} bind:group={checkedCategoryIds}>
                  </td>
                  <td>
                    <span class="badge" style="background-color: {row.category_color}; color: #333; border: 1px solid #ddd;">
                      {row.category_name}
                    </span>
                    {#if row.is_income}
                      <small class="text-success ms-1">(Income)</small>
                    {/if}
                  </td>
                  <td>
                      <div class="d-flex align-items-center">
                          $<input type="number" step="0.01" class="form-control form-control-sm border-0 bg-transparent p-0"
                                 style="width: 80px;"
                                 value={row.limit_dollars}
                                 on:blur={(e) => updateLimit(row.category_id, e.currentTarget.value)}>
                      </div>
                  </td>
                  <td>${row.spent_dollars} <small class="text-muted">({row.percent_spent}%)</small></td>
                  <td>${row.remaining_dollars}</td>
                  <td style="white-space: nowrap;">
                    <div class="d-flex gap-1">
                      <button class="btn btn-sm btn-outline-primary" on:click={() => startEditCategory(row)}>Edit</button>
                      <button class="btn btn-sm btn-outline-danger" on:click={() => deleteCategory(row.category_id)}>Delete</button>
                    </div>
                  </td>
                </tr>
              {/if}
            {/each}
          </tbody>
          <tfoot class="table-info">
            <tr>
              <td></td>
              <td>
                <form id="add-cat-form" on:submit|preventDefault={handleAddCategory} class="d-flex gap-1 align-items-center">
                    <input type="text" name="name" class="form-control form-control-sm" placeholder="New Category..." required>
                    <div class="form-check mb-0">
                        <input class="form-check-input" type="checkbox" name="is_income" id="new-cat-inc">
                        <label class="small mb-0" for="new-cat-inc">Inc</label>
                    </div>
                </form>
              </td>
              <td>
                <input type="number" form="add-cat-form" name="monthly_limit" step="0.01" class="form-control form-control-sm" placeholder="0.00" required>
              </td>
              <td>--</td>
              <td>--</td>
              <td>
                <button type="submit" form="add-cat-form" class="btn btn-sm btn-primary w-100">Add</button>
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    </section>

    <!-- Transactions Table -->
    <section>
      <div class="d-flex justify-content-between align-items-center mb-3">
        <h2 class="h4 mb-0">Transactions</h2>
        <div class="d-flex align-items-center gap-3">
          <span class="small">
            Total: <strong class={transactionsSum < 0 ? 'text-danger' : 'text-success'}>${transactionsSum.toFixed(2)}</strong>
          </span>
          <span class="small text-muted">Sort by:</span>
          <select class="form-select form-select-sm" bind:value={sortBy} style="width: auto;">
            <option value="date-desc">Date (Newest)</option>
            <option value="date-asc">Date (Oldest)</option>
            <option value="amount-desc">Amount (Highest)</option>
            <option value="category">Category</option>
            <option value="card">Card</option>
          </select>
        </div>
      </div>
      <div class="table-responsive bg-white rounded shadow-sm">
        <table class="table table-hover mb-0">
          <thead class="table-light">
            <tr>
              <th>Date</th>
              <th>Card</th>
              <th>Category</th>
              <th>Amount</th>
              <th>Notes</th>
              <th style="width: 100px;">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each sortedTransactions as t (t.id)}
              <TransactionRow {t} categories={allCategories} cards={allCards} on:refresh={fetchData} />
            {/each}
          </tbody>
          <tfoot>
            <TransactionForm categories={allCategories} cards={allCards} {currentMonth} on:success={fetchData} />
          </tfoot>
        </table>
      </div>
    </section>
  {/if}
</main>

<ManageCardsModal bind:open={cardsModalOpen} cards={allCards} on:refresh={fetchData} />

<style>
  .stats-card {
    text-align: center;
    padding: 15px;
    border-radius: 10px;
    background: white;
    box-shadow: 0 2px 4px rgba(0,0,0,0.05);
  }
  
  :global(body) {
    background-color: #f8f9fa;
  }

  .form-control:focus {
    box-shadow: none;
    border-color: #dee2e6;
  }
</style>