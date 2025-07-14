use crate::data_types::{VALQ_TYPE, ValqType};
use std::collections::VecDeque;
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
        "len" => len(ctx, args),
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
    // look up current value
    let current_value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    match current_value {
        Some(tmp) => {
            // add new value to the queue
            tmp.data_mut().push_back(value_arg);
        }
        None => {
            // create new queue with the value
            let mut value = VecDeque::new();
            value.push_back(value_arg);
            key.set_value(&VALQ_TYPE, ValqType::new(value))?;
        }
    }
    Ok("OK".into())
}

fn pop(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 1 {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let key = ctx.open_key_writable(&key_arg);
    let current_value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    match current_value {
        Some(tmp) => {
            let data: &mut VecDeque<String> = tmp.data_mut();
            let value = data.pop_front().unwrap_or_default();
            Ok(value.into())
        }
        None => Ok("".into()),
    }
}

fn len(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 1 {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let key = ctx.open_key(&key_arg);
    let current_value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    match current_value {
        Some(tmp) => Ok(tmp.data().len().into()),
        None => Ok("0".into()),
    }
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
