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
				legend: {
					show: true,
					top: 'top',
					data: ['Left Count', 'Right Count']
				},
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
						name: 'Left Count',
						data: leftCounts.map((v, i) => [i, v]),
						type: 'line'
					},
					{
						id: 'right',
						name: 'Right Count',
						data: rightCounts.map((v, i) => [i, v]),
						type: 'line'
					},
					{
						id: 'leftTarget',
						name: 'Left Target',
						data: [
							[0, leftTarget],
							[leftCounts.length, leftTarget]
						],
						type: 'line',
						itemStyle: {
							color: '#5470c6'
						},
						lineStyle: {
							type: 'dashed'
						}
					},
					{
						id: 'rightTarget',
						name: 'Right Target',
						data: [
							[0, rightTarget],
							[rightCounts.length, rightTarget]
						],
						type: 'line',

						itemStyle: {
							color: '#91cc75'
						},
						lineStyle: {
							type: 'dashed'
						}
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
					{ id: 'left', name: 'Left Count', data: leftCounts.map((v, i) => [i, v]) },
					{ id: 'right', name: 'Right Count', data: rightCounts.map((v, i) => [i, v]) },
					{
						id: 'leftTarget',
						name: 'Left Target',
						data: [
							[0, leftTarget],
							[leftCounts.length, leftTarget]
						],
						itemStyle: {
							color: '#5470c6'
						},
						lineStyle: {
							type: 'dashed'
						}
					},
					{
						id: 'rightTarget',
						name: 'Right Target',
						data: [
							[0, rightTarget],
							[rightCounts.length, rightTarget]
						],
						itemStyle: {
							color: '#91cc75'
						},
						lineStyle: {
							type: 'dashed'
						}
					}
				]
			},
			{ notMerge: false, lazyUpdate: false }
		);
	});
</script>

<div class="h-full w-full" bind:this={chartDiv}></div>
