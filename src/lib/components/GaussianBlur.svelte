<script lang="ts">
	import type { ClassValue } from 'tailwind-variants';
	import { cn } from '$lib/utils';
	import type { FilterProcess } from '$lib/utils/process';

	import Parameter from '$lib/components/Parameter.svelte';

	interface Props {
		process: FilterProcess;
		class?: ClassValue;
	}

	let { process = $bindable(), class: className }: Props = $props();

	let kernelX = $state(11);
	let kernelY = $state(11);
	let sigmaX = $state(2.0);
	let sigmaY = $state(2.0);

	function createProcess(kx: number, ky: number, sx: number, sy: number): FilterProcess {
		return (src) => {
			return src.gaussian_blur(kx, ky, sx, sy);
		};
	}

	$effect(() => {
		process = createProcess(kernelX, kernelY, sigmaX, sigmaY);
	});
</script>

<div class={cn('flex w-full flex-col', className)}>
	<Parameter label="kernel x" bind:value={kernelX} max={51} min={1} step={2} />
	<Parameter label="kernel y" bind:value={kernelY} max={51} min={1} step={2} />
	<Parameter label="sigma x" bind:value={sigmaX} max={10.0} min={0.1} step={0.1} />
	<Parameter label="sigma y" bind:value={sigmaY} max={10.0} min={0.1} step={0.1} />
</div>
