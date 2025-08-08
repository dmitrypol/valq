use getset::{Getters, MutGetters, Setters};
use std::collections::{HashMap, VecDeque};

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

    #[test]
    fn valq_msg_create() {
        let msg = ValqMsg::new(42, "test msg".to_string());
        assert_eq!(*msg.id(), 42);
        assert_eq!(msg.body(), "test msg");
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
