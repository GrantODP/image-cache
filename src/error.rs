#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Image to large max: {max}, got: {received}")]
    TooLarge { max: u64, received: u64 },

    #[cfg(feature = "decode")]
    #[error("{0}")]
    ImageError(#[from] image::ImageError),
}
