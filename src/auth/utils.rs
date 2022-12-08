// 127.0.0.1:8000 -> 127.0.0.1
pub fn domain(host: &str) -> String {
    match host.split_once(':') {
        Some((domain, _port)) => domain.to_string(),
        None => host.to_string(),
    }
}
pub async fn get_api<T: serde::de::DeserializeOwned>(url: &str, token: &str) -> fpm::Result<T> {
    let response = reqwest::Client::new()
        .get(url)
        .header(reqwest::header::AUTHORIZATION, token)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("fpm"),
        )
        .send()
        .await?;

    if !response.status().eq(&reqwest::StatusCode::OK) {
        return Err(fpm::Error::APIResponseError(format!(
            "FPM-API-ERROR: {}, Error: {}",
            url,
            response.text().await?
        )));
    }

    Ok(response.json().await?)
}
