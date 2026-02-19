<script lang="ts">
	import type { ClassValue } from 'tailwind-variants';
	import { cn } from '$lib/utils';
	import { type BlobProcess } from '$lib/utils/process';

	import { Checkbox } from '$lib/components/ui/checkbox';
	import Parameter from '$lib/components/Parameter.svelte';

	interface Props {
		process: BlobProcess;
		class?: ClassValue;
	}

	let { process = $bindable(), class: className }: Props = $props();

	let enabled = $state(false);
	let inverse = $state(false);
	let threshold = $state(128);
	let maxBright = $state(255);
	let left = $state(0);
	let top = $state(0);
	let right = $state(100);
	let bottom = $state(100);
	let minSize = $state(0);
	let maxSize = $state(100);

	function createProcess(
		iv: boolean,
		th: number,
		mb: number,
		l: number,
		t: number,
		r: number,
		b: number,
		ls: number,
		us: number
	): BlobProcess {
		return (src) => {
			return src
				.to_blob(iv, th, mb)
				.detect_blob()
				.extract_with_position(l, t, r, b)
				.extract_with_area(ls, us);
		};
	}

	$effect(() => {
		if (enabled) {
			process = createProcess(
				inverse,
				threshold,
				maxBright,
				left,
				top,
				right,
				bottom,
				minSize,
				maxSize
			);
		} else {
			process = undefined;
		}
	});
</script>

<div class={cn('flex w-full flex-col', className)}>
	<div class="m-2 text-xl font-bold">Blob</div>
	<div class="flex w-full items-center gap-2 p-4">
		<div>enable</div>
		<Checkbox bind:checked={enabled} />
	</div>

	{#if enabled}
		<div class="w-full p-2 font-bold underline">Binarization</div>
		<div class="flex w-full items-center gap-2 p-4">
			<div>inverse</div>
			<Checkbox bind:checked={inverse} />
		</div>
		<Parameter label="threshold" bind:value={threshold} max={255} min={0} step={1} />
		<Parameter label="max value" bind:value={maxBright} max={255} min={0} step={1} />

		<div class="w-full p-2 font-bold underline">Detection</div>
		<Parameter label="left" bind:value={left} max={100} min={0} step={0.1} />
		<Parameter label="top" bind:value={top} max={100} min={0} step={0.1} />
		<Parameter label="right" bind:value={right} max={100} min={0} step={0.1} />
		<Parameter label="bottom" bind:value={bottom} max={100} min={0} step={0.1} />
		<Parameter label="min size" bind:value={minSize} max={100} min={0} step={0.1} />
		<Parameter label="max size" bind:value={maxSize} max={100} min={0} step={0.1} />
	{/if}
</div>
