<script lang="ts">
	import { cn } from '$lib/utils';
	import { onMount } from 'svelte';
	import type { ClassValue } from 'tailwind-variants';

	import Icon from '@iconify/svelte';

	interface Props {
		files: File[];
		message?: boolean;
		class?: ClassValue;
	}

	let { files = $bindable([]), message = true, class: className = '' }: Props = $props();
	let isHover = $state(false);

	function on_file_select(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.files && 0 < target.files.length) {
			files = Array.from(target.files);
		}
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		isHover = true;
	}

	function handleDragLeave() {
		isHover = false;
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		isHover = false;

		if (e.dataTransfer?.files) {
			files = Array.from(e.dataTransfer.files);
		}
	}

	onMount(() => {
		let e = document.getElementById('dropzone');

		e?.addEventListener('dragover', handleDragOver);
		e?.addEventListener('dragleave', handleDragLeave);
		e?.addEventListener('drop', handleDrop);
	});
</script>

<div
	id="dropzone"
	class={cn(
		'relative flex min-h-16 min-w-64 flex-col items-center justify-center hover:bg-blue-100',
		isHover ? 'bg-blue-100 ' : '',
		className
	)}
>
	<input class="absolute inset-0 z-10 opacity-0" type="file" multiple onchange={on_file_select} />
	{#if message}
		<Icon width="32" icon="material-symbols:download" />
		<div class="font-bold">Click or Drag & Drop</div>
	{/if}
</div>
