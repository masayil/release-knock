use super::alert;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ServerConfig {
    #[serde(rename = "githubAuthorizationToken")]
    pub github_authorization_token: String,

    #[serde(rename = "dbPath")]
    pub db_path: PathBuf,

    #[serde(rename = "period")]
    // Convert the unit of period to seconds
    pub period: u64,

    #[serde(rename = "retryInterval")]
    // Convert the unit of retry_interval to seconds
    pub retry_interval: u64,

    #[serde(rename = "alert")]
    pub alert: alert::AlertConfig,

    #[serde(rename = "repoList")]
    pub repo_list: Vec<Repo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repo {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "url")]
    pub url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let mut current_home_dir = home::home_dir().expect("cannot get the home dir");
        current_home_dir.push(".release-knock");
        Self {
            github_authorization_token: String::from(""),
            db_path: current_home_dir,
            period: 7200,
            retry_interval: 600,
            alert: Default::default(),
            repo_list: Vec::new(),
        }
    }
}
