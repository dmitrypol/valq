use crate::data_types::VALQ_TYPE;
use crate::structs::valq_msg::ValqMsg;
use crate::structs::valq_type::ValqType;
use crate::utils;
use std::collections::VecDeque;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn pop(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.is_empty() {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let value = ctx
        .open_key_writable(&key_arg)
        .get_value::<ValqType>(&VALQ_TYPE)?;
    match value {
        Some(tmp) => {
            let visibility_timeout = *tmp.visibility_timeout();
            let data: &mut VecDeque<ValqMsg> = tmp.msgs_mut();
            // iterate through messages and find the first one that is visible
            for msg in data.iter_mut().filter(|msg| msg.is_visible()) {
                // set timeout_at and return the message
                msg.set_timeout_at(Some(
                    utils::now_as_seconds().saturating_add(visibility_timeout),
                ));
                return Ok(msg.clone().into());
            }
            // all messages have timeout_at, return nothing
            Ok("".into())
        }
        None => Err(ValkeyError::Str("invalid queue")),
    }
}
