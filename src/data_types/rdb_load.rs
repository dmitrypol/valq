use crate::structs::valq_msg::ValqMsg;
use crate::structs::valq_type::ValqType;
use std::os::raw::c_void;
use valkey_module::logging::log_notice;

pub(crate) extern "C" fn rdb_load(
    rdb: *mut valkey_module::RedisModuleIO,
    _encver: i32,
) -> *mut c_void {
    if rdb.is_null() {
        return std::ptr::null_mut();
    }
    let mut valq = ValqType::new(None, None).unwrap_or_default();
    // save and load must be in the same order
    // load id_sequence
    valq.set_id_sequence(match valkey_module::load_unsigned(rdb) {
        Ok(tmp) => tmp,
        Err(_err) => return std::ptr::null_mut(),
    });
    // load visibility_timeout
    let visibility_timeout = match valkey_module::load_unsigned(rdb) {
        Ok(tmp) => tmp,
        Err(_err) => return std::ptr::null_mut(),
    };
    match valq.set_visibility_timeout(visibility_timeout) {
        Ok(_) => {}
        Err(_err) => return std::ptr::null_mut(),
    }
    // load max_delivery_attempts
    let max_delivery_attempts = match valkey_module::load_unsigned(rdb) {
        Ok(tmp) => tmp,
        Err(_err) => return std::ptr::null_mut(),
    };
    match valq.set_max_delivery_attempts(max_delivery_attempts) {
        Ok(_) => {}
        Err(_err) => return std::ptr::null_mut(),
    };
    // load the size of the msgs VecDeque
    let q_size = valkey_module::load_unsigned(rdb).unwrap_or(0) as usize;
    for _ in 0..q_size {
        // load each message in the msgs VecDeque
        let id = match valkey_module::load_unsigned(rdb) {
            Ok(tmp) => tmp,
            Err(_err) => return std::ptr::null_mut(),
        };
        let body = match valkey_module::load_string(rdb) {
            Ok(tmp) => tmp.to_string(),
            Err(_err) => return std::ptr::null_mut(),
        };
        let timeout_at = match valkey_module::load_unsigned(rdb) {
            // if the timeout_at is 0, it will be loaded as None
            // if the timeout_at is Some, it will be loaded as the actual value
            Ok(tmp) if tmp > 0 => Some(tmp),
            Ok(_) => None,
            Err(_) => return std::ptr::null_mut(),
        };
        let delivery_attempts = match valkey_module::load_unsigned(rdb) {
            Ok(tmp) => tmp,
            Err(_err) => return std::ptr::null_mut(),
        };
        valq.msgs_mut()
            .push_back(ValqMsg::new(id, body, timeout_at, delivery_attempts));
    }
    log_notice(format!("rdb_load: {:?}", valq));
    Box::into_raw(Box::new(valq)) as *mut c_void
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
