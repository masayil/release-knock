pub mod db;

pub(crate) const SLACK_CONTENT_TYPE: &str = "application/json";
pub(crate) const WECHAT_CONTENT_TYPE: &str = "application/json";
pub(crate) const SLACK_COLOR: &str = "#f2c744";

pub(crate) const X_GIT_HUB_API_VERSION: &str = "2022-11-28";
pub(crate) const HEADER_ACCEPT: &str = "application/vnd.github+json";

pub(crate) const RELEASE_ALERT_CHANNEL: usize = 32;

pub(crate) const ALERT_SEND_TIMEOUT: u64 = 5;

pub(crate) const ALERT_SEND_WORKERS: usize = 4;

pub(crate) const WATCH_RETRY: u8 = 1;

pub(crate) const WATCH_PULL_TIMEOUT: u64 = 5;
