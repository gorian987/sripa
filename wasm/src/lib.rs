use image::{GrayImage, ImageBuffer, Luma, Rgba, RgbaImage};
use imageproc::{
    drawing::draw_hollow_rect_mut,
    point::Point,
    rect::Rect,
    region_labelling::{Connectivity, connected_components},
};
use kornia::{
    image::{Image, ImageSize, allocator::CpuAllocator},
    imgproc::{
        crop::crop_image,
        filter,
        threshold::{threshold_binary, threshold_binary_inverse},
    },
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct CanvasImage {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
pub struct FilterImage {
    data: Vec<f32>,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
pub struct BlobImage {
    label_map: ImageBuffer<Luma<u32>, Vec<u32>>,
    blobs: Vec<Vec<Point<u32>>>,
    search_area: Rect,
    crop_area: Rect,
}

#[wasm_bindgen]
pub struct CropRect {
    left: u32,
    top: u32,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
pub struct Center {
    x: f32,
    y: f32,
}

#[wasm_bindgen]
impl CanvasImage {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        CanvasImage {
            data: vec![0; (4 * width * height) as usize],
            width,
            height,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[wasm_bindgen(getter)]
    pub fn ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    #[wasm_bindgen]
    pub fn clone(&self) -> Self {
        CanvasImage {
            data: self.data.clone(),
            width: self.width,
            height: self.height,
        }
    }

    #[wasm_bindgen]
    pub fn to_grayscale(&self) -> FilterImage {
        let gray_data: Vec<f32> = self
            .data
            .chunks_exact(4)
            .flat_map(|rgba| {
                let r = rgba[0] as f32;
                let g = rgba[1] as f32;
                let b = rgba[2] as f32;
                [0.299 * r + 0.587 * g + 0.114 * b]
            })
            .collect();

        FilterImage {
            data: gray_data,
            width: self.width,
            height: self.height,
        }
    }

    #[wasm_bindgen]
    pub fn to_gray_from_red(&self) -> FilterImage {
        self.to_gray_from_channel(0)
    }

    #[wasm_bindgen]
    pub fn to_gray_from_green(&self) -> FilterImage {
        self.to_gray_from_channel(1)
    }

    #[wasm_bindgen]
    pub fn to_gray_from_blue(&self) -> FilterImage {
        self.to_gray_from_channel(2)
    }

    fn to_gray_from_channel(&self, channel: usize) -> FilterImage {
        assert!(channel < 3);

        let gray_data: Vec<f32> = self
            .data
            .chunks_exact(4)
            .flat_map(|rgba| [rgba[channel] as f32])
            .collect();

        FilterImage {
            data: gray_data,
            width: self.width,
            height: self.height,
        }
    }

    #[wasm_bindgen]
    pub fn crop(self, center_x: f32, center_y: f32, width_rate: f32, height_rate: f32) -> Self {
        let rect = self.calculate_crop_area(center_x, center_y, width_rate, height_rate);

        let src_size = ImageSize {
            width: self.width as usize,
            height: self.height as usize,
        };
        let dst_size = ImageSize {
            width: rect.width() as usize,
            height: rect.height() as usize,
        };
        let alloc = CpuAllocator::default();

        let src_img = Image::<u8, 4, _>::new(src_size, self.data, alloc.clone()).unwrap();
        let mut dst_img = Image::<u8, 4, _>::from_size_val(dst_size, 0, alloc).unwrap();

        let _ = crop_image(
            &src_img,
            &mut dst_img,
            rect.left() as usize,
            rect.top() as usize,
        );

        CanvasImage {
            width: dst_img.width() as u32,
            height: dst_img.height() as u32,
            data: dst_img.into_vec(),
        }
    }

    #[wasm_bindgen]
    pub fn draw_crop_area(
        self,
        center_x: f32,
        center_y: f32,
        width_rate: f32,
        height_rate: f32,
    ) -> Self {
        let rect = self.calculate_crop_area(center_x, center_y, width_rate, height_rate);
        let red_pixel = Rgba([255, 0, 0, 255]);
        let mut img = RgbaImage::from_raw(self.width, self.height, self.data).unwrap();

        draw_hollow_rect_mut(&mut img, rect, red_pixel);

        CanvasImage {
            data: img.into_raw(),
            width: self.width,
            height: self.height,
        }
    }

    fn calculate_crop_area(
        &self,
        center_x: f32,
        center_y: f32,
        width_rate: f32,
        height_rate: f32,
    ) -> Rect {
        let width_rate = width_rate.clamp(0.0, 100.0);
        let height_rate = height_rate.clamp(0.0, 100.0);

        let img_width = self.width as f32;
        let img_height = self.height as f32;

        let width = (img_width * width_rate / 100.0)
            .min(2.0 * center_x)
            .min(2.0 * (img_width - center_x));
        let height = (img_height * height_rate / 100.0)
            .min(2.0 * center_y)
            .min(2.0 * (img_height - center_y));

        let left = (center_x - width / 2.0) as i32;
        let top = (center_y - height / 2.0) as i32;

        let width = width.max(1.0) as u32;
        let height = height.max(1.0) as u32;

        Rect::at(left, top).of_size(width, height)
    }
}

#[wasm_bindgen]
impl FilterImage {
    #[wasm_bindgen]
    pub fn clone(&self) -> FilterImage {
        FilterImage {
            data: self.data.clone(),
            width: self.width,
            height: self.height,
        }
    }

    #[wasm_bindgen]
    pub fn into_canvas(self) -> CanvasImage {
        let rgba_data: Vec<u8> = self
            .data
            .iter()
            .flat_map(|&gray| {
                let v = gray as u8;
                [v, v, v, 255]
            })
            .collect();

        CanvasImage {
            data: rgba_data,
            width: self.width,
            height: self.height,
        }
    }

    #[wasm_bindgen]
    pub fn to_canvas(&self) -> CanvasImage {
        let rgba_data: Vec<u8> = self
            .data
            .iter()
            .flat_map(|&gray| {
                let v = gray as u8;
                [v, v, v, 255]
            })
            .collect();

        CanvasImage {
            data: rgba_data,
            width: self.width,
            height: self.height,
        }
    }

    #[wasm_bindgen]
    pub fn to_blob(&self, inverse: bool, threshold: f32, max_value: f32) -> BlobImage {
        let threshold = threshold.clamp(0.0, 255.0);
        let max_value = max_value.clamp(0.0, 255.0);

        let size = ImageSize {
            width: self.width as usize,
            height: self.height as usize,
        };
        let alloc = CpuAllocator::default();
        let src_img = Image::<f32, 1, _>::from_size_slice(size, &self.data, alloc.clone()).unwrap();
        let mut dst_img = Image::<f32, 1, _>::from_size_val(size, 0.0, alloc).unwrap();

        let _ = match inverse {
            false => threshold_binary(&src_img, &mut dst_img, threshold, max_value),
            true => threshold_binary_inverse(&src_img, &mut dst_img, threshold, max_value),
        };

        let gray_data: Vec<u8> = dst_img
            .as_slice()
            .iter()
            .flat_map(|&gray| [gray as u8])
            .collect();

        BlobImage::new(self.width, self.height, gray_data)
    }

    #[wasm_bindgen]
    pub fn gaussian_blur(self, kernel_x: u32, kernel_y: u32, sigma_x: f32, sigma_y: f32) -> Self {
        let kernel_x = kernel_x.max(1);
        let kernel_y = kernel_y.max(1);
        let sigma_x = sigma_x.max(0.0);
        let sigma_y = sigma_y.max(0.0);

        let size = ImageSize {
            width: self.width as usize,
            height: self.height as usize,
        };
        let alloc = CpuAllocator::default();
        let src_img = Image::<f32, 1, _>::new(size, self.data, alloc.clone()).unwrap();
        let mut dst_img = Image::<f32, 1, _>::from_size_val(size, 0.0, alloc).unwrap();

        let _ = filter::gaussian_blur(
            &src_img,
            &mut dst_img,
            (kernel_x as usize, kernel_y as usize),
            (sigma_x, sigma_y),
        );

        let width = dst_img.width() as u32;
        let height = dst_img.height() as u32;
        let data = dst_img.into_vec();

        FilterImage {
            data,
            width,
            height,
        }
    }

    #[wasm_bindgen]
    pub fn sobel(self, kernel: u32) -> Self {
        let kernel = kernel.clamp(3, 5);

        let size = ImageSize {
            width: self.width as usize,
            height: self.height as usize,
        };
        let alloc = CpuAllocator::default();

        let src_img = Image::<f32, 1, _>::new(size, self.data, alloc.clone()).unwrap();
        let mut dst_img = Image::<f32, 1, _>::from_size_val(size, 0.0, alloc).unwrap();
        let _ = filter::sobel(&src_img, &mut dst_img, kernel as usize);

        let width = dst_img.width() as u32;
        let height = dst_img.height() as u32;
        let data = dst_img.into_vec();

        FilterImage {
            data,
            width,
            height,
        }
    }
}

#[wasm_bindgen]
impl BlobImage {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        let img = GrayImage::from_raw(width, height, data).unwrap();
        let background = Luma([0]);
        let label_map = connected_components(&img, Connectivity::Eight, background);

        BlobImage {
            label_map,
            blobs: vec![],
            search_area: Rect::at(0, 0).of_size(width, height),
            crop_area: Rect::at(0, 0).of_size(width, height),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn crop_rect(&self) -> CropRect {
        CropRect {
            left: self.crop_area.left().max(0) as u32,
            top: self.crop_area.top().max(0) as u32,
            width: self.crop_area.width().max(1) as u32,
            height: self.crop_area.height().max(1) as u32,
        }
    }

    #[wasm_bindgen]
    pub fn blob_center(&self) -> Center {
        let mut count = 0.0;
        let mut center_x = 0.0;
        let mut center_y = 0.0;

        self.blobs.iter().for_each(|blob| {
            blob.iter().for_each(|pt| {
                count += 1.0;
                center_x += pt.x as f32;
                center_y += pt.y as f32;
            });
        });

        if 0.0 < count {
            center_x /= count;
            center_y /= count;
        } else {
            center_x = 0.0;
            center_y = 0.0;
        }

        Center {
            x: center_x,
            y: center_y,
        }
    }

    #[wasm_bindgen]
    pub fn into_canvas(self) -> CanvasImage {
        let width = self.label_map.width();
        let height = self.label_map.height();

        let mut rgba_img = RgbaImage::new(width, height);
        let green_pixel = Rgba([0, 255, 0, 255]);

        self.blobs.iter().for_each(|blob| {
            blob.iter()
                .for_each(|pt| rgba_img.put_pixel(pt.x, pt.y, green_pixel))
        });

        let yellow_pixel = Rgba([255, 255, 0, 255]);
        draw_hollow_rect_mut(&mut rgba_img, self.search_area, yellow_pixel);

        CanvasImage {
            data: rgba_img.into_raw(),
            width: width,
            height: height,
        }
    }

    #[wasm_bindgen]
    pub fn to_canvas(&self) -> CanvasImage {
        let width = self.label_map.width();
        let height = self.label_map.height();

        let mut rgba_img = RgbaImage::new(width, height);
        let green_pixel = Rgba([0, 255, 0, 255]);

        self.blobs.iter().for_each(|blob| {
            blob.iter()
                .for_each(|pt| rgba_img.put_pixel(pt.x, pt.y, green_pixel))
        });

        let yellow_pixel = Rgba([255, 255, 0, 255]);
        draw_hollow_rect_mut(&mut rgba_img, self.search_area, yellow_pixel);

        CanvasImage {
            data: rgba_img.into_raw(),
            width: width,
            height: height,
        }
    }

    #[wasm_bindgen]
    pub fn detect_blob(self) -> Self {
        let width = self.label_map.width();
        let height = self.label_map.height();

        let len = self.label_map.as_raw().iter().max().unwrap_or(&0) + 1;
        let mut blobs = vec![Vec::<Point<u32>>::new(); len as usize];

        for y in 0..height {
            for x in 0..width {
                let label = self.label_map.get_pixel(x, y)[0] as usize;
                if label > 0 {
                    blobs[label].push(Point::new(x, y));
                }
            }
        }

        BlobImage {
            label_map: self.label_map,
            blobs,
            search_area: Rect::at(0, 0).of_size(width, height),
            crop_area: Rect::at(0, 0).of_size(width, height),
        }
    }

    #[wasm_bindgen]
    pub fn extract_with_area(self, min_rate: f32, max_rate: f32) -> Self {
        let min_rate = min_rate.clamp(0.0, 100.0);
        let max_rate = max_rate.clamp(min_rate, 100.0);

        let width = self.label_map.width();
        let height = self.label_map.height();
        let area = (width * height) as f32;

        let min = area * min_rate / 100.0;
        let max = area * max_rate / 100.0;

        let blobs: Vec<Vec<Point<u32>>> = self
            .blobs
            .into_iter()
            .filter(|blob| {
                let area = blob.len() as f32;
                min < area && area < max
            })
            .collect();

        BlobImage {
            label_map: self.label_map,
            blobs,
            search_area: self.search_area,
            crop_area: self.crop_area,
        }
    }

    #[wasm_bindgen]
    pub fn extract_with_position(
        self,
        left_rate: f32,
        top_rate: f32,
        right_rate: f32,
        bottom_rate: f32,
    ) -> Self {
        let left_rate = left_rate.clamp(0.0, 100.0);
        let top_rate = top_rate.clamp(0.0, 100.0);
        let right_rate = right_rate.clamp(left_rate, 100.0);
        let bottom_rate = bottom_rate.clamp(top_rate, 100.0);

        let max_x = (self.label_map.width() - 1) as f32;
        let max_y = (self.label_map.height() - 1) as f32;

        let left = max_x * left_rate / 100.0;
        let top = max_y * top_rate / 100.0;
        let right = max_x * right_rate / 100.0;
        let bottom = max_y * bottom_rate / 100.0;

        let blobs: Vec<Vec<Point<u32>>> = self
            .blobs
            .into_iter()
            .filter(|blob| {
                blob.iter().all(|pt| {
                    let x = pt.x as f32;
                    let y = pt.y as f32;
                    left < x && x < right && top < y && y < bottom
                })
            })
            .collect();

        let width = (right - left + 1.0) as u32;
        let height = (bottom - top + 1.0) as u32;

        BlobImage {
            label_map: self.label_map,
            blobs,
            search_area: Rect::at(left as i32, top as i32).of_size(width, height),
            crop_area: self.crop_area,
        }
    }

    #[wasm_bindgen]
    pub fn crop_with_blob(self, width_rate: f32, height_rate: f32) -> Self {
        let width_rate = width_rate.clamp(0.0, 100.0);
        let height_rate = height_rate.clamp(0.0, 100.0);

        let img_width = self.label_map.width() as f32;
        let img_height = self.label_map.height() as f32;

        let mut count = 0.0;
        let mut center_x = 0.0;
        let mut center_y = 0.0;

        self.blobs.iter().for_each(|blob| {
            blob.iter().for_each(|pt| {
                count += 1.0;
                center_x += pt.x as f32;
                center_y += pt.y as f32;
            });
        });

        if 0.0 < count {
            center_x /= count;
            center_y /= count;
        } else {
            center_x = img_width / 2.0;
            center_y = img_height / 2.0;
        }

        let width = (img_width * width_rate / 100.0)
            .min(2.0 * center_x)
            .min(2.0 * (img_width - center_x));
        let height = (img_height * height_rate / 100.0)
            .min(2.0 * center_y)
            .min(2.0 * (img_height - center_y));

        let left = (center_x - width / 2.0) as i32;
        let top = (center_y - height / 2.0) as i32;

        let width = width.max(1.0) as u32;
        let height = height.max(1.0) as u32;

        BlobImage {
            label_map: self.label_map,
            blobs: self.blobs,
            search_area: self.search_area,
            crop_area: Rect::at(left, top).of_size(width, height),
        }
    }
}

#[wasm_bindgen]
impl CropRect {
    #[wasm_bindgen(getter)]
    pub fn left(&self) -> u32 {
        self.left
    }

    #[wasm_bindgen(getter)]
    pub fn top(&self) -> u32 {
        self.top
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[wasm_bindgen]
impl Center {
    #[wasm_bindgen(getter)]
    pub fn x(&self) -> f32 {
        self.x
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> f32 {
        self.y
    }
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! RustからWASM経由で挨拶してるよ！", name)
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
