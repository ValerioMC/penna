use std::sync::Arc;

use crate::gateway::release_feed::ReleaseFeed;
use crate::model::error::PennaResult;
use crate::model::version::Version;

pub struct UpdateService {
    feed: Arc<dyn ReleaseFeed>,
}

impl UpdateService {
    pub fn new(feed: Arc<dyn ReleaseFeed>) -> Self {
        Self { feed }
    }

    pub fn newer_version(&self, installed: Version) -> PennaResult<Option<Version>> {
        let latest = self.feed.latest_version()?;
        Ok((latest > installed).then_some(latest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::error::PennaError;

    struct FakeFeed(Result<&'static str, &'static str>);

    impl ReleaseFeed for FakeFeed {
        fn latest_version(&self) -> PennaResult<Version> {
            match self.0 {
                Ok(tag) => Version::parse(tag),
                Err(message) => Err(PennaError::ReleaseFeed(message.to_string())),
            }
        }
    }

    fn service(feed: FakeFeed) -> UpdateService {
        UpdateService::new(Arc::new(feed))
    }

    fn installed() -> Version {
        Version::parse("0.1.0").unwrap()
    }

    #[test]
    fn a_newer_release_is_reported() {
        let newer = service(FakeFeed(Ok("v0.2.0")))
            .newer_version(installed())
            .unwrap();
        assert_eq!(newer, Some(Version::parse("0.2.0").unwrap()));
    }

    #[test]
    fn the_installed_release_is_not_reported() {
        let newer = service(FakeFeed(Ok("v0.1.0")))
            .newer_version(installed())
            .unwrap();
        assert_eq!(newer, None);
    }

    #[test]
    fn an_older_release_is_not_reported() {
        let newer = service(FakeFeed(Ok("v0.0.9")))
            .newer_version(installed())
            .unwrap();
        assert_eq!(newer, None);
    }

    #[test]
    fn an_unreachable_feed_is_an_error() {
        let result = service(FakeFeed(Err("offline"))).newer_version(installed());
        assert!(matches!(result, Err(PennaError::ReleaseFeed(_))));
    }
}
