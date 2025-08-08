use crate::data_types::VALQ_TYPE;
use crate::structs::{ValqMsg, ValqType};
use std::collections::VecDeque;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn pop(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 1 {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let value = ctx
        .open_key_writable(&key_arg)
        .get_value::<ValqType>(&VALQ_TYPE)?;
    match value {
        Some(tmp) => {
            let data: &mut VecDeque<ValqMsg> = tmp.msgs_mut();
            match data.pop_front() {
                Some(msg) => {
                    // store the message to msgs_in_flight
                    tmp.msgs_in_flight_mut()
                        .insert(*msg.id(), msg.body().clone());
                    // return the message body
                    Ok(msg.body().into())
                }
                None => {
                    // queue is empty
                    Ok("".into())
                }
            }
        }
        None => Err(ValkeyError::Str("invalid queue")),
    }
}
