#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Image to large max: {max}, got: {received}")]
    TooLarge { max: u64, received: u64 },
}
