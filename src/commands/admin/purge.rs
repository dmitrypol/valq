use crate::data_types::VALQ_TYPE;
use crate::structs::q_type::QType;
use crate::structs::valq_type::ValqType;
use crate::utils::replicate_cmd_check;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn purge(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    replicate_cmd_check(ctx)?;
    if args.len() != 1 && args.len() != 2 {
        return Err(ValkeyError::Str(
            "specify q name and optionally q type (main, dlq or delayed)",
        ));
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let q_type = QType::from_str(args.next_str().unwrap_or("main"));
    let key = ctx.open_key_writable(&key_arg);
    let value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    handler(q_type, value)
}

fn handler(q_type: QType, value: Option<&mut ValqType>) -> ValkeyResult {
    match value {
        Some(tmp) => match q_type {
            QType::Main => {
                let msgs_count = tmp.msgs().len();
                tmp.msgs_mut().clear();
                Ok(msgs_count.into())
            }
            QType::Dlq => {
                let msgs_count = tmp.dlq_msgs().len();
                tmp.dlq_msgs_mut().clear();
                Ok(msgs_count.into())
            }
            QType::Delayed => {
                let msgs_count = tmp.delayed_msgs().len() as usize;
                tmp.delayed_msgs_mut().clear();
                Ok(msgs_count.into())
            }
        },
        None => Err(ValkeyError::Str("q not found")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::valq_msg::ValqMsg;
    use crate::structs::valq_type::ValqType;
    use valkey_module::ValkeyValue;

    #[test]
    fn test_with_nonexistent_queue() {
        let test = handler(QType::Main, None);
        assert!(test.is_err());
        let test = handler(QType::Dlq, None);
        assert!(test.is_err());
        let test = handler(QType::Delayed, None);
        assert!(test.is_err());
    }

    #[test]
    fn test_with_empty_queue() {
        let mut valq = ValqType::new("q", None, None, None).unwrap();
        let test = handler(QType::Main, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::Integer(0));
        let test = handler(QType::Dlq, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::Integer(0));
        let test = handler(QType::Delayed, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::Integer(0));
    }

    #[test]
    fn test_with_valid_queue() {
        let mut valq = ValqType::new("q", None, None, None).unwrap();
        let msg1 = ValqMsg::new(1, "msg".to_string(), None, 0);
        valq.msgs_mut().push_back(msg1);
        let msg2 = ValqMsg::new(2, "msg".to_string(), None, 0);
        valq.msgs_mut().push_back(msg2);
        let dlq_msg = ValqMsg::new(3, "msg".to_string(), None, 5);
        valq.dlq_msgs_mut().push_back(dlq_msg);
        let delayed_msg = ValqMsg::new(4, "msg".to_string(), None, 0);
        valq.delayed_msgs_mut().insert(delayed_msg, 1);

        let test = handler(QType::Main, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::Integer(2));
        let test = handler(QType::Dlq, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::Integer(1));
        let test = handler(QType::Delayed, Some(&mut valq));
        assert_eq!(test.unwrap(), ValkeyValue::Integer(1));
    }
}
