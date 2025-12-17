<script lang="ts">
	import { Check, X, Loader, Ban, RotateCcw, Bluetooth, Unlink2 } from '@lucide/svelte';
	import type { Component } from 'svelte';
	import type { ConnectionState } from '$lib/network';
	type Props = {
		status: ConnectionState;
		connect: () => Promise<void>;
	};

	let { status, connect }: Props = $props();

	const [colorClass, Icon] = $derived(statusToColorClassAndIcon(status));
	const showConnectButton = $derived(status === 'disconnected');
	const showRetryButton = $derived(status === 'error');

	function statusToColorClassAndIcon(status: ConnectionState): [string, Component] {
		switch (status) {
			case 'connected':
				return ['bg-green-100 text-green-800', Check];
			case 'disconnected':
				return ['bg-gray-100 text-gray-800', X];
			case 'connecting':
				return ['bg-yellow-100 text-yellow-800', Loader];
			case 'error':
				return ['bg-red-100 text-red-800', Ban];
		}
	}
</script>

<div class="rounded-lg p-3 {colorClass} flex flex-col gap-2">
	<span class="flex gap-2 font-semibold">
		<Icon /> Connection Status: {status}
	</span>
	{#if showRetryButton}
		<button
			onclick={() => {
				connect();
			}}
			class="mt-2 flex items-center justify-center gap-1 rounded bg-blue-500 px-3 py-1 text-xs font-medium text-white transition-colors hover:bg-blue-600"
		>
			<RotateCcw size={12} />
			Retry Connection
		</button>
	{/if}

	{#if showConnectButton}
		<button
			onclick={() => {
				connect();
			}}
			class="mt-2 flex items-center justify-center gap-1 rounded bg-blue-500 px-3 py-1 text-xs font-medium text-white transition-colors hover:bg-blue-600"
		>
			<Unlink2 />
			Connect
		</button>
	{/if}
</div>
