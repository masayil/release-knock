use crate::params::{db, ALERT_SEND_TIMEOUT, SLACK_COLOR, SLACK_CONTENT_TYPE};
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
                "Slack http response code is {}.",
                resp.status().as_u16()
            ));
        }
        Ok(())
    }

    fn build_http_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(SLACK_CONTENT_TYPE),
        );
        headers
    }

    fn build_http_body(release: db::Release) -> Bytes {
        let msg = format!(
            "*name:* {}\n*tag:* {}\n*release_name:* {}\n*publish_at:* {}\n*url:* {}\n",
            release.name,
            release.detail.tag_name,
            release.detail.release_name,
            release.detail.published_at,
            release.detail.html_url,
        );
        let body = SlackNoticeBlock {
            type_alias: "section".to_string(),
            text: SlackNoticeText {
                type_alias: "mrkdwn".to_string(),
                text: msg,
            },
        };
        let header = SlackNoticeBlock {
            type_alias: "header".to_string(),
            text: SlackNoticeText {
                type_alias: "plain_text".to_string(),
                text: "New Github Release Version".to_string(),
            },
        };
        let attachment = SlackNoticeAttachment {
            color: SLACK_COLOR.to_string(),
            blocks: vec![header, body],
        };
        let slack_notice = SlackNotice {
            attachments: vec![attachment],
        };

        let tmp = json!(slack_notice).to_string();
        trace!("slack json content: {}", tmp);
        Bytes::from(tmp)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SlackNotice {
    #[serde(rename = "attachments")]
    attachments: Vec<SlackNoticeAttachment>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SlackNoticeAttachment {
    #[serde(rename = "blocks")]
    blocks: Vec<SlackNoticeBlock>,
    #[serde(rename = "color")]
    color: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SlackNoticeBlock {
    #[serde(rename = "text")]
    text: SlackNoticeText,
    #[serde(rename = "type")]
    type_alias: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SlackNoticeText {
    #[serde(rename = "text")]
    text: String,
    #[serde(rename = "type")]
    type_alias: String,
}
