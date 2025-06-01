<script lang="ts">
	import { Chart } from 'svelte-echarts';
	import { init, use } from 'echarts/core';
	import { LineChart } from 'echarts/charts';
	import { GridComponent, LegendComponent, TitleComponent } from 'echarts/components';
	import { CanvasRenderer } from 'echarts/renderers';
	import type { EChartsOption } from 'echarts';

	type Props = {
		leftCounts: number[];
		rightCounts: number[];
		leftTarget: number;
		rightTarget: number;
	};

	let { leftCounts, rightCounts, leftTarget, rightTarget }: Props = $props();

	use([LineChart, GridComponent, CanvasRenderer, TitleComponent, LegendComponent]);

	const options: EChartsOption = $derived({
		legend: {
			data: ['Left Count', 'Left Target', 'Right Count', 'Right Target'],
			top: 10
		},
		xAxis: {
			name: 'Time',
			type: 'category',
			show: false,
			data: leftCounts.map((_, i) => i)
		},
		yAxis: {
			name: 'Count',
			type: 'value'
		},
		series: [
			{
				name: 'Left Count',
				data: leftCounts,
				type: 'line',
				color: '#3b82f6'
			},
			{
				name: 'Left Target',
				type: 'line',
				data: [
					[0, leftTarget],
					[leftCounts.length, leftTarget]
				],
				color: '#3b82f6'
			},
			{
				name: 'Right Count',
				data: rightCounts,
				type: 'line',
				color: '#10b981'
			},
			{
				name: 'Right Target',
				type: 'line',
				data: [
					[0, rightTarget],
					[rightCounts.length, rightTarget]
				],
				color: '#10b981'
			}
		]
	});
</script>

<Chart {init} {options} />
