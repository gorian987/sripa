use kornia_image::{Image, ImageError, ImageSize, allocator::CpuAllocator};
use kornia_imgproc::color;
use std::sync::Arc;

macro_rules! kornia_image_method {
    ($img_type:expr, $img:ident => $action:expr) => {{
        match &*$img_type {
            ImageType::Gray8($img) => $action,
            ImageType::Gray32f($img) => $action,
            ImageType::Rgb8($img) => $action,
            ImageType::Rgb32f($img) => $action,
        }
    }};
}

#[derive(Clone)]
pub enum NodeResult {
    Image(Arc<ImageType>),
    Point(f32, f32),
    Value(f32),
    None,
}

#[derive(Clone)]
pub enum ImageType {
    Gray8(Image<u8, 1, CpuAllocator>),
    Gray32f(Image<f32, 1, CpuAllocator>),
    Rgb8(Image<u8, 3, CpuAllocator>),
    Rgb32f(Image<f32, 3, CpuAllocator>),
}

impl NodeResult {
    pub fn size_bytes(&self) -> usize {
        match self {
            Self::Image(img) => img.size_bytes(),
            Self::Point(..) => 8,
            Self::Value(..) => 4,
            Self::None => 1,
        }
    }
}

impl ImageType {
    pub fn as_gray8(&self) -> Option<&Image<u8, 1, CpuAllocator>> {
        match self {
            Self::Gray8(img) => Some(&img),
            _ => None,
        }
    }

    pub fn as_gray32f(&self) -> Option<&Image<f32, 1, CpuAllocator>> {
        match self {
            Self::Gray32f(img) => Some(&img),
            _ => None,
        }
    }

    pub fn as_rgb8(&self) -> Option<&Image<u8, 3, CpuAllocator>> {
        match self {
            Self::Rgb8(img) => Some(&img),
            _ => None,
        }
    }

    pub fn as_rgb32f(&self) -> Option<&Image<f32, 3, CpuAllocator>> {
        match self {
            Self::Rgb32f(img) => Some(&img),
            _ => None,
        }
    }

    pub fn to_gray8(&self) -> Result<Image<u8, 1, CpuAllocator>, ImageError> {
        match self {
            Self::Gray8(img) => Ok(img.clone()),
            Self::Rgb8(img) => {
                let src = img.cast::<f32>()?;

                let size = ImageSize {
                    width: src.width(),
                    height: src.height(),
                };
                let alloc = CpuAllocator::default();
                let mut dst = Image::<f32, 1, _>::from_size_val(size, 0.0, alloc)?;

                color::gray_from_rgb(&src, &mut dst)?;
                dst.cast::<u8>()
            }
            Self::Gray32f(img) => img.cast::<u8>(),
            Self::Rgb32f(img) => {
                let src = img;
                let size = ImageSize {
                    width: src.width(),
                    height: src.height(),
                };
                let alloc = CpuAllocator::default();
                let mut dst = Image::<f32, 1, _>::from_size_val(size, 0.0, alloc)?;

                color::gray_from_rgb(&src, &mut dst)?;
                dst.cast::<u8>()
            }
        }
    }

    pub fn to_gray32f(&self) -> Result<Image<f32, 1, CpuAllocator>, ImageError> {
        match self {
            Self::Gray8(img) => img.cast::<f32>(),
            Self::Rgb8(img) => {
                let src = img.cast::<f32>()?;

                let size = ImageSize {
                    width: src.width(),
                    height: src.height(),
                };
                let alloc = CpuAllocator::default();
                let mut dst = Image::<f32, 1, _>::from_size_val(size, 0.0, alloc)?;

                color::gray_from_rgb(&src, &mut dst)?;
                Ok(dst)
            }
            Self::Gray32f(img) => Ok(img.clone()),
            Self::Rgb32f(img) => {
                let src = img;

                let size = ImageSize {
                    width: src.width(),
                    height: src.height(),
                };
                let alloc = CpuAllocator::default();
                let mut dst = Image::<f32, 1, _>::from_size_val(size, 0.0, alloc)?;

                color::gray_from_rgb(&src, &mut dst)?;
                Ok(dst)
            }
        }
    }

    pub fn to_rgb8(&self) -> Result<Image<u8, 3, CpuAllocator>, ImageError> {
        match self {
            Self::Gray8(img) => {
                let src = img.cast::<f32>()?;

                let size = ImageSize {
                    width: src.width(),
                    height: src.height(),
                };
                let alloc = CpuAllocator::default();
                let mut dst = Image::<f32, 3, _>::from_size_val(size, 0.0, alloc)?;

                color::rgb_from_gray(&src, &mut dst)?;
                dst.cast::<u8>()
            }
            Self::Rgb8(img) => Ok(img.clone()),
            Self::Gray32f(img) => {
                let src = img;

                let size = ImageSize {
                    width: src.width(),
                    height: src.height(),
                };
                let alloc = CpuAllocator::default();
                let mut dst = Image::<f32, 3, _>::from_size_val(size, 0.0, alloc)?;

                color::rgb_from_gray(&src, &mut dst)?;
                dst.cast::<u8>()
            }
            Self::Rgb32f(img) => img.cast::<u8>(),
        }
    }

    pub fn to_rgb32f(&self) -> Result<Image<f32, 3, CpuAllocator>, ImageError> {
        match self {
            Self::Gray8(img) => {
                let src = img.cast::<f32>()?;

                let size = ImageSize {
                    width: src.width(),
                    height: src.height(),
                };
                let alloc = CpuAllocator::default();
                let mut dst = Image::<f32, 3, _>::from_size_val(size, 0.0, alloc)?;

                color::rgb_from_gray(&src, &mut dst)?;
                Ok(dst)
            }
            Self::Rgb8(img) => img.cast::<f32>(),
            Self::Gray32f(img) => {
                let src = img;

                let size = ImageSize {
                    width: src.width(),
                    height: src.height(),
                };
                let alloc = CpuAllocator::default();
                let mut dst = Image::<f32, 3, _>::from_size_val(size, 0.0, alloc)?;

                color::rgb_from_gray(&src, &mut dst)?;
                Ok(dst)
            }
            Self::Rgb32f(img) => Ok(img.clone()),
        }
    }

    pub fn size_bytes(&self) -> usize {
        match self {
            Self::Gray8(img) => img.numel(),
            Self::Gray32f(img) => img.numel() * 4,
            Self::Rgb8(img) => img.numel(),
            Self::Rgb32f(img) => img.numel() * 4,
        }
    }

    pub fn into_result(self) -> NodeResult {
        NodeResult::Image(Arc::new(self))
    }

    pub fn height(&self) -> usize {
        kornia_image_method!(self, img => img.height())
    }

    pub fn width(&self) -> usize {
        kornia_image_method!(self, img => img.width())
    }
}
