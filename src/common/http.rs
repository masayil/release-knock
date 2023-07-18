use crate::params::{HEADER_ACCEPT, X_GIT_HUB_API_VERSION};
use anyhow::Context;
use reqwest::header::{self, HeaderMap, HeaderValue};

pub fn get_github_api_headers(token: String) -> anyhow::Result<HeaderMap> {
    let authorization_header_value = token
        .parse::<HeaderValue>()
        .context("cannot parse the given token to header value")?;
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static(X_GIT_HUB_API_VERSION),
    );
    headers.insert(header::ACCEPT, HeaderValue::from_static(HEADER_ACCEPT));
    headers.insert(header::AUTHORIZATION, authorization_header_value);

    Ok(headers)
}
