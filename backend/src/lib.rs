use crate::cv::vision::Vision;
use serial2::SerialPort;
use std::path::{Path, PathBuf};

pub mod cv;
pub mod error;

pub(crate) type Result<T> = std::result::Result<T, crate::error::Error>;

pub fn list_devices() -> Result<Vec<PathBuf>> {
    Ok(SerialPort::available_ports()?)
}

#[derive(Default)]
pub struct Turret {
    pub port: Option<SerialPort>,
    pub vision: Vision,
}

impl Turret {
    pub fn connect(&mut self, port: impl AsRef<Path>) -> Result<()> {
        self.port = Some(SerialPort::open(port, 115200)?);

        Ok(())
    }
}
