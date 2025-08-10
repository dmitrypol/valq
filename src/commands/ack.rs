use crate::data_types::VALQ_TYPE;
use crate::structs::valq_type::ValqType;
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
            // try to remove the message from msgs_in_flight
            match tmp.msgs_in_flight_mut().remove(&msg_id_arg) {
                Some(_) => {
                    // ack message
                    Ok("ack".into())
                }
                None => {
                    // message not found in msgs_in_flight
                    Err(ValkeyError::Str("message not found in flight"))
                }
            }
        }
        None => Err(ValkeyError::Str("invalid queue")),
    }
}
