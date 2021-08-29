<script>
 import turbocharger_init from "./turbocharger_generated";
 import App from "./App.svelte";
</script>

{#await turbocharger_init("turbocharger_generated/index_bg.wasm") then _}
 <App />
{:catch error}
 <p style="color: red">Error loading WASM: {error.message}</p>
{/await}
