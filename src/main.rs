use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;

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
struct TranscriptSegmentViewModel {
    simple_text: String,
    timestamp_utf16_length: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
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
        "params": "qgkPCgtzcmFhaWQ0ZGdZdxgC"
    });

    let response = reqwest::Client::new()
        .post(url)
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;

    println!("Status: {status}");

    let response = serde_json::Value::from_str(&text)?;

    let contents =
        response["content"]["engagementPanelSectionListRenderer"]["content"]["sectionListRenderer"]
            ["contents"][0]["itemSectionRenderer"]["contents"]
            .as_array()
            .context("no contents found in the response")?;

    for item in contents {
        let timeline_item = serde_json::from_value::<TimelineItemViewModel>(
            item["macroMarkersPanelItemViewModel"]["item"]["timelineItemViewModel"].clone(),
        )?;

        println!(
            "[{}] {}",
            timeline_item.timestamp,
            timeline_item.content_items[0]
                .transcript_segment_view_model
                .simple_text
        );
    }

    Ok(())
}
