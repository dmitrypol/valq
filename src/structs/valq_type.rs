use crate::structs::valq_msg::ValqMsg;
use crate::{DELIVERY_ATTEMPTS_DEFAULT, VISIBILITY_TIMEOUT_DEFAULT};
use getset::{Getters, MutGetters, Setters};
use std::collections::VecDeque;

#[derive(Debug, Clone, Getters, Setters, MutGetters)]
pub(crate) struct ValqType {
    #[getset(get = "pub", set = "pub")]
    id_sequence: u64,
    #[getset(get = "pub", set = "pub")]
    visibility_timeout: u64,
    #[getset(get = "pub", set = "pub")]
    max_delivery_attempts: u64,
    #[getset(get = "pub", get_mut = "pub")]
    msgs: VecDeque<ValqMsg>,
}

impl ValqType {
    // TODO - add validation for visibility_timeout and max_delivery_attempts
    pub fn new(visibility_timeout: Option<u64>, max_delivery_attempts: Option<u64>) -> Self {
        Self {
            id_sequence: 0,
            visibility_timeout: visibility_timeout.unwrap_or(VISIBILITY_TIMEOUT_DEFAULT),
            max_delivery_attempts: max_delivery_attempts.unwrap_or(DELIVERY_ATTEMPTS_DEFAULT),
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
        let valq = ValqType::new(None, None);
        assert_eq!(*valq.id_sequence(), 0);
        assert!(valq.msgs().is_empty());
        assert_eq!(*valq.visibility_timeout(), VISIBILITY_TIMEOUT_DEFAULT);
        assert_eq!(*valq.max_delivery_attempts(), DELIVERY_ATTEMPTS_DEFAULT);
    }

    #[test]
    fn valq_type_custom_timeout_max_delivery_attempts() {
        let valq = ValqType::new(Some(3600), Some(10));
        assert_eq!(*valq.visibility_timeout(), 3600);
        assert_eq!(*valq.max_delivery_attempts(), 10);
    }

    #[test]
    fn valq_type_add_remove_msgs() {
        let mut valq = ValqType::new(None, None);
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
    fn valq_type_updates_id_sequence() {
        let mut valq = ValqType::new(None, None);
        valq.set_id_sequence(5);
        assert_eq!(*valq.id_sequence(), 5);
        valq.set_id_sequence(valq.id_sequence() + 1);
        assert_eq!(*valq.id_sequence(), 6);
    }

    #[test]
    fn valq_type_update_visibility_timeout_max_delivery_attempts() {
        let mut valq = ValqType::new(None, None);
        valq.set_visibility_timeout(7200);
        assert_eq!(*valq.visibility_timeout(), 7200);
        valq.set_max_delivery_attempts(10);
        assert_eq!(*valq.max_delivery_attempts(), 10);
    }
}
