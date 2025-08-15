use crate::data_types::VALQ_TYPE;
use crate::structs::valq_type::ValqType;
use crate::{
    DELIVERY_ATTEMPTS_DEFAULT, DELIVERY_ATTEMPTS_MAX, VISIBILITY_TIMEOUT_DEFAULT,
    VISIBILITY_TIMEOUT_MAX,
};
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn create(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.is_empty() {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    // TODO - move validations to ValqType::new, DRY up create and update
    let visibility_timeout_arg = args.next_u64().unwrap_or(VISIBILITY_TIMEOUT_DEFAULT);
    if visibility_timeout_arg < 1 || visibility_timeout_arg > VISIBILITY_TIMEOUT_MAX {
        return Err(ValkeyError::Str(
            "timeout must be between 1 and 43_200 seconds (12 hours)",
        ));
    }
    let max_delivery_attempts_arg = args.next_u64().unwrap_or(DELIVERY_ATTEMPTS_DEFAULT);
    if max_delivery_attempts_arg < 1 || max_delivery_attempts_arg > DELIVERY_ATTEMPTS_MAX {
        return Err(ValkeyError::String(format!(
            "max delivery attempts must be between 1 and {}",
            DELIVERY_ATTEMPTS_MAX
        )));
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
            key.set_value(
                &VALQ_TYPE,
                ValqType::new(
                    Some(visibility_timeout_arg),
                    Some(max_delivery_attempts_arg),
                ),
            )?;
            Ok("created q".into())
        }
    }
}
