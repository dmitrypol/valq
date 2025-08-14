use crate::data_types::VALQ_TYPE;
use crate::structs::valq_type::ValqType;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn create(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.is_empty() {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    // TODO - move validation to ValqType::new
    let visibility_timeout_arg = args.next_u64().unwrap_or(crate::VISIBILITY_TIMEOUT_DEFAULT);
    if visibility_timeout_arg < 1 || visibility_timeout_arg > crate::VISIBILITY_TIMEOUT_MAX {
        return Err(ValkeyError::Str(
            "timeout must be between 1 and 43_200 seconds (12 hours)",
        ));
    }
    let key = ctx.open_key_writable(&key_arg);
    let value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    match value {
        Some(_) => {
            // queue already exists
            Err(ValkeyError::Str("q exists"))
        }
        None => {
            // create a new queue
            key.set_value(&VALQ_TYPE, ValqType::new(Some(visibility_timeout_arg)))?;
            Ok("created q".into())
        }
    }
}
