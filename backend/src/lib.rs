use crate::cv::vision::Vision;
use std::path::{Path, PathBuf};

pub mod cv;
pub mod error;

pub(crate) type Result<T> = std::result::Result<T, crate::error::Error>;

#[derive(Default)]
pub struct Turret {
    pub vision: Vision,
}

impl Turret {
}
