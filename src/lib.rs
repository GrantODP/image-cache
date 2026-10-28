mod engine;
mod error;
mod image;
mod source;

use std::{path::PathBuf, sync::Arc, time::Duration};

pub use error::Error;
use futures_util::FutureExt;
use tokio::runtime::Handle;

use crate::{
    engine::{Config, Engine, normalize},
    image::{ImageData, ImageState},
};
pub type Result<T> = std::result::Result<T, Arc<Error>>;

#[derive(Debug)]
pub struct ImageCache {
    engine: Arc<Engine>,
}

impl<'a> ImageCache {
    pub async fn fetch<S: AsRef<str>>(&self, url: S) -> Result<Arc<ImageData>> {
        self.engine.fetch(url).await
    }

    async fn download<S: AsRef<str>>(
        &self,
        url: S,
    ) -> std::result::Result<Arc<ImageData>, crate::Error> {
        self.engine.download(url).await
    }

    pub fn get(&self, url: &str) -> ImageState {
        let key = normalize(url);

        if let Some(hit) = self.engine.cache.get(&key).now_or_never().flatten() {
            return ImageState::Ready(hit);
        }
        if let Some(s) = self.engine.image_state.lock().get(&key) {
            return s.clone();
        }

        //Start a fetch, return Loading.

        self.engine
            .image_state
            .lock()
            .insert(key.clone(), ImageState::Loading);

        let engine = self.engine.clone();

        self.engine.handle.spawn(async move {
            let result = engine.fetch(&key).await;

            let state = match result {
                Ok(d) => ImageState::Ready(d),
                Err(e) => ImageState::Failed(e),
            };

            engine.image_state.lock().insert(key, state);

            if let Some(cb) = engine.on_update.as_ref() {
                cb();
            }
        });
        ImageState::Loading
    }
}

#[derive(Debug, Clone)]
pub struct ImageCacheBuilder {
    config: Config,
    handle: Option<Handle>,
}

impl ImageCacheBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            handle: None,
        }
    }

    pub fn max_cache_bytes(mut self, val: u64) -> Self {
        self.config.max_cache_bytes = val;
        self
    }

    pub fn max_item_bytes(mut self, val: u64) -> Self {
        self.config.max_item_bytes = val;
        self
    }

    pub fn max_pixels(mut self, val: u64) -> Self {
        self.config.max_pixels = val;
        self
    }

    pub fn http_timeout(mut self, val: Duration) -> Self {
        self.config.http_timeout = val;
        self
    }

    pub fn max_redirects(mut self, val: usize) -> Self {
        self.config.max_redirects = val;
        self
    }

    pub fn web_ttl(mut self, val: Option<Duration>) -> Self {
        self.config.web_ttl = val;
        self
    }

    pub fn negative_ttl(mut self, val: Duration) -> Self {
        self.config.negative_ttl = val;
        self
    }

    pub fn allowed_roots(mut self, val: Vec<PathBuf>) -> Self {
        self.config.allowed_roots = Some(val);
        self
    }

    pub fn allow_http_plain(mut self, val: bool) -> Self {
        self.config.allow_http_plain = val;
        self
    }

    pub fn handle(mut self, handle: Handle) -> Self {
        self.handle = Some(handle);
        self
    }

    pub fn engine(self, engine: Engine) -> ImageCache {
        ImageCache {
            engine: Arc::new(engine),
        }
    }

    pub fn build(self) -> ImageCache {
        let handle = self.handle.unwrap_or_else(Handle::current);
        let engine = Engine::new(self.config, handle);
        ImageCache {
            engine: Arc::new(engine),
        }
    }
}
