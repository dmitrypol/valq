use crate::data_types::VALQ_TYPE;
use crate::structs::{ValqMsg, ValqType};
use std::collections::VecDeque;
use valkey_module::{Context, NextArg, ValkeyError, ValkeyResult, ValkeyString, ValkeyValue};

pub(crate) fn valq_cmd(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() == 1 {
        return help();
    }
    let mut args = args.into_iter().skip(1);
    let subcmd = args.next_string()?;
    let args: Vec<ValkeyString> = args.collect();
    match subcmd.to_lowercase().as_str() {
        "create" => create(ctx, args),
        "delete" => delete(ctx, args),
        "push" => push(ctx, args),
        "pop" => pop(ctx, args),
        "ack" => ack(ctx, args),
        "len" => len(ctx, args),
        _ => help(),
    }
}

fn create(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 1 {
        return Err(ValkeyError::WrongArity);
    }
    let key = ctx.open_key_writable(&args[0]);
    let value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    match value {
        Some(_) => {
            // queue already exists
            Err(ValkeyError::Str("q exists"))
        }
        None => {
            // create a new queue
            key.set_value(&VALQ_TYPE, ValqType::new())?;
            Ok("created q".into())
        }
    }
}

fn delete(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 1 {
        return Err(ValkeyError::WrongArity);
    }
    let key = ctx.open_key_writable(&args[0]);
    match key.delete() {
        Ok(_) => Ok("deleted q".into()),
        Err(err) => Err(ValkeyError::String(format!("delete q failed: {}", err))),
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
            // increment id_sequence
            let id = tmp.id_sequence() + 1;
            tmp.set_id_sequence(id);
            // add new value to the queue
            tmp.msgs_mut().push_back(ValqMsg::new(id, value_arg));
            Ok(id.to_string().into())
        }
        None => Err(ValkeyError::Str("create the queue")),
    }
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
            let data: &mut VecDeque<ValqMsg> = tmp.msgs_mut();
            match data.pop_front() {
                Some(msg) => {
                    // store the message to msgs_in_flight
                    tmp.msgs_in_flight_mut()
                        .insert(*msg.id(), msg.body().clone());
                    // return the message body
                    Ok(msg.body().into())
                }
                None => {
                    // queue is empty
                    Ok("".into())
                }
            }
        }
        None => Err(ValkeyError::Str("invalid queue")),
    }
}

fn ack(ctx: &Context, args: Vec<ValkeyString>) -> ValkeyResult {
    if args.len() != 2 {
        return Err(ValkeyError::WrongArity);
    }
    let mut args = args.into_iter();
    let key_arg = args.next_arg()?;
    let msg_id_arg = args.next_u64()?;
    let key = ctx.open_key_writable(&key_arg);
    let current_value = key.get_value::<ValqType>(&VALQ_TYPE)?;
    match current_value {
        Some(tmp) => {
            // try to remove the message from msgs_in_flight
            match tmp.msgs_in_flight_mut().remove(&msg_id_arg) {
                Some(_) => {
                    // ack message
                    Ok("ack".into())
                }
                None => {
                    // message not found in msgs_in_flight
                    Err(ValkeyError::Str("message not found in flight"))
                }
            }
        }
        None => Err(ValkeyError::Str("invalid queue")),
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
        Some(tmp) => Ok(tmp.msgs().len().into()),
        None => Ok("0".into()),
    }
}

fn help() -> ValkeyResult {
    let output: Vec<ValkeyValue> = vec![
        "valq - top level command".into(),
        "valq create - crate new q".into(),
        "valq delete - delete q".into(),
        "valq push - push message to q".into(),
        "valq pop - get message from q".into(),
        "valq help - display this message".into(),
    ];
    Ok(output.into())
}
