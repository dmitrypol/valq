use crate::data_types::VALQ_TYPE;
use crate::structs::valq_msg::ValqMsg;
use crate::structs::valq_type::ValqType;
use std::collections::VecDeque;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn ack(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 2 {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let msg_id_arg = args.next_u64()?;
    let value = ctx
        .open_key_writable(&key_arg)
        .get_value::<ValqType>(&VALQ_TYPE)?;
    handler(msg_id_arg, value)
}

fn handler(msg_id_arg: u64, value: Option<&mut ValqType>) -> ValkeyResult {
    match value {
        Some(tmp) => {
            let msgs: &mut VecDeque<ValqMsg> = tmp.msgs_mut();
            // iterate through messages looking for the message with the given ID
            for (index, _msg) in msgs
                .iter_mut()
                .enumerate()
                .filter(|(_index, msg)| *msg.id() == msg_id_arg)
            {
                msgs.remove(index);
                return Ok("ack".into());
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
        let test = handler(1, None);
        assert!(test.is_err());
    }

    #[test]
    fn test_with_empty_queue() {
        let mut valq = ValqType::new(None, None).unwrap();
        let test = handler(1, Some(&mut valq));
        assert!(test.is_err());
    }

    #[test]
    fn test_with_valid_queue() {
        let mut valq = ValqType::new(None, None).unwrap();
        valq.msgs_mut()
            .push_back(ValqMsg::new(1, "msg1".to_string(), None, 0));
        valq.msgs_mut()
            .push_back(ValqMsg::new(2, "msg2".to_string(), None, 0));

        let test = handler(1, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::BulkString("ack".to_string()));
        assert_eq!(valq.msgs_mut().len(), 1);
        assert_eq!(valq.dlq_msgs_mut().len(), 0);

        // invalid message ID
        let test = handler(3, Some(&mut valq));
        assert!(test.is_err());
    }

    #[test]
    fn test_large_number_of_messages() {
        let mut valq = ValqType::new(None, None).unwrap();
        for i in 1..=10_000 {
            valq.msgs_mut()
                .push_back(ValqMsg::new(i, format!("msg{}", i), None, 0));
        }
        let test = handler(5_000, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::BulkString("ack".to_string()));
        let msg = valq.msgs_mut().iter().find(|msg| *msg.id() == 5_000);
        assert!(msg.is_none());
    }
}
