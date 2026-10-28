use std::path::PathBuf;
use url::Url;

use crate::Result;

pub enum Source {
    Url(Url),
    File(PathBuf),
}

impl Source {
    pub fn parse(input: &str) -> Result<Self> {
        if let Ok(url) = Url::parse(input) {
            if url.scheme().len() > 1 {
                if url.scheme() == "file" {
                    if let Ok(path) = url.to_file_path() {
                        return Ok(Source::File(path));
                    }
                }
                return Ok(Source::Url(url));
            }
        }

        Ok(Source::File(PathBuf::from(input)))
    }
}
