//! Node version declaration parsing and resolution with source tracing.

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionSource {
    NerdJson,
    Nvmrc,
    NodeVersion,
    Engines,
    Default,
}

impl fmt::Display for VersionSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NerdJson => formatter.write_str("nerd.json"),
            Self::Nvmrc => formatter.write_str(".nvmrc"),
            Self::NodeVersion => formatter.write_str(".node-version"),
            Self::Engines => formatter.write_str("package.json engines.node"),
            Self::Default => formatter.write_str("default"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VersionSpec {
    Exact(String),
    Major(u32),
    Range(String),
    Lts,
}

/// Parse a declaration into a version spec. Returns the source tag that the
/// caller associates with the declaration.
pub fn parse_spec(declaration: &str) -> Option<VersionSpec> {
    let trimmed = declaration.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = trimmed.trim_start_matches('v').trim_start_matches('V');
    if normalized.eq_ignore_ascii_case("lts") || normalized.eq_ignore_ascii_case("lts/*") {
        return Some(VersionSpec::Lts);
    }
    // Exact "20" or "20.11.1" (simple two/three-part). A leading "=" is exact.
    let simple = normalized.trim_start_matches('=');
    if let Ok(major) = simple.parse::<u32>() {
        if !simple.contains('.') {
            return Some(VersionSpec::Major(major));
        }
        return Some(VersionSpec::Exact(simple.to_owned()));
    }
    if normalized.starts_with("^") || normalized.starts_with(">=") || normalized.starts_with("~") {
        return Some(VersionSpec::Range(normalized.to_owned()));
    }
    Some(VersionSpec::Exact(normalized.to_owned()))
}

/// Compare two "MAJOR.MINOR.PATCH" version strings. Returns None if either is
/// not a parseable numeric version.
pub fn compare_versions(left: &str, right: &str) -> Option<std::cmp::Ordering> {
    let parse = |value: &str| -> Option<Vec<u32>> {
        let value = value.trim_start_matches('v');
        value
            .split('.')
            .map(|part| part.parse::<u32>().ok())
            .collect::<Option<Vec<_>>>()
    };
    let left = parse(left)?;
    let right = parse(right)?;
    for (a, b) in left.iter().zip(right.iter()) {
        let ordering = a.cmp(b);
        if ordering != std::cmp::Ordering::Equal {
            return Some(ordering);
        }
    }
    Some(left.len().cmp(&right.len()))
}

/// Extract the "MAJOR.MINOR.PATCH" prefix from a full version string such as
/// "v20.11.1" or "20.11.1".
pub fn normalize_version(version: &str) -> String {
    version.trim_start_matches('v').to_owned()
}

#[cfg(test)]
mod tests {
    use super::{VersionSpec, compare_versions, normalize_version, parse_spec};

    #[test]
    fn parses_common_declarations() {
        assert_eq!(parse_spec("20"), Some(VersionSpec::Major(20)));
        assert_eq!(
            parse_spec("v20.11.1"),
            Some(VersionSpec::Exact("20.11.1".into()))
        );
        assert_eq!(parse_spec("lts/*"), Some(VersionSpec::Lts));
        assert_eq!(parse_spec("^20"), Some(VersionSpec::Range("^20".into())));
        assert_eq!(parse_spec(""), None);
    }

    #[test]
    fn compares_and_normalizes_versions() {
        assert_eq!(
            compare_versions("20.11.1", "20.11.1"),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            compare_versions("20.11.1", "20.12.0"),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            compare_versions("21.0.0", "20.11.1"),
            Some(std::cmp::Ordering::Greater)
        );
        assert_eq!(normalize_version("v20.11.1"), "20.11.1");
        assert_eq!(normalize_version("20.11.1"), "20.11.1");
    }
}
