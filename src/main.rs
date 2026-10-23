use reqwest::{
    StatusCode,
    header::{HeaderMap, HeaderValue},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::str::FromStr;
use thiserror::Error;

mod args;
mod utils;

use utils::{format, get_end};

#[derive(Deserialize)]
struct TimelineItemViewModel {
    timestamp: String,
    #[serde(rename = "contentItems")]
    content_items: Vec<TranscriptSegmentViewModelContainer>,
}

#[derive(Deserialize)]
struct TranscriptSegmentViewModelContainer {
    #[serde(rename = "transcriptSegmentViewModel")]
    transcript_segment_view_model: TranscriptSegmentViewModel,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
struct TranscriptSegmentViewModel {
    simple_text: String,
    timestamp_utf16_length: u16,
}

type Result<T> = std::result::Result<T, TranscriptError>;

#[derive(Error, Debug)]
enum TranscriptError {
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("no chapters exist")]
    NoChapters,

    #[error("no segments exist")]
    NoSegments,
}

struct Segment(TimelineItemViewModel);

struct Transcript {
    pub segments: Vec<Segment>,
}

impl Transcript {
    fn new(text: &str) -> Result<Self> {
        let seg_container = get_segments(text)?;

        Ok(Self {
            segments: seg_container,
        })
    }

    fn print(&self) {
        for segment in &self.segments {
            println!(
                "{}: {}",
                segment.0.timestamp,
                segment.0.content_items[0]
                    .transcript_segment_view_model
                    .simple_text
            );
        }
    }

    fn print_vtt(&self) {
        println!("WEBVTT");
        println!();

        let timeline_item_list = &self.segments;

        for (index, segment) in timeline_item_list.iter().enumerate() {
            let start = &segment.0.timestamp;
            let end = timeline_item_list
                .get(index + 1)
                .map(|next_segment| &next_segment.0.timestamp);

            println!("{}", index + 1);
            println!(
                "{} --> {}",
                format(start).unwrap(),
                format(end.unwrap_or(&get_end(start).unwrap())).unwrap()
            );
            println!(
                "{}",
                segment.0.content_items[0]
                    .transcript_segment_view_model
                    .simple_text,
            );
            println!();
        }
    }
}

enum Chapter<'a> {
    Single(&'a Vec<Value>),
    Many(&'a Vec<Value>),
}

fn get_segments(text: &str) -> Result<Vec<Segment>> {
    let response = serde_json::Value::from_str(text)?;

    let chapters = {
        let chapters = response["content"]["engagementPanelSectionListRenderer"]["content"]["sectionListRenderer"]
            ["contents"].as_array().ok_or( TranscriptError::NoChapters)?;

        if chapters.len() == 1 {
            Chapter::Single(
                chapters[0]["itemSectionRenderer"]["contents"]
                    .as_array()
                    .ok_or(TranscriptError::NoSegments)?,
            )
        } else if chapters.len() > 1 {
            Chapter::Many(chapters)
        } else {
            return Err(TranscriptError::NoChapters);
        }
    };

    parse_chapters(chapters)
}

fn parse_chapters(chapters: Chapter) -> Result<Vec<Segment>> {
    let mut timeline_item_list: Vec<Segment> = Vec::new();

    match chapters {
        Chapter::Single(segs) => {
            for seg in segs {
                let timeline_item = serde_json::from_value::<TimelineItemViewModel>(
                    seg["macroMarkersPanelItemViewModel"]["item"]["timelineItemViewModel"].clone(),
                )?;

                timeline_item_list.push(Segment(timeline_item));
            }
        }
        Chapter::Many(chapters) => {
            for chapter in chapters {
                for seg in chapter["itemSectionRenderer"]["contents"]
                    .as_array()
                    .ok_or(TranscriptError::NoSegments)?
                {
                    let timeline_item = serde_json::from_value::<TimelineItemViewModel>(
                        seg["macroMarkersPanelItemViewModel"]["item"]["timelineItemViewModel"]
                            .clone(),
                    )?;

                    timeline_item_list.push(Segment(timeline_item));
                }
            }
        }
    }

    Ok(timeline_item_list)
}

#[derive(Debug, Error)]
enum YtTransError {
    #[error("Request failed. Status Code: {0}")]
    StatusNotOk(StatusCode),

    #[error("Passed URL dones't contain the search parameter 'v'")]
    InvalidUrl,

    #[error("Request Failed. Reqwest Error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

fn make_api_request() -> std::result::Result<String, YtTransError> {
    let args = args::YTransArgs::init();

    let url = "https://www.youtube.com/youtubei/v1/get_panel?prettyPrint=false";

    let mut headers = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_static("application/json"));
    headers.insert(
        "origin",
        HeaderValue::from_static("https://www.youtube.com"),
    );
    headers.insert(
        "user-agent",
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36"),
    );

    let params = args.link.get_params().ok_or(YtTransError::InvalidUrl)?;

    let payload = json!({
        "context": {
            "client": {
                "hl": "en",
                "userAgent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36,gzip(gfe)",
                "clientName": "WEB",
                "clientVersion": "2.20260819.01.00",
                "osName": "Windows",
                "osVersion": "10.0",
                "platform": "DESKTOP",
                "clientFormFactor": "UNKNOWN_FORM_FACTOR",
                "windowWidthPoints": 794,
                "screenDensityFloat": 1.0,
                "userInterfaceTheme": "USER_INTERFACE_THEME_DARK",
                "browserName": "Chrome",
                "browserVersion": "151.0.0.0",
                "acceptHeader": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
                "screenWidthPoints": 794,
                "screenHeightPoints": 765,
                "screenPixelDensity": 1,
                "utcOffsetMinutes": 330,
                "applicationState": "ACTIVE",
                "connectionType": "CONN_CELLULAR_3G",
            },
        },
        "panelId": "PAmodern_transcript_view",
        "params": params
    });

    let response = reqwest::blocking::Client::new()
        .post(url)
        .headers(headers)
        .json(&payload)
        .send()?;

    let status = response.status();
    let text = response.text()?;

    if status != StatusCode::OK {
        return Err(YtTransError::StatusNotOk(status));
    }

    Ok(text)
}

fn main() -> anyhow::Result<()> {
    let text = make_api_request()?;
    let transcript = Transcript::new(&text)?;

    let args = args::YTransArgs::init();
    if let Some(f) = args.format {
        match f {
            args::Format::Vtt => {
                transcript.print_vtt();
            }
        }
    } else {
        transcript.print();
    }

    Ok(())
}
