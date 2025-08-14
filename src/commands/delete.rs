use valkey_module::{Context, ValkeyError, ValkeyResult, ValkeyString};

pub(crate) fn delete(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 1 {
        return Err(ValkeyError::WrongArity);
    }
    let key = ctx.open_key_writable(&args[0]);
    match key.delete() {
        Ok(_) => Ok("deleted q".into()),
        Err(err) => Err(ValkeyError::String(format!("delete q failed: {}", err))),
    }
}
