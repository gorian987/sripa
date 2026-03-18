use crate::node_result::{ImageType, NodeResult};
use kornia_image::{Image, ImageError, ImageSize};
use kornia_imgproc::{filter, threshold};
use ordered_float::OrderedFloat;
use std::{hash::Hash, u32};

macro_rules! img_type_map {
    ($img_type:expr, $src:ident => $action:expr) => {{
        match &*$img_type {
            ImageType::Gray($src) => $action.map(|dst| ImageType::Gray(dst).into()),
            ImageType::Rgb($src) => $action.map(|dst| ImageType::Rgb(dst).into()),
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
        let Some(NodeResult::Image(img_type)) = inputs.into_iter().next() else {
            return Err(NodeError::InvalidInput(1));
        };

        match &*img_type {
            ImageType::Gray(_) => Ok(NodeResult::Image(img_type)),
            ImageType::Rgb(rgb) => match color {
                ColorType::Red => rgb.channel(0),
                ColorType::Green => rgb.channel(1),
                ColorType::Blue => rgb.channel(2),
            }
            .map(|gray| ImageType::Gray(gray).into())
            .map_err(|err| err.into()),
        }
    }

    fn gaussian_blur_process(
        inputs: Vec<NodeResult>,
        kernel_x: u32,
        kernel_y: u32,
        sigma_x: f32,
        sigma_y: f32,
    ) -> Result<NodeResult, NodeError> {
        let Some(NodeResult::Image(img_type)) = inputs.into_iter().next() else {
            return Err(NodeError::InvalidInput(1));
        };

        img_type_map!(img_type, src => {
            let kernel_x = kernel_x.max(0) | 1;
            let kernel_y = kernel_y.max(0) | 1;
            let sigma_x = sigma_x.max(0.0);
            let sigma_y = sigma_y.max(0.0);

            let size = ImageSize {
                width: src.width(),
                height: src.height(),
            };

            let mut dst = Image::<_, _, _>::from_size_val(size, 0.0, Default::default())?;
            filter::gaussian_blur(
                &src,
                &mut dst,
                (kernel_x as usize, kernel_y as usize),
                (sigma_x, sigma_y),
            )?;

            Ok(dst)
        })
    }

    fn sobel_process(inputs: Vec<NodeResult>, kernel: u32) -> Result<NodeResult, NodeError> {
        let Some(NodeResult::Image(img_type)) = inputs.into_iter().next() else {
            return Err(NodeError::InvalidInput(1));
        };

        img_type_map!(img_type, src => {
            let kernel = kernel.clamp(3, 5);

            let size = ImageSize {
                width: src.width(),
                height: src.height(),
            };

            let mut dst = Image::<_, _, _>::from_size_val(size, 0.0, Default::default())?;
            filter::sobel(&src, &mut dst, kernel as usize)?;

            Ok(dst)
        })
    }

    fn binarization_process(
        inputs: Vec<NodeResult>,
        threshold: f32,
        max_value: f32,
        inverse: bool,
    ) -> Result<NodeResult, NodeError> {
        let Some(NodeResult::Image(img_type)) = inputs.into_iter().next() else {
            return Err(NodeError::InvalidInput(1));
        };

        img_type_map!(img_type, src => {
            let threshold = threshold.clamp(0.0, 255.0);
            let max_value = max_value.clamp(0.0, 255.0);

            let size = ImageSize {
                width: src.width(),
                height: src.height(),
            };
            let mut dst = Image::<_, _, _>::from_size_val(size, 0.0, Default::default())?;

            match inverse {
                false => threshold::threshold_binary(&src, &mut dst, threshold, max_value),
                true => threshold::threshold_binary_inverse(&src, &mut dst, threshold, max_value),
            }?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use kornia_image::{Image, ImageSize, allocator::CpuAllocator};
    use std::sync::Arc;

    fn create_rgb_image() -> Arc<ImageType> {
        let size = ImageSize {
            width: 3,
            height: 3,
        };
        let data = vec![
            1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, // 1
            0.5, 0.5, 0.5, 0.2, 0.2, 0.2, 0.8, 0.8, 0.8, // 2
            0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.5, 0.5, 0.5, // 3
        ];
        Arc::new(ImageType::Rgb(
            Image::new(size, data, CpuAllocator::default()).unwrap(),
        ))
    }

    #[test]
    fn color_split_red() -> Result<(), NodeError> {
        let img = create_rgb_image();
        let inputs = vec![NodeResult::Image(img)];

        let node = NodeType::ColorSplit {
            color: ColorType::Red,
        };
        let result = node.process(inputs)?;

        if let NodeResult::Image(res_img) = result {
            match &*res_img {
                ImageType::Gray(gray) => {
                    let red = gray.get_pixel(0, 0, 0)?;
                    assert_eq!(*red, 1.0);
                }
                _ => return Err(NodeError::InvalidImageType()),
            }
        }
        Ok(())
    }

    #[test]
    fn gaussian_blur() {
        let img = create_rgb_image();
        let inputs = vec![NodeResult::Image(img)];

        let node = NodeType::GaussianBlur {
            kernel_x: 0,
            kernel_y: 2,
            sigma_x: OrderedFloat(0.0),
            sigma_y: OrderedFloat(-1.0),
        };

        let result = node.process(inputs);
        assert!(result.is_ok());
    }

    #[test]
    fn sobel() {
        let img = create_rgb_image();
        let inputs = vec![NodeResult::Image(img)];

        let zero = NodeType::Sobel { kernel: 0 };
        let over = NodeType::Sobel { kernel: 10 };

        let res_zero = zero.process(inputs.clone());
        let res_over = over.process(inputs);

        assert!(res_zero.is_ok());
        assert!(res_over.is_ok());
    }

    #[test]
    fn binarization() -> Result<(), NodeError> {
        let img = create_rgb_image();
        let inputs = vec![NodeResult::Image(img)];

        let node = NodeType::Binarization {
            threshold: OrderedFloat(0.5),
            max_value: OrderedFloat(1.0),
            inverse: false,
        };

        let result = node.process(inputs)?;
        if let NodeResult::Image(res_img) = result {
            match &*res_img {
                ImageType::Rgb(rgb) => {
                    let not_binarized = rgb
                        .as_slice()
                        .iter()
                        .filter(|v| **v != 0.0 && **v != 1.0)
                        .collect::<Vec<_>>();
                    assert!(not_binarized.is_empty());
                }
                _ => return Err(NodeError::InvalidImageType()),
            }
        };

        Ok(())
    }

    #[test]
    fn read_process_fail() {
        let node = NodeType::Read {
            width: 100,
            height: 100,
            filename: "test.png".to_string(),
            last_modified: 12345,
        };
        let result = node.process(vec![]);
        assert!(matches!(result, Err(NodeError::ReadProcess())));
    }

    #[test]
    fn invalid_input() {
        let node = NodeType::Sobel { kernel: 3 };
        let result = node.process(vec![]);
        assert!(matches!(result, Err(NodeError::InvalidInput(1))));
    }
}
