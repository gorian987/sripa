<script lang="ts">
	import type { ClassValue } from 'tailwind-variants';
	import { cn } from '$lib/utils';
	import { type CropProcess } from '$lib/utils/process';

	import { Checkbox } from '$lib/components/ui/checkbox';
	import * as Select from '$lib/components/ui/select';
	import Parameter from '$lib/components/Parameter.svelte';

	interface Props {
		process: CropProcess;
		class?: ClassValue;
	}

	const names = ['Image', 'Blob'];
	const items = names.map((name) => ({ value: name, label: name }));

	let { process = $bindable(), class: className }: Props = $props();

	let enabled = $state(false);
	let center = $state(names[0]);
	let width = $state(50);
	let height = $state(50);

	function createProcess(c: string, w: number, h: number): CropProcess {
		return (src, mode, rf) => {
			let cx, cy;

			if (rf && c === names[1]) {
				const pt = rf.blob_center();
				cx = pt.x;
				cy = pt.y;
				console.log('(%d, %d)', cx, cy);
			} else {
				cx = (src.width - 1) / 2;
				cy = (src.height - 1) / 2;
			}

			switch (mode) {
				case 0:
					return src.crop(cx, cy, w, h);
				case 1:
					return src.draw_crop_area(cx, cy, w, h);
				default:
					return src;
			}
		};
	}

	$effect(() => {
		if (enabled) {
			process = createProcess(center, width, height);
		} else {
			process = undefined;
		}
	});
</script>

<div class={cn('flex w-full flex-col', className)}>
	<div class="m-2 text-xl font-bold">Crop</div>
	<div class="flex w-full items-center gap-2 p-4">
		<div>enable</div>
		<Checkbox bind:checked={enabled} />
	</div>

	{#if enabled}
		<Select.Root type="single" bind:value={center}>
			<Select.Trigger class="m-2 font-bold">
				{center}
			</Select.Trigger>
			<Select.Content>
				<Select.Group>
					<Select.Label>Position type</Select.Label>
					{#each items as item}
						<Select.Item value={item.value} label={item.label}>
							{item.label}
						</Select.Item>
					{/each}
				</Select.Group>
			</Select.Content>
		</Select.Root>
		<Parameter label="width" bind:value={width} max={100} min={0} step={0.1} />
		<Parameter label="height" bind:value={height} max={100} min={0} step={0.1} />
	{/if}
</div>
