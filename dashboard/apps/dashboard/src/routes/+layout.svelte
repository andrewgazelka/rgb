<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import Navbar from '$lib/components/Navbar.svelte';
	import KeybindHelp from '$lib/components/KeybindHelp.svelte';
	import { handleKeydown } from '$lib/keybinds.svelte';

	let { children } = $props();

	function onKeydown(event: KeyboardEvent) {
		handleKeydown(event);
	}
</script>

<svelte:window onkeydown={onKeydown} />

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="app">
	<Navbar />
	<main class="main-content">
		{@render children()}
	</main>
</div>

<KeybindHelp />

<style>
	.app {
		height: 100vh;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.main-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-height: 0;
		overflow: hidden;
	}
</style>
