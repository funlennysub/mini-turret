use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    OpenCV(#[from] opencv::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}
