use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ImageData {
    pub bytes: bytes::Bytes,
    pub content_type: Option<String>,
    pub decoded: Option<Arc<Rgba>>,
}

#[derive(Debug)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum ImageState {
    Loading,
    Ready(Arc<ImageData>),
    Failed(Arc<crate::Error>),
}
