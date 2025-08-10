use getset::{Getters, MutGetters, Setters};
use std::collections::{BTreeMap, HashMap, VecDeque};
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

#[derive(Debug, Clone, Getters, Setters, MutGetters, Default)]
pub(crate) struct ValqType {
    #[getset(get = "pub", set = "pub")]
    id_sequence: u64,
    #[getset(get = "pub", get_mut = "pub")]
    msgs: VecDeque<ValqMsg>,
    #[getset(get = "pub", get_mut = "pub")]
    msgs_in_flight: HashMap<u64, String>,
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

    #[test]
    fn valq_type_init_empty() {
        let valq = ValqType::default();
        assert_eq!(*valq.id_sequence(), 0);
        assert!(valq.msgs().is_empty());
        assert!(valq.msgs_in_flight().is_empty());
    }

    #[test]
    fn valq_type_add_msg() {
        let mut valq = ValqType::default();
        let msg = ValqMsg::new(42, "test msg".to_string());
        valq.msgs_mut().push_back(msg);
        assert_eq!(valq.msgs().len(), 1);
        assert_eq!(valq.msgs().front().unwrap().body(), "test msg");
    }

    #[test]
    fn valq_type_updates_id_sequence() {
        let mut valq = ValqType::default();
        valq.set_id_sequence(5);
        assert_eq!(*valq.id_sequence(), 5);
        valq.set_id_sequence(valq.id_sequence() + 1);
        assert_eq!(*valq.id_sequence(), 6);
    }
}
