<script lang="ts">
	import { greet } from '../wasm/pkg/wasm';
	import svelteLogo from './assets/svelte.svg';
	import viteLogo from '/vite.svg';
	import Counter from '$lib/components/Counter.svelte';

	import { Separator } from '$lib/components/ui/separator';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';

	let name = $state('ユーザー');
	let message = $state('');

	async function handleGreet() {
		// Rust関数の実行
		message = greet(name);
	}
</script>

<main class="flex h-screen min-h-150 w-screen min-w-200 flex-none gap-4">
	<div class="prose flex flex-col items-center">
		<div class="flex items-center gap-16">
			<a href="https://vite.dev" target="_blank" rel="noreferrer">
				<img src={viteLogo} class="w-32" alt="Vite Logo" />
			</a>
			<a href="https://svelte.dev" target="_blank" rel="noreferrer">
				<img src={svelteLogo} class="w-32" alt="Svelte Logo" />
			</a>
		</div>

		<h1>Vite + Svelte</h1>
		<Counter />

		<p>
			Check out <a href="https://github.com/sveltejs/kit#readme" target="_blank" rel="noreferrer"
				>SvelteKit</a
			>, the official Svelte app framework powered by Vite!
		</p>
		<p class="read-the-docs">Click on the Vite and Svelte logos to learn more</p>

		<Separator class="my-4" />

		<h1>Svelte + Rust WASM</h1>
		<div class="flex">
			<Input bind:value={name} placeholder="名前を入力" />
			<Button onclick={handleGreet}>Rustを呼ぶ</Button>
		</div>
		<p>{message}</p>
	</div>
</main>
