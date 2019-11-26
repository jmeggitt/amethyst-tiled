use image::ImageError;
use std::io::Error as IOError;
use tiled::TiledError;

pub enum LoadError {
    Tiled(TiledError),
    IO(IOError),
    // TODO: Allow for non rgba8 images
    ImageType,
    ImageError(ImageError),
}

impl From<TiledError> for LoadError {
    fn from(err: TiledError) -> Self {
        Self::Tiled(err)
    }
}

impl From<IOError> for LoadError {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

impl From<ImageError> for LoadError {
    fn from(err: ImageError) -> Self {
        Self::ImageError(err)
    }
}
