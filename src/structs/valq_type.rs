use crate::structs::valq_msg::ValqMsg;
use getset::{Getters, MutGetters, Setters};
use std::collections::VecDeque;

#[derive(Debug, Clone, Getters, Setters, MutGetters)]
pub(crate) struct ValqType {
    #[getset(get = "pub", set = "pub")]
    id_sequence: u64,
    #[getset(get = "pub")]
    visibility_timeout: u64,
    #[getset(get = "pub", get_mut = "pub")]
    msgs: VecDeque<ValqMsg>,
}

impl ValqType {
    // TODO - add validation for visibility_timeout, between 1 and 43_200 seconds (12 hours)
    pub fn new(visibility_timeout: Option<u64>) -> Self {
        Self {
            id_sequence: 0,
            visibility_timeout: visibility_timeout.unwrap_or(crate::VISIBILITY_TIMEOUT_DEFAULT),
            msgs: VecDeque::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::valq_type::ValqType;

    #[test]
    fn valq_type_init_empty() {
        let valq = ValqType::new(None);
        assert_eq!(*valq.id_sequence(), 0);
        assert!(valq.msgs().is_empty());
        assert_eq!(
            *valq.visibility_timeout(),
            crate::VISIBILITY_TIMEOUT_DEFAULT
        );
    }

    #[test]
    fn valq_type_custom_timeout() {
        let valq = ValqType::new(Some(3600));
        assert_eq!(*valq.visibility_timeout(), 3600);
    }

    #[test]
    fn valq_type_add_msg() {
        let mut valq = ValqType::new(None);
        let msg = ValqMsg::new(42, "test msg".to_string(), None);
        valq.msgs_mut().push_back(msg);
        assert_eq!(valq.msgs().len(), 1);
        assert_eq!(valq.msgs().front().unwrap().body(), "test msg");
    }

    #[test]
    fn valq_type_updates_id_sequence() {
        let mut valq = ValqType::new(None);
        valq.set_id_sequence(5);
        assert_eq!(*valq.id_sequence(), 5);
        valq.set_id_sequence(valq.id_sequence() + 1);
        assert_eq!(*valq.id_sequence(), 6);
    }
}
