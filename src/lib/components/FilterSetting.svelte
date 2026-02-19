<script lang="ts">
	import type { ClassValue } from 'tailwind-variants';
	import { cn } from '$lib/utils';
	import { type FilterProcess } from '$lib/utils/process';

	import * as Select from '$lib/components/ui/select';
	import GaussianBlur from '$lib/components/GaussianBlur.svelte';
	import Sobel from '$lib/components/Sobel.svelte';

	interface Props {
		process: FilterProcess;
		class?: ClassValue;
	}

	const maxNum = 8;
	const names = ['None', 'Gaussian blur', 'Sobel'];
	const items = names.map((name) => ({ value: name, label: name }));

	let { process = $bindable(), class: className }: Props = $props();

	let processes: FilterProcess[] = $state(new Array(maxNum).fill(undefined));
	let selects: string[] = $state(new Array(maxNum).fill(names[0]));

	function createProcess(procs: FilterProcess[]): FilterProcess {
		if (procs.every((proc) => proc === undefined)) {
			return undefined;
		}

		return (src) => {
			let dst = src.clone();
			for (let proc of procs) {
				if (!proc) {
					continue;
				}
				dst = proc(dst);
			}
			return dst;
		};
	}

	$effect(() => {
		process = createProcess([...processes]);
	});
</script>

<div class={cn('flex w-full flex-col', className)}>
	<div class="m-2 text-xl font-bold">Filter</div>
	{#each { length: maxNum }, i}
		<div class="m-2 flex items-center">
			<div class="font-bold">No. {i + 1}</div>
			<Select.Root type="single" bind:value={selects[i]}>
				<Select.Trigger class="m-2 font-bold">
					{selects[i]}
				</Select.Trigger>
				<Select.Content>
					<Select.Group>
						<Select.Label>Filter type</Select.Label>
						{#each items as item}
							<Select.Item value={item.value} label={item.label}>
								{item.label}
							</Select.Item>
						{/each}
					</Select.Group>
				</Select.Content>
			</Select.Root>
		</div>
		{#if selects[i] === names[0]}
			{(processes[i] = undefined)}
		{:else if selects[i] === names[1]}
			<GaussianBlur bind:process={processes[i]} />
		{:else if selects[i] === names[2]}
			<Sobel bind:process={processes[i]} />
		{/if}
	{/each}
</div>
