use crate::data_types::VALQ_TYPE;
use crate::structs::{ValqMsg, ValqType};
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn push(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 2 {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let value_arg = args.next_string()?;
    let value = ctx
        .open_key_writable(&key_arg)
        .get_value::<ValqType>(&VALQ_TYPE)?;
    match value {
        Some(tmp) => {
            // increment id_sequence
            let id = tmp.id_sequence() + 1;
            tmp.set_id_sequence(id);
            // add new value to the queue
            tmp.msgs_mut().push_back(ValqMsg::new(id, value_arg));
            Ok(id.to_string().into())
        }
        None => Err(ValkeyError::Str("create the queue")),
    }
}
