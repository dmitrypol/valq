use crate::MIN_VALID_SERVER_VERSION;
use std::time::{SystemTime, UNIX_EPOCH};
use valkey_module::{Context, ContextFlags, ValkeyError, ValkeyResult, Version};

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

/// https://valkey.io/topics/modules-api-ref/#ValkeyModule_GetContextFlagsAll
pub(crate) fn replicate_cmd_check(ctx: &Context) -> ValkeyResult {
    let flags = ctx.get_flags();
    if flags.contains(ContextFlags::READONLY) && !flags.contains(ContextFlags::REPLICATED) {
        Err(ValkeyError::Str(
            "cannot execute command directly on a replica node",
        ))
    } else {
        ctx.replicate_verbatim();
        Ok("OK".into())
    }
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
