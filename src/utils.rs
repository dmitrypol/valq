use crate::MIN_VALID_SERVER_VERSION;
use std::time::{SystemTime, UNIX_EPOCH};
use valkey_module::Version;

pub(crate) fn now_as_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub(crate) fn valid_server_version(version: Version) -> bool {
    let server_version = &[version.major, version.minor, version.patch];
    server_version >= MIN_VALID_SERVER_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_server_version() {
        let version = Version {
            major: 7,
            minor: 2,
            patch: 8,
        };
        assert!(valid_server_version(version));

        let version = Version {
            major: 7,
            minor: 2,
            patch: 9,
        };
        assert!(valid_server_version(version));

        let version = Version {
            major: 7,
            minor: 2,
            patch: 7,
        };
        assert!(!valid_server_version(version));
    }
}
