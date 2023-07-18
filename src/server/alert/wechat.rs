use crate::params::{db, ALERT_SEND_TIMEOUT, WECHAT_CONTENT_TYPE};
use anyhow::anyhow;
use bytes::Bytes;
use log::trace;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct AlertProvider {
    #[serde(rename = "webhook-url")]
    pub webhook_url: String,
}

impl AlertProvider {
    pub async fn send(&self, release: db::Release) -> anyhow::Result<()> {
        let headers = AlertProvider::build_http_headers();
        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(ALERT_SEND_TIMEOUT))
            .build()?;
        let body = AlertProvider::build_http_body(release);

        let resp = client
            .post(self.webhook_url.clone())
            .body(body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!(
                "Wechat http response code is {}.",
                resp.status().as_u16()
            ));
        }
        Ok(())
    }

    fn build_http_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(WECHAT_CONTENT_TYPE),
        );
        headers
    }

    fn build_http_body(release: db::Release) -> Bytes {
        let msg = format!(
            "**<font color=\"warning\">New Github Release Version</font>**\n> name: <font color=\"info\">{}</font>\n\
            > tag: <font color=\"info\">{}</font>\n> release_name: <font color=\"info\">{}</font>\n\
            > published_at: <font color=\"info\">{}</font>\n> url: <font color=\"info\">{}</font>",
            release.name,
            release.detail.tag_name,
            release.detail.release_name,
            release.detail.published_at,
            release.detail.html_url,
        );
        let wx_data = WxData {
            msgtype: "markdown".to_string(),
            markdown: WxMarkdwon { content: msg },
        };

        let tmp = json!(wx_data).to_string();
        trace!("wechat json content: {}", tmp);
        Bytes::from(tmp)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WxData {
    #[serde(rename = "markdown")]
    markdown: WxMarkdwon,
    #[serde(rename = "msgtype")]
    msgtype: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WxMarkdwon {
    #[serde(rename = "content")]
    content: String,
}
