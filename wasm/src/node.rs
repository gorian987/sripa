use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    u32,
};

use image::DynamicImage;

#[derive(Clone, Debug, Hash, serde::Deserialize, serde::Serialize, specta::Type)]
#[serde(tag = "type", content = "parameter")]
pub enum NodeType {
    // Read
    Read {
        width: u32,
        height: u32,
        filename: String,
        #[specta(type = f64)]
        last_modified: u64,
    },

    // Filter
    Grayscale {
        color: ColorType,
    },
    GaussianBlur {
        kernel: u32,
        sigma: u32,
    },
    Sobel {
        kernel: u32,
    },
    Binarization {
        threshold: u32,
    },

    // Convolution
    Center {
        rect: ImageRect,
    },
    Max {
        rect: ImageRect,
    },
    Min {
        rect: ImageRect,
    },
    Average {
        rect: ImageRect,
    },
    Mode {
        rect: ImageRect,
    },
}

impl NodeType {
    pub fn process(&self, inputs: Vec<NodeResult>) -> NodeResult {
        match self {
            Self::Read { .. } => Self::read_process(),
            Self::Grayscale { color } => Self::grayscale_process(inputs, *color),
            Self::GaussianBlur { kernel, sigma } => {
                Self::gaussian_blur_process(inputs, *kernel, *sigma)
            }
            Self::Sobel { kernel } => Self::sobel_process(inputs, *kernel),
            Self::Binarization { threshold } => Self::binarization_process(inputs, *threshold),
            Self::Center { rect } => Self::center_process(inputs, rect.clone()),
            Self::Max { rect } => Self::max_process(inputs, rect.clone()),
            Self::Min { rect } => Self::min_process(inputs, rect.clone()),
            Self::Average { rect } => Self::average_process(inputs, rect.clone()),
            Self::Mode { rect } => Self::mode_process(inputs, rect.clone()),
        }
    }

    pub fn is_protected(&self) -> bool {
        match self {
            Self::Read { .. } => true,
            _ => false,
        }
    }

    fn read_process() -> NodeResult {
        NodeResult::None
    }

    fn grayscale_process(inputs: Vec<NodeResult>, color: ColorType) -> NodeResult {
        NodeResult::None
    }

    fn gaussian_blur_process(inputs: Vec<NodeResult>, kernel: u32, sigma: u32) -> NodeResult {
        NodeResult::None
    }

    fn sobel_process(inputs: Vec<NodeResult>, kernel: u32) -> NodeResult {
        NodeResult::None
    }

    fn binarization_process(inputs: Vec<NodeResult>, threshold: u32) -> NodeResult {
        NodeResult::None
    }

    fn center_process(inputs: Vec<NodeResult>, rect: ImageRect) -> NodeResult {
        NodeResult::None
    }

    fn max_process(inputs: Vec<NodeResult>, rect: ImageRect) -> NodeResult {
        NodeResult::None
    }

    fn min_process(inputs: Vec<NodeResult>, rect: ImageRect) -> NodeResult {
        NodeResult::None
    }

    fn average_process(inputs: Vec<NodeResult>, rect: ImageRect) -> NodeResult {
        NodeResult::None
    }

    fn mode_process(inputs: Vec<NodeResult>, rect: ImageRect) -> NodeResult {
        NodeResult::None
    }
}

#[derive(Clone, Debug, Hash, serde::Deserialize, serde::Serialize, specta::Type)]
pub struct ReadParams {
    width: u32,
    height: u32,
    filename: String,
    last_modified: u64,
}

impl ReadParams {
    pub fn new(
        width: u32,
        height: u32,
        filename: String,
        last_modified: u64,
        image: Arc<DynamicImage>,
        storage: &mut HashMap<u64, NodeResult>,
    ) -> Self {
        let params = ReadParams {
            width,
            height,
            filename,
            last_modified,
        };

        let mut hasher = DefaultHasher::new();
        params.hash(&mut hasher);

        storage.insert(hasher.finish(), NodeResult::Image(image));
        params
    }
}

#[derive(Clone, Copy, Debug, Hash, serde::Deserialize, serde::Serialize, specta::Type)]
pub enum ColorType {
    Gray,
    Red,
    Green,
    Blue,
}

#[derive(Clone, Debug, Hash, serde::Deserialize, serde::Serialize, specta::Type)]
pub struct ImageRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Clone, Debug)]
pub enum NodeResult {
    Image(Arc<DynamicImage>),
    Point(f32, f32),
    Value(f32),
    None,
}

impl NodeResult {
    pub fn size_bytes(&self) -> usize {
        match self {
            Self::Image(image) => image.as_bytes().len(),
            Self::Point(..) => 8,
            Self::Value(..) => 4,
            Self::None => 1,
        }
    }
}
