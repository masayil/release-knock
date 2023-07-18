use microkv::MicroKV;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Release {
    pub url: String,
    pub name: String,
    pub detail: ReleaseDetail,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ReleaseDetail {
    #[serde(rename = "name")]
    pub release_name: String,
    pub tag_name: String,
    pub prerelease: bool,
    pub published_at: String,
    pub html_url: String,
}

pub enum DbKeyFlag {
    Exist,
    NotExist,
    FnFail,
}

impl Release {
    pub fn new(url: String, name: String, detail: ReleaseDetail) -> Self {
        Release { url, name, detail }
    }
}

pub fn key_in_db_status(db: MicroKV, key: &str) -> DbKeyFlag {
    match db.exists(key) {
        Err(_) => DbKeyFlag::FnFail,
        Ok(flag) => {
            if flag {
                DbKeyFlag::Exist
            } else {
                DbKeyFlag::NotExist
            }
        }
    }
}
