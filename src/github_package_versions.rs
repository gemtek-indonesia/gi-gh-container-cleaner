use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct GithubPackageVersionsItemMetadataContainer {
    pub tags: Vec<String>,
}

impl GithubPackageVersionsItemMetadataContainer {
    pub fn is_dangling(&self) -> bool {
        self.tags.is_empty()
    }
}

#[derive(Clone, PartialEq, Debug, Default, Deserialize, Serialize)]
pub struct GithubPackageVersionsItemMetadataDocker {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<Vec<String>>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct GithubPackageVersionsItemMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<GithubPackageVersionsItemMetadataContainer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docker: Option<GithubPackageVersionsItemMetadataDocker>,
    pub package_type: String,
}

impl GithubPackageVersionsItemMetadata {
    pub fn is_dangling(&self) -> bool {
        if let Some(container_metadata) = &self.container {
            container_metadata.is_dangling()
        } else {
            false
        }
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct GithubPackageVersionsItem {
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[doc = " Unique identifier of the package version."]
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<GithubPackageVersionsItemMetadata>,
    #[doc = " The name of the package version."]
    pub name: String,
    pub package_html_url: String,
    pub updated_at: String,
    pub url: String,
}

impl GithubPackageVersionsItem {
    pub fn get_id_if_dangling(&self) -> Option<i64> {
        if let Some(metadata) = &self.metadata {
            if metadata.is_dangling() {
                Some(self.id)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct GithubPackageVersions(Vec<GithubPackageVersionsItem>);

impl GithubPackageVersions {
    pub fn get_all_dangling_indices(self) -> Vec<i64> {
        let mut dangling_indices = Vec::new();

        for package_version in self.0 {
            if let Some(id) = package_version.get_id_if_dangling() {
                dangling_indices.push(id);
            }
        }

        dangling_indices
    }
}
