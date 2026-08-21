use anyhow::{Context, Result, bail};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;

mod args;

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

#[tokio::main]
async fn main() -> Result<()> {
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

    let params = args.links[0].get_params().context(format!(
        "couldn't get params from the passed URL: {}",
        args.links[0].raw_url.as_str()
    ))?;

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

    let response = reqwest::Client::new()
        .post(url)
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;

    if status != 200 {
        bail!("Request failed. Status Code: {}", status);
    }

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
            "{}: {}",
            timeline_item.timestamp,
            timeline_item.content_items[0]
                .transcript_segment_view_model
                .simple_text
        );
    }

    Ok(())
}
