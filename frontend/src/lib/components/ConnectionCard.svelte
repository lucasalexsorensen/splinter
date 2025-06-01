<script lang="ts">
	import { Check, X, Loader, Ban, RotateCcw } from '@lucide/svelte';
	import type { Component } from 'svelte';
	import type { ConnectionState } from '$lib/types';
	type Props = {
		status: ConnectionState;
		onRetry?: () => void;
		wsUrl: string;
	};

	let { status, onRetry, wsUrl }: Props = $props();

	const [colorClass, Icon] = $derived(statusToColorClassAndIcon(status));
	const showRetryButton = $derived(status === 'disconnected' || status === 'error');

	function statusToColorClassAndIcon(status: ConnectionState): [string, Component] {
		switch (status) {
			case 'connected':
				return ['bg-green-100 text-green-800', Check];
			case 'disconnected':
				return ['bg-red-100 text-red-800', X];
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
	<span class="mt-1 text-xs">Target: {wsUrl}</span>
	{#if showRetryButton && onRetry}
		<button
			onclick={onRetry}
			class="mt-2 flex items-center justify-center gap-1 rounded bg-blue-500 px-3 py-1 text-xs font-medium text-white transition-colors hover:bg-blue-600"
		>
			<RotateCcw size={12} />
			Retry Connection
		</button>
	{/if}
</div>
