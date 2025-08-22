use crate::structs::delayed_msgs::DelayedMsgs;
use crate::structs::valq_msg::ValqMsg;
use crate::{
    DELIVERY_ATTEMPTS_DEFAULT, DELIVERY_ATTEMPTS_MAX, RETENTION_PERIOD_DEFAULT,
    RETENTION_PERIOD_MAX, RETENTION_PERIOD_MIN, VISIBILITY_TIMEOUT_DEFAULT, VISIBILITY_TIMEOUT_MAX,
};
use getset::{Getters, MutGetters, Setters};
use std::collections::VecDeque;
use valkey_module::ValkeyError;

/// Represents a job queue with configurable visibility timeout and maximum delivery attempts.
/// This structure manages a queue of messages, delayed messages and a dead-letter queue for failed messages.
#[derive(Debug, Clone, Getters, Setters, MutGetters, Default)]
pub(crate) struct ValqType {
    /// Name of the queue.
    #[getset(get = "pub", set = "pub")]
    name: String,
    /// Sequence ID for generating unique message IDs.
    #[getset(get = "pub", set = "pub")]
    id_sequence: u64,
    /// Visibility timeout for messages in seconds.
    #[getset(get = "pub")]
    visibility_timeout: u64,
    /// Maximum number of delivery attempts for a message before moving it to the dead-letter queue.
    #[getset(get = "pub")]
    max_delivery_attempts: u64,
    #[getset(get = "pub")]
    retention_period: u64,
    /// Queue of messages currently being processed.
    #[getset(get = "pub", get_mut = "pub")]
    msgs: VecDeque<ValqMsg>,
    /// Dead-letter queue for messages that failed to process after maximum delivery attempts.
    #[getset(get = "pub", get_mut = "pub")]
    dlq_msgs: VecDeque<ValqMsg>,
    /// Delayed messages that are scheduled to be processed after a certain time.
    #[getset(get = "pub", get_mut = "pub")]
    delayed_msgs: DelayedMsgs,
}

impl ValqType {
    /// Creates a new `ValqType` instance with optional visibility timeout and maximum delivery attempts.
    ///
    /// # Arguments
    /// * `name` - Name of the queue.
    /// * `visibility_timeout` - Optional visibility timeout in seconds. Defaults to `VISIBILITY_TIMEOUT_DEFAULT`.
    /// * `max_delivery_attempts` - Optional maximum delivery attempts. Defaults to `DELIVERY_ATTEMPTS_DEFAULT`.
    /// * `retention_period` - Optional retention period for DLQ. Defaults to `RETENTION_PERIOD_DEFAULT`.
    ///
    /// # Returns
    /// * `Ok(Self)` - If the provided arguments are valid.
    /// * `Err(ValkeyError)` - If the arguments are out of the allowed range.
    ///
    /// # Errors
    /// Returns an error if:
    /// * `name` is empty.
    /// * `visibility_timeout` is less than 1 or greater than `VISIBILITY_TIMEOUT_MAX`.
    /// * `max_delivery_attempts` is less than 1 or greater than `DELIVERY_ATTEMPTS_MAX`.
    /// * `retention_period` is less than 1 or greater than `RETENTION_PERIOD_MAX`.
    pub fn new(
        name: &str,
        visibility_timeout: Option<u64>,
        max_delivery_attempts: Option<u64>,
        retention_period: Option<u64>,
    ) -> Result<Self, ValkeyError> {
        if name.is_empty() {
            return Err(ValkeyError::Str("queue name cannot be empty"));
        }
        if visibility_timeout.is_some()
            && (visibility_timeout.unwrap_or_default() < 1
                || visibility_timeout.unwrap_or_default() > VISIBILITY_TIMEOUT_MAX)
        {
            return Err(ValkeyError::Str(
                "timeout must be between 1 and 43_200 seconds (12 hours)",
            ));
        }
        if max_delivery_attempts.is_some()
            && (max_delivery_attempts.unwrap_or_default() < 1
                || max_delivery_attempts.unwrap_or_default() > DELIVERY_ATTEMPTS_MAX)
        {
            return Err(ValkeyError::Str(
                "max delivery attempts must be between 1 and 20",
            ));
        }
        if retention_period.is_some()
            && (retention_period.unwrap_or_default() < RETENTION_PERIOD_MIN
                || retention_period.unwrap_or_default() > RETENTION_PERIOD_MAX)
        {
            return Err(ValkeyError::String(format!(
                "retention period must be between {} and {} seconds",
                RETENTION_PERIOD_MIN, RETENTION_PERIOD_MAX
            )));
        }
        Ok(Self {
            name: name.to_string(),
            id_sequence: 0,
            visibility_timeout: visibility_timeout.unwrap_or(VISIBILITY_TIMEOUT_DEFAULT),
            max_delivery_attempts: max_delivery_attempts.unwrap_or(DELIVERY_ATTEMPTS_DEFAULT),
            retention_period: retention_period.unwrap_or(RETENTION_PERIOD_DEFAULT),
            msgs: VecDeque::new(),
            dlq_msgs: VecDeque::new(),
            delayed_msgs: DelayedMsgs::new(),
        })
    }

    pub(crate) fn set_visibility_timeout(
        &mut self,
        visibility_timeout: u64,
    ) -> Result<String, ValkeyError> {
        if visibility_timeout < 1 || visibility_timeout > VISIBILITY_TIMEOUT_MAX {
            Err(ValkeyError::Str(
                "timeout must be between 1 and 43_200 seconds (12 hours)",
            ))
        } else {
            self.visibility_timeout = visibility_timeout;
            Ok("OK".to_string())
        }
    }

    pub(crate) fn set_max_delivery_attempts(
        &mut self,
        max_delivery_attempts: u64,
    ) -> Result<String, ValkeyError> {
        if max_delivery_attempts < 1 || max_delivery_attempts > DELIVERY_ATTEMPTS_MAX {
            Err(ValkeyError::Str(
                "max delivery attempts must be between 1 and 20",
            ))
        } else {
            self.max_delivery_attempts = max_delivery_attempts;
            Ok("OK".to_string())
        }
    }
    pub(crate) fn set_retention_period(
        &mut self,
        retention_period: u64,
    ) -> Result<String, ValkeyError> {
        if retention_period < RETENTION_PERIOD_MIN || retention_period > RETENTION_PERIOD_MAX {
            Err(ValkeyError::String(format!(
                "retention period must be between {} and {} seconds",
                RETENTION_PERIOD_MIN, RETENTION_PERIOD_MAX
            )))
        } else {
            self.retention_period = retention_period;
            Ok("OK".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::valq_type::ValqType;

    #[test]
    fn valq_type_init_empty() {
        let valq = ValqType::new("q", None, None, None).unwrap();
        assert_eq!(*valq.id_sequence(), 0);
        assert_eq!(*valq.visibility_timeout(), VISIBILITY_TIMEOUT_DEFAULT);
        assert_eq!(*valq.max_delivery_attempts(), DELIVERY_ATTEMPTS_DEFAULT);
        assert!(valq.msgs().is_empty());
        assert!(valq.dlq_msgs().is_empty());
    }

    #[test]
    fn valq_type_empty_name() {
        let test = ValqType::new("", None, None, None);
        assert!(test.is_err());
    }

    #[test]
    fn valq_type_custom_timeout_max_delivery_attempts() {
        let valq = ValqType::new("q", Some(3600), Some(10), None).unwrap();
        assert_eq!(*valq.visibility_timeout(), 3600);
        assert_eq!(*valq.max_delivery_attempts(), 10);
    }

    #[test]
    fn valq_type_visibility_timeout_max_delivery_attempts_retention_period_invalid() {
        let test = ValqType::new("q", Some(0), None, None);
        assert!(test.is_err());
        let test = ValqType::new("q", Some(VISIBILITY_TIMEOUT_MAX + 1), None, None);
        assert!(test.is_err());
        let test = ValqType::new("q", None, Some(0), None);
        assert!(test.is_err());
        let test = ValqType::new("q", None, Some(DELIVERY_ATTEMPTS_MAX + 1), None);
        assert!(test.is_err());
        let test = ValqType::new("q", None, None, Some(RETENTION_PERIOD_MIN - 1));
        assert!(test.is_err());
        let test = ValqType::new("q", None, None, Some(RETENTION_PERIOD_MAX + 1));
        assert!(test.is_err());
    }

    #[test]
    fn valq_type_add_remove_msgs() {
        let mut valq = ValqType::new("q", None, None, None).unwrap();
        let msg1 = ValqMsg::new(1, "msg1".to_string(), None, 0);
        let msg2 = ValqMsg::new(2, "msg2".to_string(), None, 0);
        valq.msgs_mut().push_back(msg1);
        valq.msgs_mut().push_back(msg2);
        assert_eq!(valq.msgs().len(), 2);
        assert_eq!(valq.msgs()[0].body(), "msg1");
        assert_eq!(valq.msgs()[1].body(), "msg2");
        valq.msgs_mut().pop_front();
        valq.msgs_mut().pop_front();
        assert!(valq.msgs().is_empty());
    }

    #[test]
    fn valq_type_update_id_sequence() {
        let mut valq = ValqType::new("q", None, None, None).unwrap();
        valq.set_id_sequence(5);
        assert_eq!(*valq.id_sequence(), 5);
        valq.set_id_sequence(valq.id_sequence() + 1);
        assert_eq!(*valq.id_sequence(), 6);
    }

    #[test]
    fn valq_type_update_visibility_timeout_max_delivery_attempts_retention_period() {
        let mut valq = ValqType::new("q", None, None, None).unwrap();
        let _ = valq.set_visibility_timeout(7200);
        assert_eq!(*valq.visibility_timeout(), 7200);
        let _ = valq.set_max_delivery_attempts(10);
        assert_eq!(*valq.max_delivery_attempts(), 10);
        let _ = valq.set_retention_period(100);
        assert_eq!(*valq.retention_period(), 100);
    }

    #[test]
    fn valq_type_set_visibility_timeout_invalid() {
        let mut valq = ValqType::new("q", None, None, None).unwrap();
        let test = valq.set_visibility_timeout(0);
        assert!(test.is_err());
        let test = valq.set_visibility_timeout(VISIBILITY_TIMEOUT_MAX + 1);
        assert!(test.is_err());
        assert_eq!(*valq.visibility_timeout(), VISIBILITY_TIMEOUT_DEFAULT);
    }

    #[test]
    fn valq_type_set_max_delivery_attempts_invalid() {
        let mut valq = ValqType::new("q", None, None, None).unwrap();
        let test = valq.set_max_delivery_attempts(0);
        assert!(test.is_err());
        let test = valq.set_max_delivery_attempts(DELIVERY_ATTEMPTS_MAX + 1);
        assert!(test.is_err());
        assert_eq!(*valq.max_delivery_attempts(), DELIVERY_ATTEMPTS_DEFAULT);
    }

    #[test]
    fn valq_type_set_retention_period_invalid() {
        let mut valq = ValqType::new("q", None, None, None).unwrap();
        let test = valq.set_retention_period(RETENTION_PERIOD_MIN - 1);
        assert!(test.is_err());
        let test = valq.set_retention_period(RETENTION_PERIOD_MAX + 1);
        assert!(test.is_err());
        assert_eq!(*valq.retention_period(), RETENTION_PERIOD_DEFAULT);
    }
}
