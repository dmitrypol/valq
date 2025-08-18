use crate::structs::valq_msg::ValqMsg;
use crate::structs::valq_type::ValqType;
use std::os::raw::c_void;
use valkey_module::{RedisModuleIO, load_string, load_unsigned, logging::log_notice};

/// Loads the state of a `ValqType` instance from the Valkey database.
///
/// This function is called by the Valkey module to restore the state of a `ValqType`
/// instance. It deserializes the fields of the `ValqType` and its associated messages
/// in the same order they were saved to ensure compatibility with the corresponding save function.
///
/// # Arguments
/// * `rdb` - A pointer to the RedisModuleIO structure used for loading data.
/// * `_encver` - The encoding version of the data being loaded.
///
/// # Returns
/// * A pointer to the newly created `ValqType` instance if successful.
/// * A null pointer if an error occurs during the loading process.
///
/// # Safety
/// This function uses unsafe code to dereference raw pointers. It ensures that
/// the pointers are not null before accessing the data.
pub(crate) extern "C" fn rdb_load(rdb: *mut RedisModuleIO, _encver: i32) -> *mut c_void {
    if rdb.is_null() {
        return std::ptr::null_mut();
    }
    let mut valq = ValqType::new(None, None).unwrap_or_default();

    for loader in [
        load_valq_attributes,
        load_msgs_attributes,
        load_dlq_msgs_attributes,
        load_delayed_msgs_attributes,
    ] {
        match loader(rdb, &mut valq) {
            Some(_) => {
                return std::ptr::null_mut();
            }
            None => continue,
        }
    }

    log_notice(format!("rdb_load: {:?}", valq));
    Box::into_raw(Box::new(valq)) as *mut c_void
}

fn load_valq_attributes(rdb: *mut RedisModuleIO, valq: &mut ValqType) -> Option<*mut c_void> {
    valq.set_id_sequence(load_unsigned(rdb).ok()?);

    let visibility_timeout = load_unsigned(rdb).ok()?;
    valq.set_visibility_timeout(visibility_timeout).ok()?;

    let max_delivery_attempts = load_unsigned(rdb).ok()?;
    valq.set_max_delivery_attempts(max_delivery_attempts).ok()?;

    None
}

fn load_msgs_attributes(rdb: *mut RedisModuleIO, valq: &mut ValqType) -> Option<*mut c_void> {
    let msgs_size = load_unsigned(rdb).unwrap_or(0) as usize;
    for _ in 0..msgs_size {
        match load_each_msg(rdb) {
            Some(msg) => {
                valq.msgs_mut().push_back(msg);
            }
            None => {
                return Some(std::ptr::null_mut());
            }
        }
    }
    None
}

fn load_dlq_msgs_attributes(rdb: *mut RedisModuleIO, valq: &mut ValqType) -> Option<*mut c_void> {
    let dlq_msgs_size = load_unsigned(rdb).unwrap_or(0) as usize;
    for _ in 0..dlq_msgs_size {
        match load_each_msg(rdb) {
            Some(msg) => {
                valq.dlq_msgs_mut().push_back(msg);
            }
            None => {
                return Some(std::ptr::null_mut());
            }
        }
    }
    None
}

fn load_delayed_msgs_attributes(
    rdb: *mut RedisModuleIO,
    valq: &mut ValqType,
) -> Option<*mut c_void> {
    let delayed_msg_size = load_unsigned(rdb).unwrap_or(0);
    for _ in 0..delayed_msg_size {
        // load the score for the delayed message
        let score = load_unsigned(rdb).ok()?;
        // load the message itself
        match load_each_msg(rdb) {
            Some(msg) => {
                valq.delayed_msgs_mut().insert(msg, score);
            }
            None => {
                return Some(std::ptr::null_mut());
            }
        }
    }
    None
}

fn load_each_msg(rdb: *mut RedisModuleIO) -> Option<ValqMsg> {
    let id = load_unsigned(rdb).ok()?;
    let body = load_string(rdb).ok()?.to_string();
    // if the timeout_at is 0, it will be loaded as None
    // if the timeout_at is Some, it will be loaded as the actual value
    let timeout_at = load_unsigned(rdb).ok().filter(|&tmp| tmp > 0);
    let delivery_attempts = load_unsigned(rdb).ok()?;
    let msg = ValqMsg::new(id, body, timeout_at, delivery_attempts);
    Some(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rdb_load_null_pointer() {
        let result = rdb_load(std::ptr::null_mut(), 0);
        assert!(result.is_null());
    }
}
