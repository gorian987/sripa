use std::{
    collections::VecDeque,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use image::{DynamicImage, RgbaImage};

pub struct ImageRecorder {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
    storage: VecDeque<TaggedImage>,
}

impl ImageRecorder {
    pub fn new(width: u32, height: u32) -> Self {
        ImageRecorder {
            buffer: vec![0; (4 * width * height) as usize],
            width,
            height,
            storage: VecDeque::new(),
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

    pub fn take_records(&mut self) -> VecDeque<TaggedImage> {
        std::mem::take(&mut self.storage)
    }

    pub fn record(&mut self, width: u32, height: u32, filename: String, last_modified: u64) {
        let width = width.min(self.width);
        let height = height.min(self.height);
        let len = (4 * width * height) as usize;

        let buf = self.buffer[..len].to_vec();
        let image =
            RgbaImage::from_raw(width, height, buf).unwrap_or(RgbaImage::new(width, height));

        let mut hasher = DefaultHasher::new();
        width.hash(&mut hasher);
        height.hash(&mut hasher);
        filename.hash(&mut hasher);
        last_modified.hash(&mut hasher);

        self.storage.push_back(TaggedImage {
            id: hasher.finish(),
            image: Arc::new(DynamicImage::ImageRgba8(image)),
        });
    }
}

#[derive(Clone, Debug)]
pub struct TaggedImage {
    pub id: u64,
    pub image: Arc<DynamicImage>,
}
