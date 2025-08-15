use crate::data_types::VALQ_TYPE;
use crate::structs::valq_type::ValqType;
use crate::{DELIVERY_ATTEMPTS_MAX, VISIBILITY_TIMEOUT_MAX};
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn update(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 3 {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    // TODO - move validations to ValqType::new, DRY up create and update
    let visibility_timeout_arg = args.next_u64()?;
    if visibility_timeout_arg < 1 || visibility_timeout_arg > VISIBILITY_TIMEOUT_MAX {
        return Err(ValkeyError::Str(
            "timeout must be between 1 and 43_200 seconds (12 hours)",
        ));
    }
    let max_delivery_attempts_arg = args.next_u64()?;
    if max_delivery_attempts_arg < 1 || max_delivery_attempts_arg > DELIVERY_ATTEMPTS_MAX {
        return Err(ValkeyError::String(format!(
            "max delivery attempts must be between 1 and {}",
            DELIVERY_ATTEMPTS_MAX
        )));
    }
    let key = ctx.open_key_writable(&key_arg);
    let value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    match value {
        Some(tmp) => {
            // update existing queue
            tmp.set_max_delivery_attempts(max_delivery_attempts_arg);
            tmp.set_visibility_timeout(visibility_timeout_arg);
            Ok("updated q".into())
        }
        None => Err(ValkeyError::Str("q does not exist")),
    }
}
