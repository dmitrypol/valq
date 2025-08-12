use crate::utils;
use getset::{Getters, Setters};
use std::collections::BTreeMap;
use valkey_module::ValkeyValue;

#[derive(Debug, Clone, Default, Getters, Setters)]
pub(crate) struct ValqMsg {
    #[getset(get = "pub")]
    id: u64,
    #[getset(get = "pub")]
    body: String,
    #[getset(get = "pub", set = "pub")]
    timeout_at: Option<u64>,
}

impl ValqMsg {
    pub(crate) fn new(id: u64, body: String, timeout_at: Option<u64>) -> Self {
        Self {
            id,
            body,
            timeout_at,
        }
    }

    // return true if without timeout_at or with timeout_at in the past
    pub(crate) fn is_visible(&self) -> bool {
        match self.timeout_at {
            Some(timeout) => timeout <= utils::now_as_seconds(),
            None => true,
        }
    }
}

impl From<ValqMsg> for ValkeyValue {
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
    use crate::utils::now_as_seconds;
    use valkey_module::redisvalue::ValkeyValueKey;

    #[test]
    fn valq_msg_create() {
        let msg = ValqMsg::new(42, "test msg".to_string(), None);
        assert_eq!(*msg.id(), 42);
        assert_eq!(msg.body(), "test msg");
        assert_eq!(*msg.timeout_at(), None);
    }

    #[test]
    fn is_visible_no_timeout() {
        let msg = ValqMsg::new(42, "test msg".to_string(), None);
        assert!(msg.is_visible());
    }

    #[test]
    fn is_visible_timeout_at_current_time() {
        let current_timeout = Some(now_as_seconds());
        let msg = ValqMsg::new(42, "test msg".to_string(), current_timeout);
        assert!(msg.is_visible());
    }

    #[test]
    fn is_visible_timeout_in_past() {
        let past_timeout = Some(now_as_seconds() - 10);
        let msg = ValqMsg::new(42, "test msg".to_string(), past_timeout);
        assert!(msg.is_visible());
    }

    #[test]
    fn is_visible_timeout_in_future() {
        let future_timeout = Some(now_as_seconds() + 10);
        let msg = ValqMsg::new(42, "test msg".to_string(), future_timeout);
        assert!(!msg.is_visible());
    }

    #[test]
    fn valq_msg_impl_valkey_value() {
        let msg = ValqMsg::new(42, "test msg".to_string(), None);
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
