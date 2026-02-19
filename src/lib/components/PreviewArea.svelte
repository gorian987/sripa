<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import type { ClassValue } from 'tailwind-variants';
	import { cn } from '$lib/utils';
	import { errAsync, okAsync, ResultAsync } from 'neverthrow';
	import JSZip from 'jszip';
	import init, { BlobImage, CanvasImage, FilterImage, type InitOutput } from '$wasm';

	import { Button } from '$lib/components/ui/button';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import * as Select from '$lib/components/ui/select';
	import Icon from '@iconify/svelte';
	import Dropzone from '$lib/components/Dropzone.svelte';
	import type { BlobProcess, ColorProcess, CropProcess, FilterProcess } from '$lib/utils/process';

	interface Props {
		colorProcess: ColorProcess;
		filterProcess: FilterProcess;
		blobProcess: BlobProcess;
		cropProcess: CropProcess;
		class?: ClassValue;
	}

	const previewSize = 640;
	const maxSize = 5120;
	const offscreen = new OffscreenCanvas(maxSize, maxSize);
	const offCtx = offscreen.getContext('2d', { willReadFrequently: true });

	const names = ['Color', 'Gray', 'Filter'];
	const items = names.map((name) => ({ value: name, label: name }));

	let {
		colorProcess,
		filterProcess,
		blobProcess,
		cropProcess,
		class: className = ''
	}: Props = $props();

	let files: File[] = $state([]);
	let urls: string[] = $state([]);
	let selectedNo = $state(0);

	let downloadType = $state(names[0]);

	let imageCanvas: HTMLCanvasElement | undefined = $state();
	let blobCanvas: HTMLCanvasElement | undefined = $state();

	let wasm: InitOutput | undefined = $state();
	let sourceImage: CanvasImage | undefined = $state();
	let coloredImage: FilterImage | undefined = $state();
	let filteredImage: FilterImage | undefined = $state();
	let blobImage: BlobImage | undefined = $state();

	function createCanvasImage(
		file: File,
		maxWidth: number,
		maxHeight: number
	): ResultAsync<CanvasImage, string> {
		if (!wasm) {
			return errAsync('WASM has not yet initialized!');
		}
		if (!offCtx) {
			return errAsync('OffScreen does not exist!');
		}

		return ResultAsync.fromPromise(
			createImageBitmap(file),
			(err) => `Selected file is not image!: ${err}`
		).map((bitmap) => {
			const scale = Math.min(maxWidth / bitmap.width, maxHeight / bitmap.height);
			const ratio = Math.min(scale, 1.0);

			const width = Math.floor(bitmap.width * ratio);
			const height = Math.floor(bitmap.height * ratio);

			// ピクセルデータに変換
			offCtx.drawImage(bitmap, 0, 0, width, height);
			const srcData = offCtx.getImageData(0, 0, width, height);

			// JS -> WASM
			const src = new CanvasImage(width, height);
			const srcView = new Uint8ClampedArray(
				wasm!.memory.buffer, // Wasm の全メモリ
				src.ptr, // 画像データの開始位置
				width * height * 4 // 長さ
			);
			srcView.set(srcData.data);

			return src;
		});
	}

	function downloadImage() {
		if (!wasm || files.length === 0) {
			return;
		}

		const zip = new JSZip();
		const folder = zip.folder('processed_images');

		if (!folder) {
			return;
		}

		const tasks = files.map((file) => {
			return createCanvasImage(file, maxSize, maxSize)
				.andThen((source) => {
					const colored = colorProcess?.(source);
					const filtered = colored ? filterProcess?.(colored) : undefined;
					const blob = filtered
						? blobProcess?.(filtered)
						: colored
							? blobProcess?.(colored)
							: undefined;

					let image: CanvasImage | undefined = undefined;
					switch (downloadType) {
						case names[0]:
							image = source.clone();
							break;
						case names[1]:
							image = colored?.to_canvas();
							break;
						case names[2]:
							image = filtered?.to_canvas();
							break;
						default:
							image = source.clone();
							break;
					}

					if (!image) {
						return errAsync(downloadType + ' image is undefined.');
					}

					if (cropProcess) {
						image = cropProcess(image, 0, blob);
					}

					// WASM -> Canvas
					const view = new Uint8ClampedArray(
						wasm!.memory.buffer, // Wasm の全メモリ
						image.ptr, // 画像データの開始位置
						image.width * image.height * 4 // 長さ
					);
					const data = new ImageData(view, image.width, image.height);

					const dstCanvas = new OffscreenCanvas(data.width, data.height);
					const dstCtx = dstCanvas.getContext('2d');
					dstCtx?.putImageData(data, 0, 0);

					// WASMのメモリ解放
					image.free();
					blob?.free();
					filtered?.free();
					colored?.free();
					source?.free();

					// バイナリデータに変換
					return ResultAsync.fromPromise(
						dstCanvas.convertToBlob({ type: 'image/bmp' }),
						(err) => `${file.name} cannot be converted to BMP: ${err}`
					);
				})
				.map((blob) => {
					folder.file(file.name, blob);
				})
				.orElse((err) => {
					console.warn(err);
					return okAsync();
				});
		});

		ResultAsync.combine(tasks)
			.andThen(() =>
				ResultAsync.fromPromise(
					// URLを生成
					zip.generateAsync({ type: 'blob' }),
					(err) => `Zip is failed: ${err}`
				)
			)
			.match(
				(content) => {
					// ダウンロード
					const url = URL.createObjectURL(content);
					const link = document.createElement('a');

					link.href = url;
					link.download = 'processed_images.zip';
					link.click();

					// メモリ解放
					URL.revokeObjectURL(url);
				},
				(err) => console.log(err)
			);
	}

	onMount(async () => {
		// WASMの初期化
		wasm = await init();
	});

	// Update sourceImage
	$effect(() => {
		if (!wasm || !files[selectedNo]) {
			return;
		}

		createCanvasImage(files[selectedNo], previewSize, previewSize).match(
			(image) => {
				sourceImage = image;
			},
			(err) => {
				console.error(err);
				sourceImage = undefined;
			}
		);

		return () => {
			sourceImage?.free();
			sourceImage = undefined;
		};
	});

	// Update coloredImage
	$effect(() => {
		if (!sourceImage) {
			return;
		}

		coloredImage = colorProcess?.(sourceImage);

		return () => {
			coloredImage?.free();
			coloredImage = undefined;
		};
	});

	// Update filteredImage
	$effect(() => {
		if (!coloredImage) {
			return;
		}

		filteredImage = filterProcess?.(coloredImage);

		return () => {
			filteredImage?.free();
			filteredImage = undefined;
		};
	});

	// Update blobImage
	$effect(() => {
		if (filteredImage) {
			blobImage = blobProcess?.(filteredImage);
		} else if (coloredImage) {
			blobImage = blobProcess?.(coloredImage);
		} else {
			return;
		}

		return () => {
			blobImage?.free();
			blobImage = undefined;
		};
	});

	// Display preview
	$effect(() => {
		if (!wasm || !imageCanvas || !blobCanvas) {
			return;
		}

		let image: CanvasImage | undefined = undefined;
		let blob: CanvasImage | undefined = undefined;

		// image
		if (filteredImage) {
			image = filteredImage.to_canvas();
		} else if (coloredImage) {
			image = coloredImage.to_canvas();
		} else if (sourceImage) {
			image = sourceImage.clone();
		} else {
			imageCanvas.getContext('2d')?.reset();
			return;
		}

		if (cropProcess) {
			image = cropProcess(image, 1, blobImage);
		}

		const imageView = new Uint8ClampedArray(
			wasm.memory.buffer, // Wasm の全メモリ
			image.ptr, // 画像データの開始位置
			image.width * image.height * 4 // 長さ
		);
		const imageData = new ImageData(imageView, image.width, image.height);

		imageCanvas.width = imageData.width;
		imageCanvas.height = imageData.height;
		imageCanvas.getContext('2d')?.putImageData(imageData, 0, 0);

		// blob
		if (blobImage) {
			blob = blobImage.to_canvas();
		} else {
			blobCanvas.getContext('2d')?.reset();
			return () => {
				image.free();
			};
		}

		const blobView = new Uint8ClampedArray(
			wasm.memory.buffer, // Wasm の全メモリ
			blob.ptr, // 画像データの開始位置
			blob.width * blob.height * 4 // 長さ
		);
		const blobData = new ImageData(blobView, blob.width, blob.height);

		blobCanvas.width = blobData.width;
		blobCanvas.height = blobData.height;
		blobCanvas.getContext('2d')?.putImageData(blobData, 0, 0);

		return () => {
			image.free();
			blob.free();
		};
	});

	// // プレビュー表示
	// $effect(() => {
	// 	if (!files[selectedNo]) return;

	// 	const index = selectedNo;
	// 	const color = colorProcess;
	// 	const filter = filterProcess;
	// 	const blob = blobProcess;

	// 	untrack(() => {
	// 		createCanvasImage(files[index], previewSize, previewSize).match(
	// 			(canvasImage) => {
	// 				if (!imageCanvas) return;
	// 				if (!blobCanvas) return;

	// 				let image: CanvasImage = canvasImage;
	// 				let blob: CanvasImage | undefined = undefined;

	// 				// WASMで画像処理
	// 				if (colorProcess) {
	// 					const color = colorProcess(canvasImage);
	// 					if (filterProcess) {
	// 						const filter = filterProcess(color);
	// 						if (blobProcess) {
	// 							const blob = blobProcess(filter);
	// 							blob = blob.into_canvas();
	// 						}
	// 						image = filter.into_canvas();
	// 					} else {
	// 						image = color.into_canvas();
	// 					}
	// 				}

	// 				// WASM -> Image Canvas
	// 				const imageView = new Uint8ClampedArray(
	// 					wasm.memory.buffer, // Wasm の全メモリ
	// 					image.ptr, // 画像データの開始位置
	// 					image.width * image.height * 4 // 長さ
	// 				);
	// 				const imageData = new ImageData(imageView, image.width, image.height);

	// 				imageCanvas.width = imageData.width;
	// 				imageCanvas.height = imageData.height;
	// 				imageCanvas.getContext('2d')?.putImageData(imageData, 0, 0);

	// 				image.free();

	// 				// WASM -> Draw Canvas
	// 				if (!blob) {
	// 					blobCanvas.getContext('2d')?.reset();
	// 					return;
	// 				}

	// 				const blobView = new Uint8ClampedArray(
	// 					wasm.memory.buffer, // Wasm の全メモリ
	// 					blob.ptr, // 画像データの開始位置
	// 					blob.width * blob.height * 4 // 長さ
	// 				);
	// 				const blobData = new ImageData(blobView, blob.width, blob.height);

	// 				blobCanvas.width = blobData.width;
	// 				blobCanvas.height = blobData.height;
	// 				blobCanvas.getContext('2d')?.putImageData(blobData, 0, 0);

	// 				blob.free();
	// 			},
	// 			(err) => {
	// 				console.log(err);
	// 				imageCanvas?.getContext('2d')?.reset();
	// 				blobCanvas?.getContext('2d')?.reset();
	// 			}
	// 		);
	// 	});
	// });

	// サムネイル表示
	$effect(() => {
		files.forEach((file, index) => {
			urls[index] = URL.createObjectURL(file);
		});

		return () => {
			urls.forEach((src) => {
				URL.revokeObjectURL(src);
			});
			selectedNo = 0;
		};
	});
</script>

<div class={cn('flex h-full flex-col items-center gap-4', className)}>
	<Dropzone bind:files class="w-full flex-2 border-2 border-dashed bg-gray-100" />
	<div class="flex min-h-0 w-full flex-6">
		<div class="flex h-full min-w-0 flex-5 flex-col items-center">
			<img
				src={urls[selectedNo]}
				class="mt-0 mb-0 aspect-square min-h-0 w-full flex-1 object-contain"
				alt="/"
			/>
			<div class="w-full text-center font-bold">Before</div>
		</div>
		<Icon class="h-full min-w-0 flex-2 " icon="material-symbols:arrow-right" />
		<div class="flex h-full min-w-0 flex-5 flex-col items-center">
			<div class="relative flex min-h-0 w-full flex-1 items-center">
				<canvas
					bind:this={imageCanvas}
					class="absolute z-0 aspect-square h-full w-full object-contain"
				></canvas>
				<canvas
					bind:this={blobCanvas}
					class="absolute z-10 aspect-square h-full w-full object-contain"
				></canvas>
			</div>
			<div class="w-full text-center font-bold">After</div>
		</div>
	</div>
	<div class="flex w-full items-center justify-center">
		<Select.Root type="single" bind:value={downloadType}>
			<Select.Trigger class="m-2 font-bold">
				{downloadType}
			</Select.Trigger>
			<Select.Content>
				<Select.Group>
					<Select.Label>Image type</Select.Label>
					{#each items as item}
						<Select.Item value={item.value} label={item.label}>
							{item.label}
						</Select.Item>
					{/each}
				</Select.Group>
			</Select.Content>
		</Select.Root>
		<Button onclick={downloadImage}>Download</Button>
	</div>
	<ScrollArea class="min-h-0 w-full flex-4">
		<div class="grid w-full grid-cols-10">
			{#each files as file, index}
				<button
					class="flex flex-col border p-2 hover:bg-blue-100"
					onclick={() => {
						selectedNo = index;
					}}
				>
					<img src={urls[index]} class="mt-0 mb-0 aspect-square w-full object-contain" alt="/" />
					<div class="w-full truncate text-center text-xs">{file.name}</div>
				</button>
			{/each}
		</div>
	</ScrollArea>
</div>
