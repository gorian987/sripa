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

	let kernel = $state(3);

	function createProcess(k: number): FilterProcess {
		return (src) => {
			return src.sobel(k);
		};
	}

	$effect(() => {
		process = createProcess(kernel);
	});
</script>

<div class={cn('flex w-full flex-col', className)}>
	<Parameter label="kernel" bind:value={kernel} max={5} min={3} step={2} />
</div>
