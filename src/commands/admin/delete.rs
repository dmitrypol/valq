use crate::GLOBAL_Q_LIST;
use crate::utils::replicate_cmd_check;
use valkey_module::{Context, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn delete(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    replicate_cmd_check(ctx)?;
    if args.len() != 1 {
        return Err(ValkeyError::Str("specify q name"));
    }
    let key_arg = &args[0];
    let key = ctx.open_key_writable(key_arg);
    match key.delete() {
        Ok(_) => {
            let mut q_list = GLOBAL_Q_LIST.write()?;
            q_list.remove(&key_arg.to_string());
            Ok("deleted q".into())
        }
        Err(err) => Err(ValkeyError::String(format!("delete q failed: {}", err))),
    }
}
