use crate::data_types::VALQ_TYPE;
use crate::structs::valq_msg::ValqMsg;
use crate::structs::valq_type::ValqType;
use crate::utils;
use crate::utils::replicate_cmd_check;
use std::collections::VecDeque;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn extend(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    replicate_cmd_check(ctx)?;
    if args.len() != 3 {
        return Err(ValkeyError::Str("specify q name and message ID"));
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let msg_id_arg = args.next_u64()?;
    let extend_seconds_arg = args.next_u64()?;
    if extend_seconds_arg > crate::VISIBILITY_TIMEOUT_MAX {
        return Err(ValkeyError::Str(
            "extend timeout must be less than or equal to 43_200 seconds (12 hours)",
        ));
    }
    let value = ctx
        .open_key_writable(&key_arg)
        .get_value::<ValqType>(&VALQ_TYPE)?;
    handler(msg_id_arg, extend_seconds_arg, value)
}

fn handler(msg_id_arg: u64, extend_seconds_arg: u64, value: Option<&mut ValqType>) -> ValkeyResult {
    match value {
        Some(tmp) => {
            let msgs: &mut VecDeque<ValqMsg> = tmp.msgs_mut();
            // iterate through messages looking for the message with the given ID
            for msg in msgs.iter_mut().filter(|msg| *msg.id() == msg_id_arg) {
                // update timeout_at
                msg.set_timeout_at(Some(
                    utils::now_as_seconds().saturating_add(extend_seconds_arg),
                ));
                return Ok("extend".into());
            }
            Err(ValkeyError::String(format!(
                "message not found with id {}",
                msg_id_arg
            )))
        }
        None => Err(ValkeyError::Str("invalid queue")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use valkey_module::ValkeyValue;

    #[test]
    fn test_with_nonexistent_queue() {
        let test = handler(1, 10, None);
        assert!(test.is_err());
    }

    #[test]
    fn test_with_empty_queue() {
        let mut valq = ValqType::new(None, None).unwrap();
        let test = handler(1, 10, Some(&mut valq));
        assert!(test.is_err());
    }

    #[test]
    fn test_with_valid_queue() {
        let mut valq = ValqType::new(None, None).unwrap();
        valq.msgs_mut()
            .push_back(ValqMsg::new(1, "msg1".to_string(), None, 0));
        valq.msgs_mut()
            .push_back(ValqMsg::new(2, "msg2".to_string(), None, 0));
        let test = handler(1, 10, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::BulkString("extend".to_string()));
        assert_eq!(valq.msgs_mut().len(), 2);
        assert_eq!(valq.dlq_msgs_mut().len(), 0);
        // check if the timeout_at is updated
        let msg = valq.msgs_mut().get(0).unwrap();
        assert!(msg.timeout_at().unwrap() > utils::now_as_seconds());

        // invalid message ID
        let test = handler(3, 10, Some(&mut valq));
        assert!(test.is_err());
    }

    #[test]
    fn test_large_number_of_messages() {
        let mut valq = ValqType::new(None, None).unwrap();
        for i in 1..=10_000 {
            valq.msgs_mut()
                .push_back(ValqMsg::new(i, format!("msg{}", i), None, 0));
        }
        let test = handler(5_000, 30, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::BulkString("extend".to_string()));
        let msg = valq
            .msgs_mut()
            .iter()
            .find(|msg| *msg.id() == 5_000)
            .unwrap();
        assert!(msg.timeout_at().unwrap() > utils::now_as_seconds());
    }
}
