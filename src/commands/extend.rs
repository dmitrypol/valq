use crate::data_types::VALQ_TYPE;
use crate::structs::valq_msg::ValqMsg;
use crate::structs::valq_type::ValqType;
use crate::utils;
use std::collections::VecDeque;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn extend(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 3 {
        return Err(ValkeyError::WrongArity);
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
    match value {
        Some(tmp) => {
            let data: &mut VecDeque<ValqMsg> = tmp.msgs_mut();
            // iterate through messages looking for the message with the given ID
            for msg in data.iter_mut().filter(|msg| *msg.id() == msg_id_arg) {
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
