<script lang="ts">
  import { Chart } from "svelte-echarts";
  import { init, use } from "echarts/core";
  import { LineChart } from "echarts/charts";
  import {
    GridComponent,
    TitleComponent,
    LegendComponent,
  } from "echarts/components";
  import { CanvasRenderer } from "echarts/renderers";
  import { onDestroy, onMount } from "svelte";
  import { parsePayload } from "./lib/parsing";

  use([
    LineChart,
    GridComponent,
    CanvasRenderer,
    TitleComponent,
    LegendComponent,
  ]);

  // plot the last MAX_COUNT counts
  const MAX_COUNT = 100;
  let left_counts = $state<number[]>(Array(MAX_COUNT).fill(0));
  let right_counts = $state<number[]>(Array(MAX_COUNT).fill(0));
  let left_target = $state<number>(0);
  let right_target = $state<number>(0);

  const options = $derived({
    legend: {
      data: ["Left Count", "Right Count", "Left Target", "Right Target"],
      top: 10,
    },
    xAxis: {
      type: "category",
      show: false,
      data: left_counts.map((_, i) => i),
    },
    yAxis: {
      type: "value",
    },
    series: [
      {
        name: "Left Count",
        data: left_counts,
        type: "line",
        color: "#3b82f6",
      },
      {
        name: "Right Count",
        data: right_counts,
        type: "line",
        color: "#10b981",
      },
      {
        name: "Left Target",
        type: "line",
        data: [
          [0, left_target],
          [MAX_COUNT, left_target],
        ],
      },
      {
        name: "Right Target",
        type: "line",
        data: [
          [0, right_target],
          [MAX_COUNT, right_target],
        ],
      },
    ],
  });

  let ws: WebSocket | null = $state(null);
  let connectionState = $state<string>("DISCONNECTED");
  let lastError = $state<string>("");

  onMount(() => {
    ws = new WebSocket("ws://192.168.1.243:9999");
    ws.binaryType = "arraybuffer";
    connectionState = "CONNECTING";

    ws.onopen = function (event) {
      connectionState = "CONNECTED";
      lastError = "";
    };

    ws.onmessage = function (event) {
      const bytes = new Uint8Array(event.data);
      const parsed = parsePayload(bytes);
      left_counts.push(parsed.left_count);
      right_counts.push(parsed.right_count);
      left_counts = left_counts.slice(-MAX_COUNT);
      right_counts = right_counts.slice(-MAX_COUNT);
      left_target = parsed.left_target;
      right_target = parsed.right_target;
    };

    ws.onerror = function (event) {
      console.error("WebSocket error:", event);
      lastError = "Connection error occurred";
      connectionState = "ERROR";
    };

    ws.onclose = function (event) {
      connectionState = "DISCONNECTED";
      if (event.code !== 1000) {
        lastError = `Connection closed unexpectedly (code: ${event.code})`;
      }
    };
  });

  onDestroy(() => {
    if (ws) {
      connectionState = "DISCONNECTING";
      ws.close();
    }
  });

  function turnLeft() {
    if (ws?.readyState === WebSocket.OPEN) {
      const data = new Uint8Array([1]);
      ws?.send(data);
    } else {
      console.warn("WebSocket not connected, cannot send turn left command");
    }
  }

  function turnRight() {
    if (ws?.readyState === WebSocket.OPEN) {
      const data = new Uint8Array([2]);
      ws?.send(data);
    } else {
      console.warn("WebSocket not connected, cannot send turn right command");
    }
  }
</script>

<div
  class="flex flex-col items-center justify-center h-screen bg-gray-100 p-12"
>
  <Chart {init} options={options as any} />

  <hr />

  <!-- Connection Status -->
  <div
    class="mb-4 p-3 rounded-lg {connectionState === 'CONNECTED'
      ? 'bg-green-100 text-green-800'
      : connectionState === 'CONNECTING'
        ? 'bg-yellow-100 text-yellow-800'
        : 'bg-red-100 text-red-800'}"
  >
    <div class="font-semibold">Connection Status: {connectionState}</div>
    {#if lastError}
      <div class="text-sm mt-1">Error: {lastError}</div>
    {/if}
    <div class="text-xs mt-1">Target: ws://192.168.1.243:9999</div>
  </div>

  <h2 class="text-3xl font-bold text-gray-800 mb-8">Robot Control</h2>

  <div class="flex gap-4">
    <button
      onclick={turnLeft}
      disabled={connectionState !== "CONNECTED"}
      class="px-6 py-3 bg-blue-500 hover:bg-blue-600 disabled:bg-gray-400 disabled:cursor-not-allowed text-white font-semibold rounded-lg shadow-md transition-colors duration-200 flex items-center gap-2"
    >
      <span class="text-xl">←</span>
      Turn Left
    </button>

    <button
      onclick={turnRight}
      disabled={connectionState !== "CONNECTED"}
      class="px-6 py-3 bg-green-500 hover:bg-green-600 disabled:bg-gray-400 disabled:cursor-not-allowed text-white font-semibold rounded-lg shadow-md transition-colors duration-200 flex items-center gap-2"
    >
      Turn Right
      <span class="text-xl">→</span>
    </button>
  </div>
</div>
