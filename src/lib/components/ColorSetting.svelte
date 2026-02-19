<script lang="ts">
	import type { ClassValue } from 'tailwind-variants';
	import { cn } from '$lib/utils';
	import type { ColorProcess } from '$lib/utils/process';

	import * as Select from '$lib/components/ui/select';

	interface Props {
		process: ColorProcess;
		class?: ClassValue;
	}

	const names = ['Color', 'Grayscale', 'Red', 'Green', 'Blue'];
	const items = names.map((name) => ({ value: name, label: name }));

	let { process = $bindable(), class: className }: Props = $props();
	let select = $state(names[0]);

	function createProcess(name: string): ColorProcess {
		switch (name) {
			case names[0]:
				return undefined;
			case names[1]:
				return (src) => src.to_grayscale();
			case names[2]:
				return (src) => src.to_gray_from_red();
			case names[3]:
				return (src) => src.to_gray_from_green();
			case names[4]:
				return (src) => src.to_gray_from_blue();
			default:
				return undefined;
		}
	}

	$effect(() => {
		process = createProcess(select);
	});
</script>

<div class={cn('flex w-full flex-col', className)}>
	<div class="m-2 text-xl font-bold">Color</div>
	<Select.Root type="single" bind:value={select}>
		<Select.Trigger class="m-2 font-bold">
			{select}
		</Select.Trigger>
		<Select.Content>
			<Select.Group>
				<Select.Label>Color type</Select.Label>
				{#each items as item}
					<Select.Item value={item.value} label={item.label}>
						{item.label}
					</Select.Item>
				{/each}
			</Select.Group>
		</Select.Content>
	</Select.Root>
</div>
