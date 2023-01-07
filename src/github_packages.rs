use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct GithubPackagesItem {
    pub created_at: String,
    pub html_url: String,
    #[doc = " Unique identifier of the package."]
    pub id: i64,
    #[doc = " The name of the package."]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<serde_json::Value>,
    pub package_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<serde_json::Value>,
    pub updated_at: String,
    pub url: String,
    #[doc = " The number of versions of the package."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_count: Option<i64>,
    pub visibility: String,
}

pub type GithubPackages = Vec<GithubPackagesItem>;
