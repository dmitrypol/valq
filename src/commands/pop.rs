use crate::data_types::VALQ_TYPE;
use crate::structs::valq_msg::ValqMsg;
use crate::structs::valq_type::ValqType;
use crate::utils;
use crate::utils::replicate_cmd_check;
use std::collections::VecDeque;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn pop(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    replicate_cmd_check(ctx)?;
    if args.is_empty() {
        return Err(ValkeyError::Str("specify q name"));
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let value = ctx
        .open_key_writable(&key_arg)
        .get_value::<ValqType>(&VALQ_TYPE)?;
    handler(value)
}

fn handler(value: Option<&mut ValqType>) -> ValkeyResult {
    match value {
        Some(tmp) => {
            move_delayed_msgs_to_main_q(tmp);
            let max_delivery_attempts_msgs = match process_main_q(tmp) {
                Ok(value) => value,
                Err(value) => return value,
            };
            move_max_delivery_msgs_to_dlq(tmp, &max_delivery_attempts_msgs);
            // all messages have timeout_at, return nothing
            Ok("".into())
        }
        None => Err(ValkeyError::Str("invalid queue")),
    }
}

fn move_delayed_msgs_to_main_q(valq: &mut ValqType) {
    let binding = valq.delayed_msgs().clone();
    let delayed_msgs_to_process: Vec<_> = binding
        .ready_to_process()
        .into_iter()
        .map(|(_, msg)| msg)
        .collect();
    for msg in delayed_msgs_to_process {
        // remove from delayed_msgs and add to msgs
        valq.delayed_msgs_mut().remove(&msg);
        // push to the front of msgs to process delayed messages first
        valq.msgs_mut().push_front(msg.clone());
    }
}

fn process_main_q(tmp: &mut ValqType) -> Result<Vec<(usize, ValqMsg)>, ValkeyResult> {
    let visibility_timeout = *tmp.visibility_timeout();
    let max_delivery_attempts = *tmp.max_delivery_attempts();
    let msgs: &mut VecDeque<ValqMsg> = tmp.msgs_mut();
    let mut max_delivery_attempts_msgs = Vec::new();
    // iterate through messages and find the first one that is visible
    for (index, msg) in msgs
        .iter_mut()
        .enumerate()
        .filter(|(_index, msg)| msg.check_timeout_at())
    {
        if !msg.check_max_delivery_attempts(max_delivery_attempts) {
            max_delivery_attempts_msgs.push((index, msg.clone()));
            continue; // skip this message
        }
        // set timeout_at
        msg.set_timeout_at(Some(
            utils::now_as_seconds().saturating_add(visibility_timeout),
        ));
        // increment delivery_attempts
        msg.set_delivery_attempts(msg.delivery_attempts() + 1);
        // return the message
        return Err(Ok(msg.clone().into()));
    }
    Ok(max_delivery_attempts_msgs)
}

fn move_max_delivery_msgs_to_dlq(
    valq: &mut ValqType,
    max_delivery_attempts_msgs: &Vec<(usize, ValqMsg)>,
) {
    // remove from msgs and add to dlq_msgs
    for (index, msg) in max_delivery_attempts_msgs {
        valq.msgs_mut().remove(*index);
        valq.dlq_msgs_mut().push_back(msg.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use valkey_module::ValkeyValue;

    #[test]
    fn test_with_nonexistent_queue() {
        let test = handler(None);
        assert!(test.is_err());
    }

    #[test]
    fn test_with_empty_queue_returns_nothing() {
        let mut valq = ValqType::new(None, None).unwrap();
        let test = handler(Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::BulkString("".to_string()));
        assert!(valq.msgs().is_empty());
        assert!(valq.dlq_msgs().is_empty());
    }

    #[test]
    fn test_with_no_visible_message_in_queue() {
        let mut valq = ValqType::new(None, None).unwrap();
        let msg = ValqMsg::new(1, "msg".to_string(), Some(utils::now_as_seconds() + 10), 0);
        valq.msgs_mut().push_back(msg);
        let test = handler(Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::BulkString("".to_string()));
    }

    #[test]
    fn test_with_delivery_attempts_exceeded() {
        let mut valq = ValqType::new(None, None).unwrap();
        let msg = ValqMsg::new(1, "msg".to_string(), Some(utils::now_as_seconds()), 5);
        valq.msgs_mut().push_back(msg);
        let test = handler(Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::BulkString("".to_string()));
        assert_eq!(valq.dlq_msgs().len(), 1);
    }

    #[test]
    fn test_with_visible_message_in_queue() {
        let mut valq = ValqType::new(None, None).unwrap();
        let msg = ValqMsg::new(1, "msg".to_string(), Some(utils::now_as_seconds()), 0);
        valq.msgs_mut().push_back(msg);
        let test = handler(Some(&mut valq));
        assert!(test.is_ok());
        assert!(valq.dlq_msgs().is_empty());
    }

    #[test]
    fn test_move_message_to_dlq_when_delivery_attempts_exceeded() {
        let mut valq = ValqType::new(None, None).unwrap();
        let msg = ValqMsg::new(1, "msg".to_string(), Some(utils::now_as_seconds()), 5);
        valq.msgs_mut().push_back(msg);

        let test = handler(Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::BulkString("".to_string()));
        assert!(valq.msgs().is_empty());
        assert_eq!(valq.dlq_msgs().len(), 1);
        assert_eq!(valq.dlq_msgs()[0].id(), &1);
    }

    #[test]
    fn test_move_delayed_msgs_to_main_q_moves_ready_messages() {
        let mut valq = ValqType::new(None, None).unwrap();
        let msg1 = ValqMsg::new(1, "msg1".to_string(), None, 0);
        let msg2 = ValqMsg::new(2, "msg2".to_string(), None, 0);
        valq.delayed_msgs_mut()
            .insert(msg1.clone(), utils::now_as_seconds() - 1);
        valq.delayed_msgs_mut()
            .insert(msg2.clone(), utils::now_as_seconds());

        let _ = handler(Some(&mut valq));
        assert_eq!(valq.delayed_msgs().len(), 0);
        assert_eq!(valq.msgs().len(), 2);
        assert_eq!(*valq.msgs()[0].id(), 2);
        assert_eq!(*valq.msgs()[1].id(), 1);
    }

    #[test]
    fn test_move_delayed_msgs_to_main_q_handles_empty_delayed_msgs() {
        let mut valq = ValqType::new(None, None).unwrap();
        let _ = handler(Some(&mut valq));
        assert_eq!(valq.delayed_msgs().len(), 0);
        assert!(valq.msgs().is_empty());
    }

    #[test]
    fn test_move_delayed_msgs_to_main_q_does_not_move_non_ready_messages() {
        let mut valq = ValqType::new(None, None).unwrap();
        let msg = ValqMsg::new(1, "msg".to_string(), None, 0);
        valq.delayed_msgs_mut()
            .insert(msg.clone(), utils::now_as_seconds() + 10);

        let _ = handler(Some(&mut valq));
        assert_eq!(valq.delayed_msgs().len(), 1);
        assert!(valq.msgs().is_empty());
    }
}
