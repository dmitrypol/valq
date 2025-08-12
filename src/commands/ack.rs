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
    match value {
        Some(tmp) => {
            let data: &mut VecDeque<ValqMsg> = tmp.msgs_mut();
            // iterate through messages looking for the message with the given ID
            for (index, _msg) in data
                .iter_mut()
                .enumerate()
                .filter(|(_index, msg)| *msg.id() == msg_id_arg)
            {
                data.remove(index);
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
