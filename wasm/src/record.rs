use std::sync::Arc;

use image::{DynamicImage, RgbaImage};

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

    pub fn record(&mut self, width: u32, height: u32) -> Arc<DynamicImage> {
        let width = width.min(self.width);
        let height = height.min(self.height);
        let len = (4 * width * height) as usize;

        let buf = self.buffer[..len].to_vec();
        let image =
            RgbaImage::from_raw(width, height, buf).unwrap_or(RgbaImage::new(width, height));

        Arc::new(DynamicImage::ImageRgba8(image))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_abnormal_size() -> Result<(), ()> {
        let width = 5;
        let height = 5;

        let mut recorder = ImageRecorder::new(width, height);
        let zero_size = recorder.record(0, 0);
        let over_width = recorder.record(2 * width, height);
        let over_height = recorder.record(width, 2 * height);

        assert_eq!(zero_size.width(), 0);
        assert_eq!(zero_size.height(), 0);

        assert_eq!(over_width.width(), width);
        assert_eq!(over_width.height(), height);

        assert_eq!(over_height.width(), width);
        assert_eq!(over_height.height(), height);

        Ok(())
    }
}
