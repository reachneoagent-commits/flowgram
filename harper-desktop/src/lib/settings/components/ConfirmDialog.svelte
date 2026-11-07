<script lang="ts">
import { Button } from 'components';

export let title: string;

export let message: string;
export let confirmLabel = 'Turn off';
export let cancelLabel = 'Keep on';
export let onConfirm: () => void;
export let onCancel: () => void;

/** Move to <body> so the dialog sits above scrolled and clipped settings content. */
function portal(node: HTMLElement) {
	document.body.appendChild(node);

	return { destroy: () => node.remove() };
}

function onKeydown(event: KeyboardEvent) {
	if (event.key === 'Escape') onCancel();
}
</script>

<svelte:window on:keydown={onKeydown} />

<div class="modal-backdrop" role="presentation" use:portal on:click|self={onCancel}>
  <div class="modal confirm-dialog" role="alertdialog" aria-modal="true" aria-labelledby="confirm-title" aria-describedby="confirm-message">
    <h2 id="confirm-title">{title}</h2>
    <p id="confirm-message">{message}</p>
    <div class="confirm-actions">
      <!-- Keep the safe choice first so Enter and a quick click do not turn things off. -->
      <Button on:click={onCancel}>{cancelLabel}</Button>
      <Button color="white" on:click={onConfirm}>{confirmLabel}</Button>
    </div>
  </div>
</div>

<style>
  .confirm-dialog {
    max-width: 420px;
    padding: 22px;
  }
  h2 {
    margin: 0 0 8px;
    font-size: 17px;
  }
  p {
    margin: 0 0 18px;
    font-size: 13.5px;
    line-height: 1.5;
  }
  .confirm-actions {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
  }
</style>
