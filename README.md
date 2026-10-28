# Async Image Cache

A asynchronous image caching and loading engine for Rust.

## Features

* **Dual-Source Support:** Parses and handles local file paths  and web URLs.
* **Memory Caching:** Backed by Moka to keep memory usage strictly within your specified limits (`max_cache_bytes`).
* **Background Fetching:** Spawns asynchronous download and decode tasks onto a user-provided Tokio runtime handle.
* **Deduplication & State Tracking:** Tracks loading states (`Ready`, `Failed`, or pending).


---
# Quick Start

## Set up the cache using the builder pattern and initialize it:
``` rust
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build the image cache with custom configuration
    let cache = ImageCacheBuilder::new()
        .max_cache_bytes(50 * 1024 * 1024) // 50 MB limit
        .http_timeout(Duration::from_secs(10))
        .allow_http_plain(false)
        .build();

    // Used in non async. Gets a cached image, if none it will spawn a fetch request and cache image. 
    let image = cache.get("https://example.com/image.png")?;
    match image {
        ImageState::Loading, => {},
        ImageState::Ready(image_data) => {},
        ImageState::Failed(error) => {},
    }

    //For Async
    let image = cache.fetch("https://example.com/image.png").await?

    Ok(())
}```
