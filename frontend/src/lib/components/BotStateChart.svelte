<script lang="ts">
	import { onMount } from 'svelte';
	import * as echarts from 'echarts';

	type Props = {
		leftCounts: number[];
		rightCounts: number[];
		leftTarget: number;
		rightTarget: number;
		// gyroX: number[];
		// gyroY: number[];
		// gyroZ: number[];
	};

	let { leftCounts, rightCounts, leftTarget, rightTarget }: Props = $props();

	let chartDiv: HTMLDivElement | null = $state(null);
	let chart: echarts.ECharts | null = $state(null);

	onMount(() => {
		if (chartDiv) {
			chart = echarts.init(chartDiv);
			chart.setOption({
				xAxis: {
					name: 'Time',
					show: false,
					min: 'dataMin',
					max: 'dataMax'
				},
				yAxis: {
					name: 'Count'
				},
				series: [
					{
						id: 'left',
						data: leftCounts.map((v, i) => [i, v]),
						type: 'line'
					},
					{
						id: 'right',
						data: rightCounts.map((v, i) => [i, v]),
						type: 'line'
					},
					{
						id: 'leftTarget',
						data: [
							[0, leftTarget],
							[leftCounts.length, leftTarget]
						],
						type: 'line'
					},
					{
						id: 'rightTarget',
						data: [
							[0, rightTarget],
							[rightCounts.length, rightTarget]
						],
						type: 'line'
					}
				]
			});

			const resize = () => chart!.resize();
			window.addEventListener('resize', resize);

			return () => {
				window.removeEventListener('resize', resize);
				chart!.dispose();
			};
		}
	});

	$effect(() => {
		chart?.setOption(
			{
				series: [
					{ id: 'left', data: leftCounts.map((v, i) => [i, v]) },
					{ id: 'right', data: rightCounts.map((v, i) => [i, v]) },
					{
						id: 'leftTarget',
						data: [
							[0, leftTarget],
							[leftCounts.length, leftTarget]
						]
					},
					{
						id: 'rightTarget',
						data: [
							[0, rightTarget],
							[rightCounts.length, rightTarget]
						]
					}
				]
			},
			{ notMerge: false, lazyUpdate: false }
		);
	});
</script>

<div class="h-full w-full" bind:this={chartDiv}></div>
