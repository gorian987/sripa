use kornia_image::{Image, ImageError, ImageSize, allocator::CpuAllocator};
use kornia_imgproc::color;
use std::sync::Arc;

macro_rules! kornia_image_method {
    ($img_type:expr, $img:ident => $action:expr) => {{
        match &*$img_type {
            ImageType::Gray($img) => $action,
            ImageType::Rgb($img) => $action,
        }
    }};
}

#[derive(Clone)]
pub enum NodeValue {
    Image(Arc<ImageType>),
    Point(f32, f32),
    Value(f32),
    None,
}

#[derive(Clone)]
pub enum ImageType {
    Gray(Image<f32, 1, CpuAllocator>),
    Rgb(Image<f32, 3, CpuAllocator>),
}

impl From<ImageType> for NodeValue {
    fn from(img: ImageType) -> Self {
        Self::Image(Arc::new(img))
    }
}

impl From<Arc<ImageType>> for NodeValue {
    fn from(img: Arc<ImageType>) -> Self {
        Self::Image(img)
    }
}

impl NodeValue {
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
    pub fn height(&self) -> usize {
        kornia_image_method!(self, img => img.height())
    }

    pub fn size_bytes(&self) -> usize {
        let numel = kornia_image_method!(self, img => {
            img.numel()
        });

        numel * 4
    }

    pub fn width(&self) -> usize {
        kornia_image_method!(self, img => img.width())
    }

    pub fn as_gray(&self) -> Option<&Image<f32, 1, CpuAllocator>> {
        match self {
            Self::Gray(img) => Some(&img),
            _ => None,
        }
    }

    pub fn as_rgb(&self) -> Option<&Image<f32, 3, CpuAllocator>> {
        match self {
            Self::Rgb(img) => Some(&img),
            _ => None,
        }
    }

    pub fn to_gray(&self) -> Result<Image<f32, 1, CpuAllocator>, ImageError> {
        match self {
            Self::Gray(img) => Ok(img.clone()),
            Self::Rgb(img) => {
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

    pub fn to_rgb(&self) -> Result<Image<f32, 3, CpuAllocator>, ImageError> {
        match self {
            Self::Gray(img) => {
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
            Self::Rgb(img) => Ok(img.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn width() {
        let size = ImageSize {
            width: 5,
            height: 5,
        };
        let alloc = CpuAllocator::default();

        let (gray, rgb) = define_all_type(size, alloc);

        assert_eq!(gray.width(), 5);
        assert_eq!(rgb.width(), 5);
    }

    #[test]
    fn height() {
        let size = ImageSize {
            width: 5,
            height: 5,
        };
        let alloc = CpuAllocator::default();

        let (gray, rgb) = define_all_type(size, alloc);

        assert_eq!(gray.height(), 5);
        assert_eq!(rgb.height(), 5);
    }

    #[test]
    fn size_bytes() {
        let size = ImageSize {
            width: 5,
            height: 5,
        };
        let alloc = CpuAllocator::default();

        let (gray, rgb) = define_all_type(size, alloc);

        assert_eq!(gray.size_bytes(), 100);
        assert_eq!(rgb.size_bytes(), 300);
    }

    #[test]
    fn as_() {
        let size = ImageSize {
            width: 5,
            height: 5,
        };
        let alloc = CpuAllocator::default();

        let (gray, rgb) = define_all_type(size, alloc);

        assert!(gray.as_gray().is_some());
        assert!(gray.as_rgb().is_none());

        assert!(rgb.as_gray().is_none());
        assert!(rgb.as_rgb().is_some());
    }

    #[test]
    fn to_gray() {
        let size = ImageSize {
            width: 5,
            height: 5,
        };
        let alloc = CpuAllocator::default();

        let (gray, rgb) = define_all_type(size, alloc);

        assert!(ImageType::Gray(gray.to_gray().unwrap()).as_gray().is_some());
        assert!(ImageType::Gray(rgb.to_gray().unwrap()).as_gray().is_some());
    }

    #[test]
    fn to_rgb32f() {
        let size = ImageSize {
            width: 5,
            height: 5,
        };
        let alloc = CpuAllocator::default();

        let (gray, rgb) = define_all_type(size, alloc);

        assert!(ImageType::Rgb(gray.to_rgb().unwrap()).as_rgb().is_some());
        assert!(ImageType::Rgb(rgb.to_rgb().unwrap()).as_rgb().is_some());
    }

    fn define_all_type(size: ImageSize, alloc: CpuAllocator) -> (ImageType, ImageType) {
        let gray =
            ImageType::Gray(Image::<f32, 1, _>::from_size_val(size, 0.0, alloc.clone()).unwrap());
        let rgb =
            ImageType::Rgb(Image::<f32, 3, _>::from_size_val(size, 0.0, alloc.clone()).unwrap());

        (gray, rgb)
    }
}
