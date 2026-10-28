use std::sync::Arc;

use image::imageops::FilterType;

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

impl ImageData {
    pub fn decode(&mut self, max_pixels: u64) -> Result<Arc<Rgba>, crate::Error> {
        if let Some(ref decoded) = self.decoded {
            return Ok(Arc::clone(decoded));
        }

        let cursor = std::io::Cursor::new(&self.bytes);
        let reader = image::ImageReader::new(cursor).with_guessed_format()?;

        let (width, height) = reader.into_dimensions()?;
        let total_pixels = width as u64 * height as u64;

        if total_pixels > max_pixels {
            return Err(crate::Error::TooLarge {
                received: total_pixels,
                max: max_pixels,
            });
        }

        let img = image::load_from_memory(&self.bytes)?.to_rgba8();
        let (w, h) = img.dimensions();

        let rgba = Arc::new(Rgba {
            width: w,
            height: h,
            pixels: img.into_raw(),
        });

        // Cache the decoded result
        self.decoded = Some(Arc::clone(&rgba));

        Ok(rgba)
    }
    pub fn decode_with_limit(&mut self, max_pixels: u64) -> Result<Arc<Rgba>, crate::Error> {
        if let Some(ref decoded) = self.decoded {
            return Ok(Arc::clone(decoded));
        }

        let cursor = std::io::Cursor::new(&self.bytes);
        let mut reader = image::ImageReader::new(cursor).with_guessed_format()?;
        let mut img = reader.decode()?;

        let width = img.width();
        let height = img.height();
        let total_pixels = width as u64 * height as u64;

        if total_pixels > max_pixels {
            let scale = (max_pixels as f64 / total_pixels as f64).sqrt();
            let new_width = (width as f64 * scale) as u32;
            let new_height = (height as f64 * scale) as u32;

            img = img.resize_exact(new_width, new_height, FilterType::Lanczos3);
        }

        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();

        let decoded_rgba = Arc::new(Rgba {
            width: w,
            height: h,
            pixels: rgba.into_raw(),
        });

        self.decoded = Some(Arc::clone(&decoded_rgba));
        Ok(decoded_rgba)
    }
}
