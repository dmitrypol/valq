use crate::data_types::VALQ_TYPE;
use crate::structs::valq_type::ValqType;
use crate::{DELIVERY_ATTEMPTS_DEFAULT, VISIBILITY_TIMEOUT_DEFAULT};
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn create(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.is_empty() {
        return Err(ValkeyError::Str(
            "specify q name, visibility timeout and max delivery attempts",
        ));
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let visibility_timeout_arg = args.next_u64().unwrap_or(VISIBILITY_TIMEOUT_DEFAULT);
    let max_delivery_attempts_arg = args.next_u64().unwrap_or(DELIVERY_ATTEMPTS_DEFAULT);
    let key = ctx.open_key_writable(&key_arg);
    let value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    match value {
        Some(_) => {
            // queue already exists
            Err(ValkeyError::Str("q exists"))
        }
        None => {
            // create a new queue
            let valq = ValqType::new(
                Some(visibility_timeout_arg),
                Some(max_delivery_attempts_arg),
            )?;
            key.set_value(&VALQ_TYPE, valq)?;
            Ok("created q".into())
        }
    }
}
