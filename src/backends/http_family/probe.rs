use crate::backends::BackendError;
use reqwest::Url;

pub async fn probe_range_support(
    client: &reqwest::Client,
    url: &Url,
) -> Result<Option<u64>, BackendError> {
    let response = match client.head(url.clone()).send().await {
        Ok(response) => response,
        Err(_) => return Ok(None),
    };

    if !response.status().is_success() {
        return Ok(None);
    }

    let total_bytes = response.content_length();
    let accepts_ranges = response
        .headers()
        .get("accept-ranges")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_lowercase().contains("bytes"))
        .unwrap_or(false);

    if accepts_ranges {
        Ok(total_bytes)
    } else {
        Ok(None)
    }
}
