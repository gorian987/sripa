<script lang="ts">
	import { Separator } from '$lib/components/ui/separator';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import PreviewArea from '$lib/components/PreviewArea.svelte';
	import ColorSetting from '$lib/components/ColorSetting.svelte';
	import FilterSetting from '$lib/components/FilterSetting.svelte';
	import BlobSetting from '$lib/components/BlobSetting.svelte';
	import CropSetting from '$lib/components/CropSetting.svelte';

	let colorProcess = $state(undefined);
	let filterProcess = $state(undefined);
	let blobProcess = $state(undefined);
	let cropProcess = $state(undefined);
</script>

<main class="flex h-screen min-h-150 w-screen min-w-200 flex-none gap-4">
	<PreviewArea class="flex-2" {colorProcess} {filterProcess} {blobProcess} {cropProcess} />
	<ScrollArea class="h-full flex-1 border">
		<div class="m-2 text-2xl font-bold">Image processing parameters</div>
		<Separator class="my-4" />
		<ColorSetting bind:process={colorProcess} />
		{#if colorProcess}
			<Separator class="my-4" />
			<FilterSetting bind:process={filterProcess} />
			<Separator class="my-4" />
			<BlobSetting bind:process={blobProcess} />
		{/if}
		<Separator class="my-4" />
		<CropSetting bind:process={cropProcess} />
	</ScrollArea>
</main>
