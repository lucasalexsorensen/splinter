<script lang="ts">
	import { onMount } from 'svelte';
	import * as echarts from 'echarts';

	type Props = {
		yaw: number[];
		pitch: number[];
		roll: number[];
	};

	let { yaw, pitch, roll }: Props = $props();

	let chartDiv: HTMLDivElement | null = $state(null);
	let chart: echarts.ECharts | null = $state(null);

	onMount(() => {
		if (!chartDiv) return;

		chart = echarts.init(chartDiv);
		chart.setOption({
			legend: {
				show: true,
				bottom: 0,
				data: ['Yaw', 'Pitch', 'Roll']
			},
			tooltip: {
				trigger: 'axis',
				formatter: (params: echarts.DefaultLabelFormatterCallbackParams[]) => {
					let result = '';
					for (const param of params) {
						const value = ((param.data as number[])[1] / 100).toFixed(2);
						result += `${param.marker} ${param.seriesName}: ${value}°<br/>`;
					}
					return result;
				}
			},
			xAxis: {
				name: 'Time',
				show: false,
				min: 'dataMin',
				max: 'dataMax'
			},
			yAxis: {
				name: 'Degrees (×100)',
				axisLabel: {
					formatter: (value: number) => (value / 100).toFixed(0) + '°'
				}
			},
			series: [
				{
					id: 'yaw',
					name: 'Yaw',
					data: yaw.map((v, i) => [i, v]),
					type: 'line',
					smooth: true,
					itemStyle: { color: '#ee6666' }
				},
				{
					id: 'pitch',
					name: 'Pitch',
					data: pitch.map((v, i) => [i, v]),
					type: 'line',
					smooth: true,
					itemStyle: { color: '#91cc75' }
				},
				{
					id: 'roll',
					name: 'Roll',
					data: roll.map((v, i) => [i, v]),
					type: 'line',
					smooth: true,
					itemStyle: { color: '#5470c6' }
				}
			]
		});

		const resize = () => chart!.resize();
		window.addEventListener('resize', resize);

		return () => {
			window.removeEventListener('resize', resize);
			chart!.dispose();
		};
	});

	$effect(() => {
		chart?.setOption(
			{
				series: [
					{ id: 'yaw', name: 'Yaw', data: yaw.map((v, i) => [i, v]) },
					{ id: 'pitch', name: 'Pitch', data: pitch.map((v, i) => [i, v]) },
					{ id: 'roll', name: 'Roll', data: roll.map((v, i) => [i, v]) }
				]
			},
			{ notMerge: false, lazyUpdate: false }
		);
	});
</script>

<div class="h-full w-full" bind:this={chartDiv}></div>
