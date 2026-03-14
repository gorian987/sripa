use crate::node_result::ImageType;
use kornia_image::{Image, ImageError, ImageSize, allocator::CpuAllocator};
use kornia_imgproc::color;
use std::sync::Arc;

pub struct ImageRecorder {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
}

impl ImageRecorder {
    pub fn new(width: u32, height: u32) -> Self {
        ImageRecorder {
            buffer: vec![0; (4 * width * height) as usize],
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn record(&mut self, width: u32, height: u32) -> Result<Arc<ImageType>, ImageError> {
        let width = width.clamp(1, self.width) as usize;
        let height = height.clamp(1, self.height) as usize;
        let len = 4 * width * height;

        let size = ImageSize { width, height };
        let data = self.buffer[..len].to_vec();
        let alloc = CpuAllocator::default();
        let rgba = Image::<u8, 4, _>::new(size, data, alloc.clone())?;

        let mut rgb = Image::<u8, 3, _>::from_size_val(size, 0, alloc)?;
        color::rgb_from_rgba(&rgba, &mut rgb, None)?;

        Ok(Arc::new(ImageType::Rgb8(rgb)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_abnormal_size() -> Result<(), ImageError> {
        let width = 5;
        let height = 5;

        let mut recorder = ImageRecorder::new(width, height);
        let zero_size = recorder.record(0, 0)?;
        let over_width = recorder.record(2 * width, height)?;
        let over_height = recorder.record(width, 2 * height)?;

        let width = width as usize;
        let height = height as usize;

        assert_eq!(zero_size.width(), 1);
        assert_eq!(zero_size.height(), 1);

        assert_eq!(over_width.width(), width);
        assert_eq!(over_width.height(), height);

        assert_eq!(over_height.width(), width);
        assert_eq!(over_height.height(), height);

        Ok(())
    }
}
