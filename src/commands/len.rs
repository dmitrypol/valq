use crate::data_types::VALQ_TYPE;
use crate::structs::valq_type::ValqType;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn len(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 1 {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let key = ctx.open_key(&key_arg);
    let current_value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    // TODO - exclude messages with timeout_at
    match current_value {
        Some(tmp) => Ok(tmp.msgs().len().into()),
        None => Ok("0".into()),
    }
}
