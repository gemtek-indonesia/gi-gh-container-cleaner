mod github_package_versions;
mod github_packages;

use anyhow::Result;
use env_logger::builder as log_builder;
use github_package_versions::GithubPackageVersions;
use github_packages::GithubPackages;
use log::{debug as log_debug, error as log_error, info as log_info};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client as HttpClient, Response as HttpResponse};
use std::collections::HashSet;
use std::env;
use structopt::StructOpt;
use tokio::time::Duration;

#[derive(Debug, StructOpt)]
#[structopt(about = "Github's untagged Docker image cleaner (organization wide).")]
pub struct RunOptions {
    /// Github organization name
    #[structopt(short = "o", long = "organization")]
    organization: String,
    /// Github PAT for list & delete package in organization scope
    #[structopt(short = "p", long = "pat")]
    personal_access_token: String,
}

impl RunOptions {
    const ENV_RUST_LOG: &'static str = "RUST_LOG";
    const HEADER_ACCEPT: (&'static str, &'static str) = ("Accept", "application/vnd.github+json");
    const HEADER_APIVER: (&'static str, &'static str) = ("X-GitHub-Api-Version", "2022-11-28");

    fn get_url_org_list_packages(&self) -> String {
        format!(
            "https://api.github.com/orgs/{}/packages?package_type=container",
            self.organization
        )
    }

    fn get_url_org_list_package_versions(&self, package_name: &str) -> String {
        format!(
            "https://api.github.com/orgs/{}/packages/container/{}/versions",
            self.organization, package_name
        )
    }

    fn get_url_org_delete_package_version(&self, package_name: &str, package_id: i64) -> String {
        format!(
            "https://api.github.com/orgs/{}/packages/container/{}/versions/{}",
            self.organization, package_name, package_id
        )
    }

    fn get_preconfigured_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(Self::HEADER_ACCEPT.0, HeaderValue::from_static(Self::HEADER_ACCEPT.1));
        headers.insert(Self::HEADER_APIVER.0, HeaderValue::from_static(Self::HEADER_APIVER.1));
        headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));

        headers
    }

    async fn get_response(&self, url: &str) -> Result<HttpResponse> {
        let headers = self.get_preconfigured_headers();
        let http_client = HttpClient::builder()
            .https_only(true)
            .connection_verbose(true)
            .http1_only()
            .http1_title_case_headers()
            .use_native_tls()
            .tcp_nodelay(true)
            .timeout(Duration::from_secs(10))
            .build()?;
        let request = http_client
            .get(url)
            .bearer_auth(&self.personal_access_token)
            .headers(headers)
            .build()?;
        let response = http_client.execute(request).await?;

        Ok(response)
    }

    async fn delete_response(&self, url: &str) -> Result<HttpResponse> {
        let headers = self.get_preconfigured_headers();
        let http_client = HttpClient::builder()
            .https_only(true)
            .connection_verbose(true)
            .http1_only()
            .http1_title_case_headers()
            .use_native_tls()
            .tcp_nodelay(true)
            .timeout(Duration::from_secs(10))
            .build()?;
        let request = http_client
            .delete(url)
            .bearer_auth(&self.personal_access_token)
            .headers(headers)
            .build()?;
        let response = http_client.execute(request).await?;

        Ok(response)
    }

    pub async fn get_package_names(&self) -> Result<Vec<String>> {
        let response = self.get_response(&self.get_url_org_list_packages()).await?;
        log_debug!("{response:#?}");
        let packages = response.json::<GithubPackages>().await?;
        let mut package_names = HashSet::<String>::new();

        for package_item in packages {
            package_names.insert(package_item.name);
        }

        Ok(package_names.into_iter().collect())
    }

    pub async fn get_package_dangling_indices(&self, package_name: &str) -> Result<Vec<i64>> {
        let response = self
            .get_response(&self.get_url_org_list_package_versions(package_name))
            .await?;
        log_debug!("{response:#?}");
        let package_versions = response.json::<GithubPackageVersions>().await?;

        Ok(package_versions.get_all_dangling_indices())
    }

    pub async fn delete_package_dangling_id(&self, package_name: &str, package_id: i64) -> Result<()> {
        let response = self
            .delete_response(&self.get_url_org_delete_package_version(package_name, package_id))
            .await?;
        log_debug!("{response:#?}");
        response.error_for_status()?;

        Ok(())
    }
}

impl Default for RunOptions {
    fn default() -> Self {
        #[cfg(not(debug_assertions))]
        {
            if env::var(Self::ENV_RUST_LOG).is_err() {
                env::set_var(Self::ENV_RUST_LOG, "info");
            }

            log_builder()
                .default_format()
                .format_timestamp_millis()
                .format_indent(Some(4))
                .init();
        }
        #[cfg(debug_assertions)]
        {
            if env::var(Self::ENV_RUST_LOG).is_err() {
                env::set_var(Self::ENV_RUST_LOG, "debug");
            }

            log_builder()
                .default_format()
                .format_timestamp_millis()
                .format_indent(Some(4))
                .init();
        }

        Self::from_args()
    }
}

#[tokio::main(worker_threads = 1)]
async fn main() -> Result<()> {
    let run_options = RunOptions::default();

    match run_options.get_package_names().await {
        Err(err) => log_error!("{err}"),
        Ok(package_names) => {
            log_info!("Package Names for the organizations:\n{:#?}", package_names);

            for package_name in &package_names {
                let dangling_indices = run_options.get_package_dangling_indices(package_name).await?;
                log_info!(
                    "Package \"{package_name}\" has {} untagged version(s)",
                    dangling_indices.len()
                );

                for dangling_id in dangling_indices {
                    log_info!("Deleting {} for \"{}\"", dangling_id, package_name);
                    run_options.delete_package_dangling_id(package_name, dangling_id).await?;
                }
            }
        }
    }

    Ok(())
}
