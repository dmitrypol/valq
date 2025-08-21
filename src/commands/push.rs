use crate::data_types::VALQ_TYPE;
use crate::structs::valq_msg::ValqMsg;
use crate::structs::valq_type::ValqType;
use crate::utils;
use crate::utils::replicate_cmd_check;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn push(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    replicate_cmd_check(ctx)?;
    if args.len() != 2 && args.len() != 3 {
        return Err(ValkeyError::Str(
            "specify q name, message and optional delay",
        ));
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let value_arg = args.next_string()?;
    let delay_arg = args.next_u64().unwrap_or(0);
    let value = ctx
        .open_key_writable(&key_arg)
        .get_value::<ValqType>(&VALQ_TYPE)?;
    handler(value_arg, delay_arg, value)
}

fn handler(value_arg: String, delay_arg: u64, value: Option<&mut ValqType>) -> ValkeyResult {
    match value {
        Some(tmp) => {
            // increment id_sequence
            let id = tmp.id_sequence() + 1;
            tmp.set_id_sequence(id);
            let msg = ValqMsg::new(id, value_arg, None, 0);
            if delay_arg == 0 {
                // add new value to the queue
                tmp.msgs_mut().push_back(msg);
            } else {
                // add new value to the delayed messages
                tmp.delayed_msgs_mut()
                    .insert(msg, utils::now_as_seconds().saturating_add(delay_arg));
            }
            Ok(id.to_string().into())
        }
        None => Err(ValkeyError::Str("create the queue")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use valkey_module::ValkeyValue;

    #[test]
    fn test_with_nonexistent_queue() {
        let test = handler("msg1".to_string(), 0, None);
        assert!(test.is_err());
    }

    #[test]
    fn test_with_valid_queue() {
        let mut valq = ValqType::new(None, None).unwrap();
        let test = handler("msg1".to_string(), 0, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::BulkString("1".to_string()));
        let test = handler("msg2".to_string(), 0, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::BulkString("2".to_string()));
    }

    #[test]
    fn test_large_number_of_messages() {
        let mut valq = ValqType::new(None, None).unwrap();
        for i in 1..=10_000 {
            let test = handler(format!("msg{}", i), 0, Some(&mut valq));
            assert!(test.is_ok());
            assert_eq!(test.unwrap(), ValkeyValue::BulkString(i.to_string()));
        }
        assert_eq!(valq.msgs_mut().len(), 10_000);
    }

    #[test]
    fn test_with_delayed_message() {
        let mut valq = ValqType::new(None, None).unwrap();
        let test = handler("delayed_msg".to_string(), 1, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::BulkString("1".to_string()));
        assert_eq!(valq.delayed_msgs().len(), 1);
        assert_eq!(valq.msgs().len(), 0);
    }
}
