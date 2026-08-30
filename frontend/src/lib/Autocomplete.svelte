<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  // items: array of { id, name } (only active items should be passed in)
  export let items: { id: number; name: string }[] = [];
  export let selectedId: number | string = '';
  export let placeholder = '';
  export let required = false;

  const dispatch = createEventDispatcher();

  let inputEl: HTMLInputElement;
  let inputText = '';
  let showList = false;
  let activeIndex = -1;
  let invalid = false;

  // Keep the visible text in sync when the selected id changes from outside
  // (e.g. when the parent resets the form or loads a row for editing).
  let lastSyncedId: number | string = '';
  $: if (selectedId !== lastSyncedId) {
    lastSyncedId = selectedId;
    const match = items.find(i => i.id === selectedId);
    inputText = match ? match.name : '';
    invalid = false;
  }

  function escapeRegex(s: string) {
    return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }

  $: filteredItems = inputText
    ? items.filter(i => i.name.toLowerCase().includes(inputText.toLowerCase()))
    : items;

  function highlighted(name: string) {
    if (!inputText) return name;
    const regex = new RegExp(`(${escapeRegex(inputText)})`, 'gi');
    return name.replace(regex, '<strong>$1</strong>');
  }

  function validate() {
    if (!inputText) {
      invalid = false;
      if (selectedId !== '') {
        selectedId = '';
        lastSyncedId = '';
        dispatch('select', null);
      }
      return;
    }
    const match = items.find(i => i.name.toLowerCase() === inputText.toLowerCase());
    if (match) {
      invalid = false;
      if (selectedId !== match.id) {
        selectedId = match.id;
        lastSyncedId = match.id;
        dispatch('select', match);
      }
    } else {
      invalid = true;
      if (selectedId !== '') {
        selectedId = '';
        lastSyncedId = '';
        dispatch('select', null);
      }
    }
  }

  function selectItem(item: { id: number; name: string }) {
    inputText = item.name;
    selectedId = item.id;
    lastSyncedId = item.id;
    invalid = false;
    showList = false;
    dispatch('select', item);
  }

  function handleInput() {
    showList = true;
    activeIndex = -1;
    validate();
  }

  function handleFocus() {
    showList = true;
    activeIndex = -1;
  }

  function handleBlur() {
    // Delay so a click on a list item registers before the list closes.
    setTimeout(() => {
      showList = false;
      validate();
    }, 150);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!showList) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = Math.min(activeIndex + 1, filteredItems.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = Math.max(activeIndex - 1, 0);
    } else if (e.key === 'Enter') {
      if (activeIndex > -1 && filteredItems[activeIndex]) {
        e.preventDefault();
        selectItem(filteredItems[activeIndex]);
      }
    } else if (e.key === 'Tab') {
      if (filteredItems.length > 0) {
        const idx = activeIndex > -1 ? activeIndex : 0;
        selectItem(filteredItems[idx]);
      }
    } else if (e.key === 'Escape') {
      showList = false;
    }
  }
</script>

<div class="autocomplete-wrapper">
  <input
    bind:this={inputEl}
    type="text"
    class="form-control form-control-sm"
    class:is-invalid={invalid}
    {placeholder}
    {required}
    bind:value={inputText}
    on:input={handleInput}
    on:focus={handleFocus}
    on:blur={handleBlur}
    on:keydown={handleKeyDown}
    autocomplete="off"
  >
  {#if showList && filteredItems.length > 0}
    <div class="autocomplete-items">
      {#each filteredItems as item, i}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
          class:autocomplete-active={i === activeIndex}
          on:mousedown|preventDefault={() => selectItem(item)}
        >
          {@html highlighted(item.name)}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .autocomplete-wrapper {
    position: relative;
    z-index: 1050;
  }

  .autocomplete-items {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    border: 1px solid #d4d4d4;
    z-index: 9999;
    background-color: #fff;
    max-height: 200px;
    overflow-y: auto;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  }

  .autocomplete-items div {
    padding: 10px;
    cursor: pointer;
    background-color: #fff;
    border-bottom: 1px solid #d4d4d4;
  }

  .autocomplete-items div:hover {
    background-color: #e9e9e9;
  }

  .autocomplete-active {
    background-color: DodgerBlue !important;
    color: #ffffff;
  }
</style>
