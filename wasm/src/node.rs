use crate::node_result::{ImageType, NodeResult};
use kornia_image::{Image, ImageError, ImageSize, allocator::CpuAllocator};
use kornia_imgproc::{filter, threshold};
use ordered_float::OrderedFloat;
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    u32,
};

macro_rules! map {
    ($img_type:expr, $src:ident => $action:expr) => {{
        match &*$img_type {
            ImageType::Gray8($src) => $action.map(|dst| ImageType::Gray8(dst).into_result()),
            ImageType::Gray32f($src) => $action.map(|dst| ImageType::Gray32f(dst).into_result()),
            ImageType::Rgb8($src) => $action.map(|dst| ImageType::Rgb8(dst).into_result()),
            ImageType::Rgb32f($src) => $action.map(|dst| ImageType::Rgb32f(dst).into_result()),
        }
    }};
}

// macro_rules! map_f32 {
//     ($img_type:expr, $src:ident => $action:expr) => {{
//         match &*$img_type {
//             ImageType::Gray8(_) => Ok(NodeResult::Image($img_type)),
//             ImageType::Gray32f($src) => $action.map(|dst| ImageType::Gray32f(dst).into_result()),
//             ImageType::Rgb8(_) => Ok(NodeResult::Image($img_type)),
//             ImageType::Rgb32f($src) => $action.map(|dst| ImageType::Rgb32f(dst).into_result()),
//         }
//     }};
// }

macro_rules! map_rgb_to_gray {
    ($img_type:expr, $src:ident => $action:expr) => {{
        match &*$img_type {
            ImageType::Gray8(_) => Ok(NodeResult::Image($img_type)),
            ImageType::Gray32f(_) => Ok(NodeResult::Image($img_type)),
            ImageType::Rgb8($src) => $action.map(|dst| ImageType::Gray8(dst).into_result()),
            ImageType::Rgb32f($src) => $action.map(|dst| ImageType::Gray32f(dst).into_result()),
        }
    }};
}

macro_rules! map_to_f32 {
    ($img_type:expr, $src:ident => $action:expr) => {{
        match &*$img_type {
            ImageType::Gray8(temp) => {
                let $src = temp.cast::<f32>()?;
                $action.map(|dst| ImageType::Gray32f(dst).into_result())
            }
            ImageType::Gray32f($src) => $action.map(|dst| ImageType::Gray32f(dst).into_result()),
            ImageType::Rgb8(temp) => {
                let $src = temp.cast::<f32>()?;
                $action.map(|dst| ImageType::Rgb32f(dst).into_result())
            }
            ImageType::Rgb32f($src) => $action.map(|dst| ImageType::Rgb32f(dst).into_result()),
        }
    }};
}

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
    ColorSplit {
        color: ColorType,
    },
    GaussianBlur {
        kernel_x: u32,
        kernel_y: u32,
        #[specta(type = f32)]
        sigma_x: OrderedFloat<f32>,
        #[specta(type = f32)]
        sigma_y: OrderedFloat<f32>,
    },
    Sobel {
        kernel: u32,
    },
    Binarization {
        #[specta(type = f32)]
        threshold: OrderedFloat<f32>,
        #[specta(type = f32)]
        max_value: OrderedFloat<f32>,
        inverse: bool,
    },
    // // Convolution
    // Center {
    //     rect: ImageRect,
    // },
    // Max {
    //     rect: ImageRect,
    // },
    // Min {
    //     rect: ImageRect,
    // },
    // Average {
    //     rect: ImageRect,
    // },
    // Mode {
    //     rect: ImageRect,
    // },
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
        image: Arc<ImageType>,
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

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("Image error: {0}")]
    ImageError(#[from] ImageError),
    #[error("No. {0} input is invalid!")]
    InvalidInput(usize),
    #[error("Color type is invalid!")]
    InvalidColorType(),
    #[error("Image type is invalid!")]
    InvalidImageType(),
    #[error("Read process should not be called!")]
    ReadProcess(),
}

impl NodeType {
    pub fn process(&self, inputs: Vec<NodeResult>) -> Result<NodeResult, NodeError> {
        match self {
            Self::Read { .. } => Self::read_process(),
            Self::ColorSplit { color } => Self::color_split_process(inputs, *color),
            Self::GaussianBlur {
                kernel_x,
                kernel_y,
                sigma_x,
                sigma_y,
            } => Self::gaussian_blur_process(
                inputs,
                *kernel_x,
                *kernel_y,
                *sigma_x.as_ref(),
                *sigma_y.as_ref(),
            ),
            Self::Sobel { kernel } => Self::sobel_process(inputs, *kernel),
            Self::Binarization {
                threshold,
                max_value,
                inverse,
            } => Self::binarization_process(
                inputs,
                *threshold.as_ref(),
                *max_value.as_ref(),
                *inverse,
            ),
            // Self::Center { rect } => Self::center_process(inputs, rect.clone()),
            // Self::Max { rect } => Self::max_process(inputs, rect.clone()),
            // Self::Min { rect } => Self::min_process(inputs, rect.clone()),
            // Self::Average { rect } => Self::average_process(inputs, rect.clone()),
            // Self::Mode { rect } => Self::mode_process(inputs, rect.clone()),
        }
    }

    pub fn is_protected(&self) -> bool {
        match self {
            Self::Read { .. } => true,
            _ => false,
        }
    }

    fn read_process() -> Result<NodeResult, NodeError> {
        Err(NodeError::ReadProcess())
    }

    fn color_split_process(
        inputs: Vec<NodeResult>,
        color: ColorType,
    ) -> Result<NodeResult, NodeError> {
        let Some(NodeResult::Image(img)) = inputs.into_iter().next() else {
            return Err(NodeError::InvalidInput(1));
        };

        map_rgb_to_gray!(img, src => {
            match color {
                ColorType::Red => src.channel(0),
                ColorType::Green => src.channel(1),
                ColorType::Blue => src.channel(2),
            }.map_err(|err| NodeError::ImageError(err))
        })
    }

    fn gaussian_blur_process(
        inputs: Vec<NodeResult>,
        kernel_x: u32,
        kernel_y: u32,
        sigma_x: f32,
        sigma_y: f32,
    ) -> Result<NodeResult, NodeError> {
        let Some(NodeResult::Image(img)) = inputs.into_iter().next() else {
            return Err(NodeError::InvalidInput(1));
        };

        map_to_f32!(img, src => {
            let kernel_x = kernel_x.max(1);
            let kernel_y = kernel_y.max(1);
            let sigma_x = sigma_x.max(0.0);
            let sigma_y = sigma_y.max(0.0);

            let size = ImageSize {
                width: src.width() as usize,
                height: src.height() as usize,
            };
            let alloc = CpuAllocator::default();
            let mut dst = Image::<f32, _, _>::from_size_val(size, 0.0, alloc)?;

            filter::gaussian_blur(
                &src,
                &mut dst,
                (kernel_x as usize, kernel_y as usize),
                (sigma_x, sigma_y),
            );

            Ok(dst)
        })
    }

    fn sobel_process(inputs: Vec<NodeResult>, kernel: u32) -> Result<NodeResult, NodeError> {
        let Some(NodeResult::Image(img)) = inputs.into_iter().next() else {
            return Err(NodeError::InvalidInput(1));
        };

        map_to_f32!(img, src => {
            let kernel = kernel.clamp(3, 5);

            let size = ImageSize {
                width: src.width() as usize,
                height: src.height() as usize,
            };
            let alloc = CpuAllocator::default();
            let mut dst = Image::<f32, _, _>::from_size_val(size, 0.0, alloc)?;

            filter::sobel(&src, &mut dst, kernel as usize);

            Ok(dst)
        })
    }

    fn binarization_process(
        inputs: Vec<NodeResult>,
        threshold: f32,
        max_value: f32,
        inverse: bool,
    ) -> Result<NodeResult, NodeError> {
        let Some(NodeResult::Image(img)) = inputs.into_iter().next() else {
            return Err(NodeError::InvalidInput(1));
        };

        map!(img, src => {
            let threshold = threshold.clamp(0.0, 255.0) as _;
            let max_value = max_value.clamp(0.0, 255.0) as _;

            let size = ImageSize {
                width: src.width() as usize,
                height: src.height() as usize,
            };
            let alloc = CpuAllocator::default();
            let mut dst = Image::<_, _, _>::from_size_val(size, Default::default(), alloc)?;

            match inverse {
                false => threshold::threshold_binary(&src, &mut dst, threshold, max_value),
                true => threshold::threshold_binary_inverse(&src, &mut dst, threshold, max_value),
            };

            Ok(dst)
        })
    }

    // fn center_process(inputs: Vec<NodeResult>, rect: ImageRect) -> Result<NodeResult, NodeError> {
    //     NodeResult::None
    // }

    // fn max_process(inputs: Vec<NodeResult>, rect: ImageRect) -> Result<NodeResult, NodeError> {
    //     NodeResult::None
    // }

    // fn min_process(inputs: Vec<NodeResult>, rect: ImageRect) -> Result<NodeResult, NodeError> {
    //     NodeResult::None
    // }

    // fn average_process(inputs: Vec<NodeResult>, rect: ImageRect) -> Result<NodeResult, NodeError> {
    //     NodeResult::None
    // }

    // fn mode_process(inputs: Vec<NodeResult>, rect: ImageRect) -> Result<NodeResult, NodeError> {
    //     NodeResult::None
    // }
}
