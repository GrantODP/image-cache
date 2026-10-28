use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc, time::Duration};

use moka::future::Cache;
use parking_lot::Mutex;
use tokio::runtime::Handle;
use url::Url;

use crate::image::{ImageData, ImageState};
use crate::{Error, Result};

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub max_cache_bytes: u64, // total memory budget
    pub max_item_bytes: u64,  // per-image download/file limit
    pub max_pixels: u64,      // decode limit
    pub http_timeout: Duration,
    pub max_redirects: usize,
    pub web_ttl: Option<Duration>,
    pub negative_ttl: Duration,
    pub allowed_roots: Option<Vec<PathBuf>>,
    pub allow_http_plain: bool, // permit http:// vs https-only
}

pub struct Engine {
    pub client: reqwest::Client,
    pub cache: Cache<String, Arc<ImageData>>,
    pub handle: tokio::runtime::Handle,
    pub image_state: parking_lot::Mutex<HashMap<String, ImageState>>,
    pub on_update: Option<Arc<dyn Fn() + Send + Sync>>,
    pub cfg: Config,
}

impl Engine {
    pub fn new(cfg: Config, handle: Handle) -> Self {
        let client = reqwest::Client::builder()
            .timeout(cfg.http_timeout)
            .redirect(reqwest::redirect::Policy::limited(cfg.max_redirects))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let cache = Cache::builder().max_capacity(cfg.max_cache_bytes).build();

        Self {
            client,
            cache,
            image_state: Mutex::new(HashMap::new()),
            handle,
            on_update: None,
            cfg,
        }
    }

    pub async fn fetch<S: AsRef<str>>(&self, url: S) -> Result<Arc<ImageData>> {
        let key = normalize(url.as_ref());
        self.cache
            .try_get_with(key.clone(), self.download(key))
            .await
    }

    pub(crate) async fn download<S: AsRef<str>>(
        &self,
        url: S,
    ) -> std::result::Result<Arc<ImageData>, crate::Error> {
        let resp = self
            .client
            .get(url.as_ref())
            .send()
            .await?
            .error_for_status()?;
        if let Some(len) = resp.content_length() {
            let max = self.cfg.max_item_bytes;
            if len > max {
                return Err(Error::TooLarge {
                    max: max,
                    received: len,
                });
            }
        }
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let bytes = resp.bytes().await?;
        Ok(Arc::new(ImageData {
            bytes,
            content_type: ct,
            decoded: None,
        }))
    }
}

impl fmt::Debug for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Engine")
            .field("client", &self.client)
            .field("cache", &self.cache)
            .field("states", &self.image_state)
            .field("handle", &self.handle)
            .field("on_update", &self.on_update.as_ref().map(|_| "<function>"))
            .field("cfg", &self.cfg)
            .finish()
    }
}
pub(crate) fn normalize(input: &str) -> String {
    if let Ok(mut url) = Url::parse(input) {
        url.set_fragment(None);

        return url.to_string();
    }

    if let Ok(path) = std::path::Path::new(input).canonicalize() {
        if let Some(path_str) = path.to_str() {
            return path_str.to_owned();
        }
    }

    input.trim().to_owned()
}
