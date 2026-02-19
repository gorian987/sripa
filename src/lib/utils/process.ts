import { BlobImage, CanvasImage, FilterImage } from '$wasm';

export type ColorProcess = ((src: CanvasImage) => FilterImage) | undefined;
export type FilterProcess = ((src: FilterImage) => FilterImage) | undefined
export type BlobProcess = ((src: FilterImage) => BlobImage) | undefined;
export type CropProcess = ((src: CanvasImage, mode: number, rf?: BlobImage) => CanvasImage) | undefined;