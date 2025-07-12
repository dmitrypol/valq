use crate::data_types::{VALQ_TYPE, ValqType};
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

pub(crate) fn valq_cmd(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() == 1 {
        return help();
    };
    let mut args = args.into_iter().skip(1);
    let subcmd = args.next_string()?;
    let args: Vec<ValkeyString> = args.collect();
    match subcmd.to_lowercase().as_str() {
        "push" => push(ctx, args),
        "pop" => pop(ctx, args),
        "help" => help(),
        _ => help(),
    }
}

fn push(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 2 {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let value_arg = args.next_string()?;
    let key = ctx.open_key_writable(&key_arg);
    let value = ValqType::new(value_arg.into());
    let resp = key.set_value(&VALQ_TYPE, value)?;
    Ok(resp.into())
}

fn pop(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 1 {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let key = ctx.open_key(&key_arg);
    let value = match key.get_value::<ValqType>(&VALQ_TYPE)? {
        Some(tmp) => tmp.data().into(),
        None => "".into(),
    };
    Ok(value)
}

fn help() -> ValkeyResult {
    let output: Vec<ValkeyValue> = vec![
        "valq - top level command".into(),
        "valq push - push message to q".into(),
        "valq pop - get message from q".into(),
        "valq help - display this message".into(),
    ];
    Ok(output.into())
}
