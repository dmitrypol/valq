use getset::Getters;
use std::collections::BTreeMap;
use valkey_module::ValkeyValue;

#[derive(Debug, Clone, Default, Getters)]
pub(crate) struct ValqMsg {
    #[getset(get = "pub")]
    id: u64,
    #[getset(get = "pub")]
    body: String,
}

impl ValqMsg {
    pub(crate) fn new(id: u64, body: String) -> Self {
        Self { id, body }
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
    use valkey_module::redisvalue::ValkeyValueKey;

    #[test]
    fn valq_msg_create() {
        let msg = ValqMsg::new(42, "test msg".to_string());
        assert_eq!(*msg.id(), 42);
        assert_eq!(msg.body(), "test msg");
    }

    #[test]
    fn valq_msg_impl_valkey_value() {
        let msg = ValqMsg::new(42, "test msg".to_string());
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
