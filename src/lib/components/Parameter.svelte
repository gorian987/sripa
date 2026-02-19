<script lang="ts">
	import type { ClassValue } from 'tailwind-variants';
	import { cn } from '$lib/utils';

	import { Input } from '$lib/components/ui/input';
	import { Slider } from '$lib/components/ui/slider';

	interface Props {
		label: string;
		value: number;
		max?: number;
		min?: number;
		step?: number;
		class?: ClassValue;
	}

	let {
		label,
		value = $bindable(),
		max = 100,
		min = 0,
		step = 1,
		class: className
	}: Props = $props();
</script>

<div class={cn('flex w-full flex-col items-start p-4', className)}>
	<div>{label}</div>
	<div class="flex w-full gap-4">
		<Slider type="single" class="flex-10" bind:value {max} {min} {step} />
		<Input
			type="number"
			{value}
			class="flex-2 text-center"
			{max}
			{min}
			{step}
			onchange={(e) => {
				value = parseFloat(e.currentTarget.value);
			}}
		/>
	</div>
</div>
