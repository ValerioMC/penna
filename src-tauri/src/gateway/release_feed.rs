use std::time::Duration;

use serde::Deserialize;

use crate::model::error::{PennaError, PennaResult};
use crate::model::version::Version;

const LATEST_RELEASE_API: &str = "https://api.github.com/repos/ValerioMC/penna/releases/latest";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

pub trait ReleaseFeed: Send + Sync {
    fn latest_version(&self) -> PennaResult<Version>;
}

#[derive(Deserialize)]
struct LatestRelease {
    tag_name: String,
}

pub struct GithubReleaseFeed;

impl ReleaseFeed for GithubReleaseFeed {
    fn latest_version(&self) -> PennaResult<Version> {
        let release: LatestRelease = ureq::get(LATEST_RELEASE_API)
            .config()
            .timeout_global(Some(REQUEST_TIMEOUT))
            .build()
            .header("Accept", "application/vnd.github+json")
            .call()
            .map_err(feed_error)?
            .body_mut()
            .read_json()
            .map_err(feed_error)?;
        Version::parse(&release.tag_name)
    }
}

fn feed_error(error: ureq::Error) -> PennaError {
    PennaError::ReleaseFeed(error.to_string())
}
