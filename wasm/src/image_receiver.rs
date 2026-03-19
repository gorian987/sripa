use crate::node_value::ImageType;
use kornia_image::{Image, ImageError, ImageSize, allocator::CpuAllocator};
use std::sync::Arc;

pub struct ImageReceiver {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
}

impl ImageReceiver {
    pub fn new(width: u32, height: u32) -> Self {
        ImageReceiver {
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

    pub fn get(&mut self, width: u32, height: u32) -> Result<Arc<ImageType>, ImageError> {
        if width == 0 || height == 0 {
            return Err(ImageError::InvalidImageSize(
                width as usize,
                height as usize,
                self.width as usize,
                self.height as usize,
            ));
        }

        let width = width.min(self.width) as usize;
        let height = height.min(self.height) as usize;
        let len = 4 * width * height;

        let size = ImageSize { width, height };
        let data = self.buffer[..len]
            .chunks_exact(4)
            .flat_map(|v| [v[0] as f32, v[1] as f32, v[2] as f32])
            .collect();
        let alloc = CpuAllocator::default();
        let img = Image::<f32, 3, _>::new(size, data, alloc)?;

        Ok(Arc::new(ImageType::Rgb(img)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_invalid_size() -> Result<(), ImageError> {
        let width = 5;
        let height = 5;

        let mut recorder = ImageReceiver::new(width, height);
        let zero_size = recorder.get(0, 0);
        let over_width = recorder.get(2 * width, height)?;
        let over_height = recorder.get(width, 2 * height)?;

        let width = width as usize;
        let height = height as usize;

        assert!(zero_size.is_err());

        assert_eq!(over_width.width(), width);
        assert_eq!(over_width.height(), height);

        assert_eq!(over_height.width(), width);
        assert_eq!(over_height.height(), height);

        Ok(())
    }
}
