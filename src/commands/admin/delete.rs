use crate::utils::replicate_cmd_check;
use valkey_module::{Context, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn delete(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    replicate_cmd_check(ctx)?;
    if args.len() != 1 {
        return Err(ValkeyError::Str("specify q name"));
    }
    let key = ctx.open_key_writable(&args[0]);
    match key.delete() {
        Ok(_) => Ok("deleted q".into()),
        Err(err) => Err(ValkeyError::String(format!("delete q failed: {}", err))),
    }
}
