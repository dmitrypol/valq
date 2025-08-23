use crate::data_types::VALQ_TYPE;
use crate::structs::valq_type::ValqType;
use crate::utils::replicate_cmd_check;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn update(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    replicate_cmd_check(ctx)?;
    if args.len() != 4 {
        return Err(ValkeyError::Str(
            "specify q name, visibility timeout, max delivery attempts and retention period",
        ));
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let visibility_timeout_arg = args.next_u64()?;
    let max_delivery_attempts_arg = args.next_u64()?;
    let retention_period_arg = args.next_u64()?;
    let key = ctx.open_key_writable(&key_arg);
    let value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    match value {
        Some(tmp) => {
            // update existing queue
            tmp.set_visibility_timeout(visibility_timeout_arg)?;
            tmp.set_max_delivery_attempts(max_delivery_attempts_arg)?;
            tmp.set_retention_period(retention_period_arg)?;
            Ok("updated q".into())
        }
        None => Err(ValkeyError::Str("q does not exist")),
    }
}
