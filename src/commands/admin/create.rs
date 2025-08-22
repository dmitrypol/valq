use crate::data_types::VALQ_TYPE;
use crate::structs::valq_type::ValqType;
use crate::utils::replicate_cmd_check;
use crate::{
    DELIVERY_ATTEMPTS_DEFAULT, GLOBAL_Q_LIST, RETENTION_PERIOD_DEFAULT, VISIBILITY_TIMEOUT_DEFAULT,
};
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn create(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    replicate_cmd_check(ctx)?;
    if args.is_empty() {
        return Err(ValkeyError::Str(
            "specify q name and optional visibility timeout, max delivery attempts, retention period",
        ));
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let visibility_timeout_arg = args.next_u64().unwrap_or(VISIBILITY_TIMEOUT_DEFAULT);
    let max_delivery_attempts_arg = args.next_u64().unwrap_or(DELIVERY_ATTEMPTS_DEFAULT);
    let retention_period_arg = args.next_u64().unwrap_or(RETENTION_PERIOD_DEFAULT);
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
                key_arg.to_string().as_str(),
                Some(visibility_timeout_arg),
                Some(max_delivery_attempts_arg),
                Some(retention_period_arg),
            )?;
            key.set_value(&VALQ_TYPE, valq)?;
            let mut q_list = GLOBAL_Q_LIST.write()?;
            q_list.insert(key_arg.to_string());
            Ok("created q".into())
        }
    }
}
