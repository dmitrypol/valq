use crate::data_types::VALQ_TYPE;
use crate::structs::valq_type::ValqType;
use crate::{GLOBAL_Q_LIST, utils};
use valkey_module::ContextGuard;
use valkey_module::logging::log_notice;

pub(crate) fn run(ctx: &ContextGuard) {
    for q_string in get_all_queues() {
        log_notice(format!("retention_period_gc q: {}", q_string).as_str());
        let q_valkey_string = ctx.create_string(q_string);
        let q_key = ctx.open_key_writable(&q_valkey_string);
        let q_value = q_key.get_value::<ValqType>(&VALQ_TYPE).unwrap_or(None);
        handler(q_value);
    }
}

fn get_all_queues() -> Vec<String> {
    match GLOBAL_Q_LIST.read() {
        Ok(guard) => guard.iter().cloned().collect(),
        Err(_) => vec![],
    }
}

// loop through dlq_msgs and delete messages where timeout_at > now - RETENTION_PERIOD
fn handler(valq_type: Option<&mut ValqType>) {
    match valq_type {
        Some(tmp) => {
            let retention_period = tmp.retention_period().clone();
            let dlq_msgs = tmp.dlq_msgs_mut();
            let mut msgs_to_remove = vec![];
            for (index, msg) in dlq_msgs.iter().enumerate() {
                // check if msg is too old
                if msg.timeout_at().unwrap_or(0)
                    > utils::now_as_seconds().saturating_sub(retention_period)
                {
                    msgs_to_remove.push(index);
                }
            }
            // remove msgs in reverse order to avoid index shifting
            for index in msgs_to_remove.iter().rev() {
                dlq_msgs.remove(*index);
            }
        }
        None => {
            log_notice("q does not exist");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RETENTION_PERIOD_DEFAULT;
    use crate::structs::valq_msg::ValqMsg;

    #[test]
    fn test_get_all_queues() {
        assert!(get_all_queues().is_empty());
    }

    #[test]
    fn handler_empty_dlq() {
        let mut valq = ValqType::new("q", None, None, None).unwrap();
        handler(Some(&mut valq));
        assert!(valq.dlq_msgs().is_empty());
    }

    #[test]
    fn handler_removes_msg_after_retention_period() {
        let mut valq = ValqType::new("q", None, None, None).unwrap();
        let msg1 = ValqMsg::new(1, "m1".to_string(), Some(utils::now_as_seconds()), 1);
        let msg2 = ValqMsg::new(
            2,
            "m2".to_string(),
            Some(utils::now_as_seconds().saturating_sub(RETENTION_PERIOD_DEFAULT)),
            1,
        );
        valq.dlq_msgs_mut().push_back(msg1);
        valq.dlq_msgs_mut().push_back(msg2);
        assert_eq!(valq.dlq_msgs().len(), 2);
        handler(Some(&mut valq));
        assert_eq!(valq.dlq_msgs().len(), 1);
    }
}
