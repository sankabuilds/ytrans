#![allow(unused)]

use std::path::{Path, PathBuf};

use base64::{Engine, engine::general_purpose};
use clap::{Parser, ValueEnum};
use reqwest::Url;

/// Simple program to get transcripts of YouTube videos
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Video link
    pub link: Url,

    #[arg(short, long, value_enum)]
    pub format: Option<Format>,

    #[arg(short, long)]
    pub output_dir: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Format {
    /// WebVTT
    Vtt,
}

#[derive(Debug)]
pub struct YTransArgs {
    pub link: VideoUrl,
    pub format: Option<Format>,
    pub output: Option<PathBuf>,
}

impl YTransArgs {
    pub fn init() -> Self {
        let args = Args::parse();

        Self {
            link: VideoUrl { raw_url: args.link },
            format: args.format,
            output: args.output_dir,
        }
    }
}

const PARAMS_PREFIX: &[u8; 5] = &[170, 9, 15, 10, 11];
const PARAMS_SUFFIX: &[u8; 2] = &[24, 1];

#[derive(Debug)]
pub struct VideoUrl {
    pub raw_url: Url,
}

impl VideoUrl {
    pub fn get_v(&self) -> Option<&str> {
        let query = self
            .raw_url
            .query()?
            .split("&")
            .find(|q| q.starts_with("v="))?;

        query.split("=").last()
    }

    pub fn get_params(&self) -> Option<String> {
        let v = self.get_v()?.as_bytes();

        let mut params: Vec<u8> = Vec::new();

        params.extend_from_slice(PARAMS_PREFIX);
        params.extend_from_slice(v);
        params.extend_from_slice(PARAMS_SUFFIX);

        Some(general_purpose::STANDARD.encode(params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_params() {
        let vid_url = VideoUrl {
            raw_url: "https://www.youtube.com/watch?v=pCH-R3L6QTc"
                .parse::<Url>()
                .unwrap(),
        };

        assert_eq!(
            vid_url.get_params(),
            Some("qgkPCgtwQ0gtUjNMNlFUYxgB".to_owned())
        );
    }
}
