<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Autocomplete from './Autocomplete.svelte';

  export let categories: any[] = [];
  export let cards: any[] = [];
  export let currentMonth: string;

  const dispatch = createEventDispatcher();

  let transaction_date = `${currentMonth}-01`;
  let card_id: number | string = "";
  let category_id: number | string = "";
  let amount_dollars: number | null = null;
  let notes = "";
  let is_income = false;

  $: activeCards = cards.filter(c => c.is_active);
  $: activeCategories = categories.filter(c => c.is_active);

  async function handleSubmit() {
    if (!category_id || !card_id || amount_dollars === null) {
      alert("Please fill in Category, Card and Amount");
      return;
    }

    // Amount is always entered as positive, sign is handled by backend or logic
    // but in our API we expect amount_dollars.
    // The Rust service handles the sign if we use create_transaction_api correctly.
    // Actually, looking at rust code:
    // TransactionService::create_transaction(..., payload.amount_dollars, ...)
    // If we want to support the "Income" checkbox logic:
    const finalAmount = is_income ? Math.abs(amount_dollars) : -Math.abs(amount_dollars);

    try {
      const response = await fetch('/budget/api/add', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          category_id: typeof category_id === 'string' ? parseInt(category_id) : category_id,
          card_id: typeof card_id === 'string' ? parseInt(card_id) : card_id,
          transaction_date,
          amount_dollars: finalAmount,
          notes
        })
      });

      if (response.ok) {
        dispatch('success');
        // Reset form
        category_id = "";
        card_id = "";
        amount_dollars = null;
        notes = "";
        is_income = false;
      }
    } catch (e) {
      console.error("Save failed", e);
    }
  }
</script>

<tr class="table-info">
  <td><input type="date" bind:value={transaction_date} class="form-control form-control-sm"></td>
  <td class="card-col">
    <Autocomplete items={activeCards} bind:selectedId={card_id} placeholder="Card..." required />
  </td>
  <td class="category-col">
    <Autocomplete items={activeCategories} bind:selectedId={category_id} placeholder="Category..." required />
  </td>
  <td>
    <div class="input-group input-group-sm">
      <input type="number" step="0.01" bind:value={amount_dollars} class="form-control" placeholder="0.00">
      <div class="input-group-text">
        <input class="form-check-input mt-0" type="checkbox" bind:checked={is_income} title="Is Income?">
        <span class="ms-1 small">Inc</span>
      </div>
    </div>
  </td>
  <td>
    <input type="text" bind:value={notes} class="form-control form-control-sm" placeholder="Notes" on:keydown={(e) => e.key === 'Enter' && handleSubmit()}>
  </td>
  <td>
    <button class="btn btn-sm btn-primary w-100" on:click={handleSubmit}>Add</button>
  </td>
</tr>
