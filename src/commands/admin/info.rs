use crate::data_types::VALQ_TYPE;
use crate::structs::valq_type::ValqType;
use std::collections::BTreeMap;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

pub(crate) fn info(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 1 {
        return Err(ValkeyError::Str("specify q name"));
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let key = ctx.open_key(&key_arg);
    let value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    handler(value)
}

fn handler(value: Option<&ValqType>) -> ValkeyResult {
    match value {
        Some(tmp) => {
            let output = ValkeyValue::OrderedMap(BTreeMap::from([
                (
                    "visibility_timeout".into(),
                    tmp.visibility_timeout().to_string().into(),
                ),
                (
                    "max_delivery_attempts".into(),
                    tmp.max_delivery_attempts().to_string().into(),
                ),
                ("id_sequence".into(), tmp.id_sequence().to_string().into()),
                ("dlq_msgs".into(), tmp.dlq_msgs().len().to_string().into()),
                // TODO - exclude messages with timeout_at and max_delivery_attempts
                ("msgs".into(), tmp.msgs().len().to_string().into()),
                (
                    "delayed_msgs".into(),
                    tmp.delayed_msgs().len().to_string().into(),
                ),
            ]));
            Ok(output)
        }
        None => Err(ValkeyError::Str("q not found")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::valq_msg::ValqMsg;
    use valkey_module::ValkeyValue;

    #[test]
    fn test_with_nonexistent_queue() {
        let test = handler(None);
        assert!(test.is_err());
    }

    #[test]
    fn test_with_empty_queue() {
        let valq = ValqType::new("q", None, None).unwrap();
        let test = handler(Some(&valq));
        assert_eq!(
            test.unwrap(),
            ValkeyValue::OrderedMap(BTreeMap::from([
                ("delayed_msgs".into(), "0".into()),
                ("dlq_msgs".into(), "0".into()),
                ("id_sequence".into(), "0".into()),
                ("max_delivery_attempts".into(), "5".into()),
                ("msgs".into(), "0".into()),
                ("visibility_timeout".into(), "30".into()),
            ]))
        );
    }

    #[test]
    fn test_with_valid_queue() {
        let mut valq = ValqType::new("q", None, None).unwrap();
        valq.msgs_mut()
            .push_back(ValqMsg::new(1, "msg1".to_string(), None, 0));
        valq.msgs_mut()
            .push_back(ValqMsg::new(2, "msg2".to_string(), None, 0));
        valq.dlq_msgs_mut()
            .push_back(ValqMsg::new(3, "dlq_msg1".to_string(), None, 0));

        let test = handler(Some(&mut valq));
        assert_eq!(
            test.unwrap(),
            ValkeyValue::OrderedMap(BTreeMap::from([
                ("delayed_msgs".into(), "0".into()),
                ("dlq_msgs".into(), "1".into()),
                ("id_sequence".into(), "0".into()),
                ("max_delivery_attempts".into(), "5".into()),
                ("msgs".into(), "2".into()),
                ("visibility_timeout".into(), "30".into())
            ]))
        );
    }
}
