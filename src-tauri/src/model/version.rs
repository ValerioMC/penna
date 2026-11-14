use std::fmt;

use super::error::{PennaError, PennaResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    pub fn parse(text: &str) -> PennaResult<Self> {
        let invalid = || PennaError::InvalidVersion(text.to_string());
        let mut numbers = text.trim().trim_start_matches('v').split('.');
        let mut next_number = || numbers.next()?.parse::<u64>().ok();
        let version = Self {
            major: next_number().ok_or_else(invalid)?,
            minor: next_number().ok_or_else(invalid)?,
            patch: next_number().ok_or_else(invalid)?,
        };
        match numbers.next() {
            None => Ok(version),
            Some(_) => Err(invalid()),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_release_tag_and_a_bare_version_parse_to_the_same_version() {
        assert_eq!(
            Version::parse("v0.2.1").unwrap(),
            Version::parse("0.2.1").unwrap()
        );
    }

    #[test]
    fn versions_compare_by_number_not_by_text() {
        assert!(Version::parse("0.10.0").unwrap() > Version::parse("0.9.0").unwrap());
    }

    #[test]
    fn text_that_is_not_three_numbers_is_rejected() {
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("1.2.3.4").is_err());
        assert!(Version::parse("latest").is_err());
    }
}
