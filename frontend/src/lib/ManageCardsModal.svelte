<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let cards: any[] = [];
  export let open = false;

  const dispatch = createEventDispatcher();

  let newCardName = '';
  let saving = false;

  function close() {
    open = false;
    newCardName = '';
  }

  async function addCard() {
    if (!newCardName.trim()) return;
    saving = true;
    try {
      const res = await fetch('/cards', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: newCardName.trim() })
      });
      if (res.ok) {
        newCardName = '';
        dispatch('refresh');
      }
    } catch (e) {
      console.error('Add card failed', e);
    } finally {
      saving = false;
    }
  }

  async function toggleCard(card: any) {
    try {
      const res = await fetch(`/cards/${card.id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: card.name, is_active: !card.is_active })
      });
      if (res.ok) {
        dispatch('refresh');
      }
    } catch (e) {
      console.error('Toggle card failed', e);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
</script>

<svelte:window on:keydown={open ? onKeydown : undefined} />

{#if open}
  <div class="modal fade show" style="display: block;" tabindex="-1" role="dialog">
    <div class="modal-dialog" role="document">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title mb-0">Manage Cards</h5>
          <button type="button" class="btn-close" aria-label="Close" on:click={close}></button>
        </div>
        <div class="modal-body">
          <div class="mb-3">
            {#each cards as card (card.id)}
              <div class="d-flex justify-content-between align-items-center mb-2">
                <span>{card.name} {#if !card.is_active}<span class="text-muted">(Inactive)</span>{/if}</span>
                <button
                  class="btn btn-sm {card.is_active ? 'btn-outline-danger' : 'btn-outline-success'}"
                  on:click={() => toggleCard(card)}
                >
                  {card.is_active ? 'Deactivate' : 'Activate'}
                </button>
              </div>
            {/each}
            {#if cards.length === 0}
              <p class="text-muted mb-0">No cards yet.</p>
            {/if}
          </div>
          <div class="input-group">
            <input
              type="text"
              class="form-control"
              placeholder="New Card Name"
              bind:value={newCardName}
              on:keydown={(e) => e.key === 'Enter' && addCard()}
            >
            <button class="btn btn-primary" disabled={saving || !newCardName.trim()} on:click={addCard}>Add</button>
          </div>
        </div>
      </div>
    </div>
  </div>
  <div class="modal-backdrop fade show" on:click={close} role="presentation"></div>
{/if}
