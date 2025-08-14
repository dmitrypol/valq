mod ack;
mod admin;
mod extend;
mod len;
mod pop;
mod push;

use valkey_module::{Context, NextArg, ValkeyResult, ValkeyString, ValkeyValue};

pub(crate) fn valq_cmd(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() == 1 {
        return help();
    }
    let mut args = args.into_iter().skip(1);
    let subcmd = args.next_string()?;
    let args: Vec<ValkeyString> = args.collect();
    match subcmd.to_lowercase().as_str() {
        "create" => admin::create(ctx, args),
        "delete" => admin::delete(ctx, args),
        "push" => push::push(ctx, args),
        "pop" => pop::pop(ctx, args),
        "ack" => ack::ack(ctx, args),
        "len" => len::len(ctx, args),
        "extend" => extend::extend(ctx, args),
        _ => help(),
    }
}

fn help() -> ValkeyResult {
    let output: Vec<ValkeyValue> = vec![
        "valq - top level command".into(),
        "valq create - crate new q".into(),
        "valq delete - delete q".into(),
        "valq push - push message to q".into(),
        "valq pop - get message from q".into(),
        "valq ack - ack message completion".into(),
        "valq extend - extend message to have more time to complete it".into(),
        "valq help - display this message".into(),
    ];
    Ok(output.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help() {
        let result = help();
        assert!(result.is_ok());
    }
}
