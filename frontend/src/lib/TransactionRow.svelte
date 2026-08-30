<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Autocomplete from './Autocomplete.svelte';
  export let t: any;
  export let categories: any[];
  export let cards: any[];

  const dispatch = createEventDispatcher();
  let isEditing = false;

  let editData = { ...t };
  // Convert positive amount back to absolute for the input if needed,
  // but we'll just use the raw amount_dollars from the view
  editData.amount_dollars = parseFloat(t.amount_dollars);

  // Active items, plus whatever is currently assigned to this transaction
  // (even if it's since been deactivated) so the row can still display/edit it.
  $: editCards = cards.some(c => c.id === editData.card_id)
    ? cards.filter(c => c.is_active || c.id === editData.card_id)
    : cards;
  $: editCategories = categories.some(c => c.id === editData.category_id)
    ? categories.filter(c => c.is_active || c.id === editData.category_id)
    : categories;

  async function save() {
    try {
      const response = await fetch(`/budget/api/transaction/${t.id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          category_id: editData.category_id,
          card_id: editData.card_id === 0 ? null : editData.card_id,
          transaction_date: editData.transaction_date,
          amount_dollars: t.is_income ? Math.abs(editData.amount_dollars) : -Math.abs(editData.amount_dollars),
          notes: editData.notes
        })
      });
      if (response.ok) {
        isEditing = false;
        dispatch('refresh');
      }
    } catch (e) {
      console.error("Update failed", e);
    }
  }

  async function remove() {
    if (!confirm("Delete transaction?")) return;
    try {
      const response = await fetch(`/budget/transaction/${t.id}`, { method: 'DELETE' });
      if (response.ok) dispatch('refresh');
    } catch (e) {
      console.error("Delete failed", e);
    }
  }
</script>

{#if isEditing}
  <tr class="table-warning">
    <td><input type="date" bind:value={editData.transaction_date} class="form-control form-control-sm"></td>
    <td class="card-col">
      <Autocomplete items={editCards} bind:selectedId={editData.card_id} placeholder="Card..." required />
    </td>
    <td class="category-col">
      <Autocomplete items={editCategories} bind:selectedId={editData.category_id} placeholder="Category..." required />
    </td>
    <td><input type="number" step="0.01" bind:value={editData.amount_dollars} class="form-control form-control-sm"></td>
    <td><input type="text" bind:value={editData.notes} class="form-control form-control-sm"></td>
    <td>
      <div class="d-flex gap-1">
        <button class="btn btn-sm btn-success" on:click={save}>Save</button>
        <button class="btn btn-sm btn-secondary" on:click={() => isEditing = false}>Cancel</button>
      </div>
    </td>
  </tr>
{:else}
  <tr>
    <td>{t.transaction_date_display}</td>
    <td>{t.card_name}</td>
    <td>
      <span class="badge" style="background-color: {t.category_color}; color: #333; border: 1px solid #ddd;">
        {t.category_name}
      </span>
    </td>
    <td class={t.is_income ? 'text-success' : 'text-danger'}>
      ${t.amount_dollars}
    </td>
    <td>{t.notes}</td>
    <td>
      <div class="d-flex gap-1">
        <button class="btn btn-sm btn-outline-primary" on:click={() => isEditing = true}>Edit</button>
        <button class="btn btn-sm btn-outline-danger" on:click={remove}>Delete</button>
      </div>
    </td>
  </tr>
{/if}
