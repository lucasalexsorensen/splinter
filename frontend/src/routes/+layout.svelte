<script lang="ts">
	import '../app.css';

	let { children } = $props();
</script>

<svelte:head>
	<title>Rat Command Center</title>
</svelte:head>
{@render children()}
