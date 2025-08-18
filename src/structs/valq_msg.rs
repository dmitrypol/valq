use crate::utils;
use getset::{Getters, Setters};
use std::collections::BTreeMap;
use valkey_module::ValkeyValue;

/// Represents a message in the queue with metadata such as ID, body, timeout, and delivery attempts.
#[derive(Debug, Clone, Default, Getters, Setters, Ord, Eq, PartialEq, PartialOrd, Hash)]
pub(crate) struct ValqMsg {
    /// Unique identifier for the message.
    #[getset(get = "pub")]
    id: u64,

    /// The content or payload of the message.
    #[getset(get = "pub")]
    body: String,

    /// Optional timeout timestamp (in seconds) indicating when the message expires.
    #[getset(get = "pub", set = "pub")]
    timeout_at: Option<u64>,

    /// The number of times the message has been delivered.
    #[getset(get = "pub", set = "pub")]
    delivery_attempts: u64,
}

impl ValqMsg {
    /// Creates a new `ValqMsg` instance.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the message.
    /// * `body` - The content or payload of the message.
    /// * `timeout_at` - Optional timeout timestamp (in seconds).
    /// * `delivery_attempts` - Initial number of delivery attempts.
    ///
    /// # Returns
    /// A new `ValqMsg` instance with the provided values.
    pub(crate) fn new(
        id: u64,
        body: String,
        timeout_at: Option<u64>,
        delivery_attempts: u64,
    ) -> Self {
        Self {
            id,
            body,
            timeout_at,
            delivery_attempts,
        }
    }

    /// Checks if the message has expired based on its `timeout_at` value.
    ///
    /// # Returns
    /// * `true` - If `timeout_at` is `None` or the timeout is in the past.
    /// * `false` - If the timeout is in the future.
    pub(crate) fn check_timeout_at(&self) -> bool {
        match self.timeout_at {
            Some(timeout) => timeout <= utils::now_as_seconds(),
            None => true,
        }
    }

    /// Checks if the message can still be delivered based on the maximum allowed delivery attempts.
    ///
    /// # Arguments
    /// * `max_delivery_attempts` - The maximum number of delivery attempts allowed.
    ///
    /// # Returns
    /// * `true` - If the current delivery attempts are less than the maximum allowed.
    /// * `false` - If the delivery attempts have reached or exceeded the maximum allowed.
    pub(crate) fn check_max_delivery_attempts(&self, max_delivery_attempts: u64) -> bool {
        self.delivery_attempts < max_delivery_attempts
    }
}

impl From<ValqMsg> for ValkeyValue {
    /// Converts a `ValqMsg` instance into a `ValkeyValue` representation.
    ///
    /// # Returns
    /// A `ValkeyValue::OrderedMap` containing the message's ID and body as key-value pairs.
    fn from(msg: ValqMsg) -> Self {
        ValkeyValue::OrderedMap(BTreeMap::from([
            ("id".into(), msg.id().to_string().into()),
            ("body".into(), msg.body().into()),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DELIVERY_ATTEMPTS_DEFAULT;
    use crate::utils::now_as_seconds;
    use valkey_module::redisvalue::ValkeyValueKey;

    #[test]
    fn valq_msg_create() {
        let msg = ValqMsg::new(42, "test msg".to_string(), None, 0);
        assert_eq!(*msg.id(), 42);
        assert_eq!(msg.body(), "test msg");
        assert_eq!(*msg.timeout_at(), None);
    }

    #[test]
    fn no_timeout() {
        let msg = ValqMsg::new(42, "test msg".to_string(), None, 0);
        assert!(msg.check_timeout_at());
    }

    #[test]
    fn timeout_at_current_time() {
        let msg = ValqMsg::new(42, "test msg".to_string(), Some(now_as_seconds()), 0);
        assert!(msg.check_timeout_at());
    }

    #[test]
    fn timeout_in_past() {
        let msg = ValqMsg::new(42, "test msg".to_string(), Some(now_as_seconds() - 10), 0);
        assert!(msg.check_timeout_at());
    }

    #[test]
    fn timeout_in_future() {
        let msg = ValqMsg::new(42, "test msg".to_string(), Some(now_as_seconds() + 10), 0);
        assert!(!msg.check_timeout_at());
    }

    #[test]
    fn valq_msg_update_timeout_at() {
        let mut msg = ValqMsg::new(42, "test msg".to_string(), None, 0);
        let new_timeout = Some(now_as_seconds() + 100);
        msg.set_timeout_at(new_timeout);
        assert_eq!(*msg.timeout_at(), new_timeout);
        assert!(!msg.check_timeout_at());
    }

    #[test]
    fn max_delivery_attempts() {
        let msg = ValqMsg::new(1, "msg".to_string(), None, 2);
        assert!(!msg.check_max_delivery_attempts(1));
    }

    #[test]
    fn max_delivery_attempts_exceeded() {
        let msg = ValqMsg::new(1, "msg".to_string(), None, DELIVERY_ATTEMPTS_DEFAULT + 1);
        assert!(!msg.check_max_delivery_attempts(DELIVERY_ATTEMPTS_DEFAULT));
    }

    #[test]
    fn valq_msg_update_delivery_attempts() {
        let mut msg = ValqMsg::new(42, "test msg".to_string(), None, 0);
        msg.set_delivery_attempts(3);
        assert_eq!(*msg.delivery_attempts(), 3);
        assert!(msg.check_max_delivery_attempts(DELIVERY_ATTEMPTS_DEFAULT));
    }

    #[test]
    fn valq_msg_impl_valkey_value() {
        let msg = ValqMsg::new(42, "test msg".to_string(), None, 0);
        match msg.into() {
            ValkeyValue::OrderedMap(map) => {
                assert_eq!(
                    map.get(&ValkeyValueKey::String("id".to_string())).unwrap(),
                    &ValkeyValue::BulkString("42".to_string())
                );
                assert_eq!(
                    map.get(&ValkeyValueKey::String("body".to_string()))
                        .unwrap(),
                    &ValkeyValue::BulkString("test msg".to_string())
                );
            }
            _ => panic!("Expected ValkeyValue::OrderedMap"),
        }
    }
}
